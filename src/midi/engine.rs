use std::sync::mpsc::SyncSender;
use wd_log::{log_debug_ln, log_warn_ln};

use crate::midi::consts::MAX_PART_SIZE;
use crate::midi::ram::RAMCallbackEffects;
use crate::voice_manager::DRUM_BANK_MSB_GS;
use crate::voice_manager::VoiceManager;
use crate::{
    config::Config,
    midi::{
        consts::DEFAULT_MASTER_VOLUME,
        event::MidiEvent,
        interface::EventParser,
        note::Note,
        part::Part,
        ram::RAM,
        sysex::{
            Event, ManufacturerId, gm::GeneralMIDISysEx, realtime::UniversalRealtimeSysEx,
            roland::RolandSysEx, yamaha::YamahaSysEx,
        },
    },
};

pub type NoteCentTable = [[[f32; 128]; 128]; 128];

#[derive(Debug)]
pub struct Engine {
    /// Master volume for SysEx 0x7F, little endian
    /// Default = 0x4000
    pub master_volume: u16,
    pub note_cent_table: NoteCentTable,
    pub dev_id: u8,
    pub effect_group: usize,

    pub parts: Box<[Part]>,

    pub ram: RAM,

    pub client_active: bool,
    pub voice_manager: VoiceManager,

    pub chan_tx: SyncSender<MidiEvent>,
}

impl Engine {
    pub fn new(cfg: &Config, tx: SyncSender<MidiEvent>) -> Self {
        let voice_manager = VoiceManager::load_tbl(cfg).unwrap();
        let drum_data = voice_manager
            .get_drum_setup(DRUM_BANK_MSB_GS as u8, 0)
            .unwrap();
        let ram = RAM::new(MidiResetMode::GM, drum_data);

        let parts = (0..MAX_PART_SIZE)
            .map(|i| {
                Part::new(
                    i,
                    &voice_manager,
                    ram.xg.multi_part[i].clone(),
                    ram.xg.multi_part_ext[i].clone(),
                )
            })
            .collect();

        Self {
            master_volume: DEFAULT_MASTER_VOLUME,
            note_cent_table: NOTE_CENT_TABLE,
            dev_id: cfg.midi.device_id,
            effect_group: 1,
            parts,
            ram,
            client_active: false,
            voice_manager,
            chan_tx: tx,
        }
    }

    pub fn get_sample_playspeed_ratio(&self, channel: usize, sample: f32, note: usize) -> f32 {
        let current_tune_bank = self.parts[channel].rpn.tuning_bank_select;
        let current_tune_prog = self.parts[channel].rpn.tuning_prog_select;
        self.note_cent_table[current_tune_bank as usize][current_tune_prog as usize][note] / sample
    }

    pub fn on_event(&mut self, ev: MidiEvent) {
        log_debug_ln!("got midi event {:?}", ev);
        let callbacks = match ev {
            MidiEvent::SysEx {
                manufacturer_id,
                data,
            } => self.on_sysex(manufacturer_id, data),
            MidiEvent::ControlChange {
                channel,
                controller,
                value,
            } => self.on_controller(channel, controller, value),
            MidiEvent::RPN {
                channel,
                parameter,
                value,
            } => self.on_rpn(channel, parameter, value),
            MidiEvent::NRPN {
                channel,
                parameter,
                value,
            } => self.on_nrpn(channel, parameter, value),
            MidiEvent::ProgramChange { channel, program } => {
                self.on_program_change(channel, program)
            }
            MidiEvent::PitchBend { channel, value } => self.on_pitchbend(channel, value),
            MidiEvent::ChannelPressure { channel, pressure } => self.on_cat(channel, pressure),
            MidiEvent::PolyPressure {
                channel,
                note,
                pressure,
            } => self.on_pat(channel, note, pressure),
            MidiEvent::NoteOn {
                channel: _,
                note: _,
                velocity: _,
                off_velocity: _,
                duration: _,
            }
            | MidiEvent::NoteOff {
                channel: _,
                note: _,
                velocity: _,
                off_velocity: _,
                duration: _,
            } => {
                let _ = self.chan_tx.send(ev);
                vec![]
            }
            MidiEvent::ActiveSensing => self.on_active_sensing(),

            _ => {
                log_warn_ln!("non supported event: {:?}", ev);
                vec![]
            }
        };

        self.hook_exec(callbacks);
    }

