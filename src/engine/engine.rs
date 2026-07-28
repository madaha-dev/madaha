use std::sync::mpsc::{self, Receiver, Sender};
//use std::sync::Arc;
//use std::cell::RefCell;
use wd_log::log_debug_ln;

use crate::engine::consts::DRUM_CHANNEL_ID;
use crate::voice_manager::VoiceManager;
use crate::voice_manager::{DRUM_BANK_MSB_GS, DRUM_BANK_MSB_XG};
use crate::{
    config::Config,
    engine::{
        channel::Channel,
        consts::DEFAULT_MASTER_VOLUME,
        controller::ControllerCallback,
        data_entry::{data_entry_handler_lsb, data_entry_handler_msb},
        errors::MidiError,
        event::MidiEvent,
        ram::{MemoryAddr, RAM, interface::Memory},
        rpn::rpn_data_change,
        sysex::{
            Event, ManufacturerId, gm::GeneralMIDISysEx, realtime::UniversalRealtimeSysEx,
            roland::RolandSysEx, yamaha::YamahaSysEx,
        },
    },
    get_lsb, get_msb,
};

pub type NoteCentTable = [[[f64; 128]; 128]; 128];

#[derive(Debug)]
pub struct Engine {
    /// Master volume for SysEx 0x7F, little endian
    /// Default = 0x4000
    pub master_volume: u16,
    pub note_cent_table: NoteCentTable,
    pub dev_id: u8,
    pub effect_group: usize,

    pub channels: [Channel; 16],

    pub ram: RAM,

    pub client_active: bool,
    pub voice_manager: VoiceManager,

    pub chan_tx: Sender<MidiEvent>,
    pub chan_rx: Receiver<MidiEvent>,
}

impl Engine {
    pub fn new(cfg: &Config) -> Self {
        let voice_manager = VoiceManager::load_tbl(cfg).unwrap();
        let drum_data = voice_manager
            .get_drum_setup(DRUM_BANK_MSB_GS as u8, 0)
            .unwrap();
        let ram = RAM::new(MidiResetMode::GM, drum_data);

        let (tx, rx) = mpsc::channel();

        let channels = {
            let mut data = [Channel::new(0, &voice_manager); 16];
            for (ch, item) in data.iter_mut().enumerate() {
                item._channel = ch;
                item.controller._channel = ch;
                if ch == DRUM_CHANNEL_ID {
                    item.program_entry = voice_manager.get_program(DRUM_BANK_MSB_XG as u8, 0, 0);
                }
            }
            data
        };

        Self {
            master_volume: DEFAULT_MASTER_VOLUME,
            note_cent_table: NOTE_CENT_TABLE,
            dev_id: cfg.audio.device_id,
            effect_group: 1,
            channels,
            ram,
            client_active: false,
            voice_manager,

            chan_rx: rx,
            chan_tx: tx,
        }
    }

    pub fn gm_reset(&mut self) {
        self.ram.reset_mode = MidiResetMode::GM;
        for i in 0..16 {
            self.channels[i].reset();
        }
    }

    pub fn gm2_reset(&mut self) {
        self.ram.reset_mode = MidiResetMode::GM2;
        for i in 0..16 {
            self.channels[i].reset();
        }
    }

    pub fn xg_reset(&mut self) {
        self.ram.reset_mode = MidiResetMode::XG;
        self.ram.reset();
    }

    pub fn gs_reset(&mut self) {
        self.ram.reset_mode = MidiResetMode::GS;
        self.ram.reset();
    }

    pub fn get_sample_playspeed_ratio(&self, channel: usize, sample: f64, note: usize) -> f64 {
        let current_tune_bank = self.channels[channel].tuning_bank_select;
        let current_tune_prog = self.channels[channel].tuning_prog_select;
        self.note_cent_table[current_tune_bank as usize][current_tune_prog as usize][note] / sample
    }

