use wd_log::log_debug_ln;

use crate::{
    config::Config,
    engine::{
        channel::Channel,
        consts::DEFAULT_MASTER_VOLUME,
        event::MidiEvent,
        ram::{RAM, interface::Memory},
        rpn::RPN,
        sysex::{
            Event, ManufacturerId, gm::GeneralMIDISysEx, realtime::UniversalRealtimeSysEx,
            roland::RolandSysEx, yamaha::YamahaSysEx,
        },
    },
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
}

impl Engine {
    pub fn new(cfg: &Config) -> Self {
        Self {
            master_volume: DEFAULT_MASTER_VOLUME,
            note_cent_table: NOTE_CENT_TABLE,
            dev_id: 0x0, // TODO: should change by config later
            effect_group: 1,

            channels: [Channel::new(); 16],
            ram: RAM::new(MidiResetMode::GM, xg_drum_data),
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
        let current_tune_bank = self.channels[channel].rpns[RPN::TuningBankSelect as usize];
        let current_tune_prog = self.channels[channel].rpns[RPN::TuningProgSelect as usize];
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
            } => self.on_controller(channel as usize, controller as usize, value),
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

    fn on_controller(&mut self, channel: usize, controller: usize, value: u8) {
        if controller > 127 {
            return;
        }

        match controller {
            120 => todo!(),
            121 => {
                if value == 0 {
                    self.channels[channel].reset_controller();
                }
            }
            122..127 => todo!(),
            _ => self.channels[channel].controllers[controller] = value,
        }
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