    fn on_active_sensing(&mut self) -> Vec<RAMCallbackEffects> {
        if !self.client_active {
            // start a timer on another thread as watchdog
        }
        self.client_active = true;

        // TODO watchdog for 500ms, then reset.

        vec![]
    }

    pub fn find_all_parts(&mut self, channel: u8) -> Box<[usize]> {
        self.ram
            .xg
            .multi_part
            .iter()
            .enumerate()
            .filter(|(_, mp)| mp.read().map(|r| r.rcv_channel == channel).unwrap_or(false))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn reset(&mut self, mode: MidiResetMode) {
        self.ram.reset_mode = mode;
        self.parts
            .iter_mut()
            .for_each(|p| p.reset(&self.voice_manager));
    }
}

impl EventParser for Engine {
    fn on_sysex(&mut self, mfid: ManufacturerId, data: Box<[u8]>) -> Vec<RAMCallbackEffects> {
        match mfid {
            ManufacturerId::UniversalRealTime => UniversalRealtimeSysEx::parse(self, data),
            ManufacturerId::UniversalNonRealTime => GeneralMIDISysEx::parse(self, data),
            ManufacturerId::Yamaha => YamahaSysEx::parse(self, data),
            ManufacturerId::Roland => RolandSysEx::parse(self, data),
            _ => {
                log_debug_ln!("non-supported manufacturer, ignore");
                vec![]
            }
        }
    }

    fn on_controller(&mut self, channel: u8, cc: u8, value: u8) -> Vec<RAMCallbackEffects> {
        let parts = self.find_all_parts(channel);
        parts
            .iter()
            .map(|&i| self.parts[i].on_controller(channel, cc, value))
            .flatten()
            .collect()
    }

    fn on_program_change(&mut self, channel: u8, program: u8) -> Vec<RAMCallbackEffects> {
        let parts = self.find_all_parts(channel);
        parts
            .iter()
            .map(|&i| self.parts[i].on_program_change(channel, program))
            .flatten()
            .collect()
    }

    fn on_pitchbend(&mut self, channel: u8, value: u16) -> Vec<RAMCallbackEffects> {
        self.find_all_parts(channel)
            .iter()
            .map(|&i| self.parts[i].on_pitchbend(channel, value))
            .flatten()
            .collect()
    }

    fn on_rpn(&mut self, channel: u8, param: u16, value: u16) -> Vec<RAMCallbackEffects> {
        self.find_all_parts(channel)
            .iter()
            .map(|&i| self.parts[i].on_rpn(channel, param, value))
            .flatten()
            .collect()
    }

    fn on_nrpn(&mut self, channel: u8, param: u16, value: u16) -> Vec<RAMCallbackEffects> {
        self.find_all_parts(channel)
            .iter()
            .map(|&i| self.parts[i].on_nrpn(channel, param, value))
            .flatten()
            .collect()
    }

    fn on_cat(&mut self, channel: u8, pressure: u8) -> Vec<RAMCallbackEffects> {
        self.find_all_parts(channel)
            .iter()
            .map(|&i| self.parts[i].on_cat(channel, pressure))
            .flatten()
            .collect()
    }

    fn on_pat(&mut self, channel: u8, note: Note, pressure: u8) -> Vec<RAMCallbackEffects> {
        self.find_all_parts(channel)
            .iter()
            .map(|&i| self.parts[i].on_pat(channel, note, pressure))
            .flatten()
            .collect()
    }
}

const NOTE_CENT_TABLE: NoteCentTable = {
    let mut notes = [0.0; 128];
    let mut note = 0;
    while note < 128 {
        notes[note] = (note as f32) * 100.0;
        note += 1;
    }

    let mut progs = [[0.0; 128]; 128];
    let mut prog = 0;
    while prog < 128 {
        progs[prog] = notes;
        prog += 1;
    }

    let mut data = [[[0.0; 128]; 128]; 128];
    let mut bank = 0;
    while bank < 128 {
        data[bank] = progs;
        bank += 1;
    }
    data
};

#[derive(Debug, PartialEq)]
pub enum MidiResetMode {
    GM,
    XG,
    GS,
    GM2,
}