    pub fn on_event(&mut self, ev: MidiEvent) {
        log_debug_ln!("got midi event {:?}", ev);
        match ev {
            MidiEvent::SysEx {
                manufacturer_id,
                data,
            } => {
                self.on_sysex(manufacturer_id, data);
            }
            MidiEvent::ControlChange {
                channel,
                controller,
                value,
            } => self.on_controller(channel as usize, controller, value),
            MidiEvent::RPN {
                channel,
                parameter,
                value,
            } => {
                let id_msb = get_msb!(parameter);
                let id_lsb = get_lsb!(parameter);
                let value_msb = get_msb!(value);
                let value_lsb = get_lsb!(value);
                self.on_controller(channel.into(), 100, id_lsb);
                self.on_controller(channel.into(), 101, id_msb);
                self.on_controller(channel.into(), 6, value_msb);
                self.on_controller(channel.into(), 38, value_lsb);
            }
            MidiEvent::NRPN {
                channel,
                parameter,
                value,
            } => {
                let id_msb = get_msb!(parameter);
                let id_lsb = get_lsb!(parameter);
                let value_msb = get_msb!(value);
                self.on_controller(channel.into(), 98, id_lsb);
                self.on_controller(channel.into(), 99, id_msb);
                self.on_controller(channel.into(), 6, value_msb);
            }
            MidiEvent::ProgramChange { channel, program } => {
                self.on_program_change(channel as usize, program);
            }
            MidiEvent::PitchBend { channel, value } => {
                self.on_pitchbend(channel as usize, value);
            }
            MidiEvent::ChannelPressure { channel, pressure } => todo!(),
            MidiEvent::PolyPressure {
                channel,
                note,
                pressure,
            } => todo!(),
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
            }
            MidiEvent::ActiveSensing => self.on_active_sensing(),

            _ => todo!(),
        }
    }

    fn on_sysex(&mut self, mfid: ManufacturerId, data: Box<[u8]>) {
        match mfid {
            ManufacturerId::UniversalRealTime => UniversalRealtimeSysEx::parse(self, data),
            ManufacturerId::UniversalNonRealTime => GeneralMIDISysEx::parse(self, data),
            ManufacturerId::Yamaha => YamahaSysEx::parse(self, data),
            ManufacturerId::Roland => RolandSysEx::parse(self, data),
            _ => {
                log_debug_ln!("non-supported manufacturer, ignore")
            }
        }
    }

    fn on_controller(&mut self, channel: usize, cc: u8, value: u8) {
        let ram = &mut self.ram;

        let channel = &mut self.channels[channel];
        let controller = &mut channel.controller;

        if let Some(callback) = controller.set(ram, cc, value).ok() {
            match callback {
                ControllerCallback::DataEntrySelectChange(v) => channel.data_entry_select = v,
                ControllerCallback::EntryLSBChange => {
                    let _ = data_entry_handler_lsb(channel, ram, value);
                }
                ControllerCallback::EntryMSBChange => {
                    if let Ok(addr) = data_entry_handler_msb(channel, ram, value) {
                        if addr.is_valid() {
                            let _ = self.mem_set(addr, value);
                        }
                    };
                }
                ControllerCallback::RPNChange(u) => rpn_data_change(channel, ram, u),
                ControllerCallback::RAMChange(addr, value) => {
                    let _ = self.mem_set(addr, value);
                }
                _ => return,
            }
        }
    }

    fn on_program_change(&mut self, channel: usize, program: u8) {
        let rcv_prog_change = self.ram.xg.multi_part[channel]
            .rcv_switches
            .rcv_program_change
            != 0;
        if !rcv_prog_change {
            return;
        }

        self.mem_set(MemoryAddr::new(0x08, channel as u8, 0x03), program);
    }

    fn on_pitchbend(&mut self, channel: usize, value: u16) {
        self.channels[channel].pitchbend = value;
    }

    fn on_active_sensing(&mut self) {
        if !self.client_active {
            // start a timer on another thread as watchdog
        }
        self.client_active = true;

        // TODO watchdog for 500ms, then reset.
    }

    pub fn mem_set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        self.pre_hooks(addr);
        self.ram.set(addr, value)?;
        self.post_hooks(addr);

        Ok(())
    }
}

const NOTE_CENT_TABLE: NoteCentTable = {
    let mut notes = [0.0f64; 128];
    let mut note = 0;
    while note < 128 {
        notes[note] = (note as f64) * 100.0;
        note += 1;
    }

    let mut progs = [[0.0f64; 128]; 128];
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
