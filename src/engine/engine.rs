use std::ops::BitAnd;

use wd_log::log_debug_ln;

use crate::{
    config::Config,
    engine::{
        channel::{Channel, DataEntrySelect},
        consts::DEFAULT_MASTER_VOLUME,
        controller::ControllerCallback,
        errors::MidiError,
        event::MidiEvent,
        nrpn,
        ram::{RAM, interface::Memory},
        rpn::RPNType,
        sysex::{
            Event, ManufacturerId, gm::GeneralMIDISysEx, realtime::UniversalRealtimeSysEx,
            roland::RolandSysEx, yamaha::YamahaSysEx,
        },
    },
    get_14bit, get_lsb, get_msb,
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

            channels: {
                let mut data = [Channel::new(); 16];
                for ch in 0..16 {
                    data[ch]._channel = ch;
                    data[ch].controller._channel = ch;
                }
                data
            },
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
                    data_entry_handler_lsb(channel, ram, value);
                }
                ControllerCallback::EntryMSBChange => {
                    data_entry_handler_msb(channel, ram, value);
                }
                ControllerCallback::RPNChange(u) => if u > 0 {},
                _ => return,
            }
        }
    }
}

fn rpn_data_change(channel: &mut Channel, ram: &RAM, u: i8) {
    let rcv_rpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_rpn != 0;
    let rpn_type = RPNType::from((channel.controller.rpn_id_msb, channel.controller.rpn_id_lsb));
    rcv_rpn.then(|| match rpn_type {
        RPNType::PitchbendSensitivity => {
            channel.pitchbend_sensitivity = if u > 0 {
                channel.pitchbend_sensitivity.wrapping_add(1).bitand(0x7F)
            } else if u < 0 {
                channel.pitchbend_cents.wrapping_sub(1).bitand(0x7F)
            } else {
                channel.pitchbend_sensitivity
            };
        }
        RPNType::FineTuning => {
            let mut data = get_14bit!(channel.fine_msb, channel.fine_lsb);
            data = if u > 0 {
                data.wrapping_add(1).bitand(0x3FFF)
            } else if u < 0 {
                data.wrapping_sub(1).bitand(0x3FFF)
            } else {
                data
            };
            channel.fine_msb = get_msb!(data);
            channel.fine_lsb = get_lsb!(data);
        }
        RPNType::CoarseTuning => {
            channel.coarse = if u > 0 {
                channel.coarse.wrapping_add(1).bitand(0x7F)
            } else if u < 0 {
                channel.coarse.wrapping_sub(1).bitand(0x7F)
            } else {
                channel.coarse
            };
        }
        RPNType::TuningBankSelect => {
            channel.tuning_bank_select = if u > 0 {
                channel.tuning_bank_select.wrapping_add(1).bitand(0x7F)
            } else if u < 0 {
                channel.tuning_bank_select.wrapping_sub(1).bitand(0x7F)
            } else {
                channel.tuning_bank_select
            };
        }
        RPNType::TuningProgSelect => {
            channel.tuning_prog_select = if u > 0 {
                channel.tuning_prog_select.wrapping_add(1).bitand(0x7F)
            } else if u < 0 {
                channel.tuning_prog_select.wrapping_sub(1).bitand(0x7F)
            } else {
                channel.tuning_prog_select
            };
        }
    });
}

fn data_entry_handler_msb(
    channel: &mut Channel,
    ram: &mut RAM,
    value: u8,
) -> Result<(), MidiError> {
    let rcv_rpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_rpn != 0;
    let rcv_nrpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_nrpn != 0;
    match channel.data_entry_select {
        DataEntrySelect::None => Ok(()),
        DataEntrySelect::RPN => rcv_rpn
            .then(|| {
                match RPNType::from((channel.controller.rpn_id_msb, channel.controller.rpn_id_lsb))
                {
                    RPNType::PitchbendSensitivity => Ok(channel.pitchbend_sensitivity = value),
                    RPNType::FineTuning => Ok(channel.fine_msb = value),
                    RPNType::CoarseTuning => Ok(channel.coarse = value),
                    RPNType::TuningBankSelect => Ok(channel.tuning_bank_select = value),
                    RPNType::TuningProgSelect => Ok(channel.tuning_prog_select = value),
                }
            })
            .unwrap(),

        DataEntrySelect::NRPN => rcv_nrpn
            .then(|| {
                match nrpn::nrpn_to_addr(
                    ram,
                    channel._channel as u8,
                    channel.controller.nrpn_id_msb,
                    channel.controller.nrpn_id_lsb,
                ) {
                    Some(addr) => ram.set(addr, value),
                    None => Err(MidiError::UnknownNRPN {
                        msb: channel.controller.nrpn_id_msb,
                        lsb: channel.controller.nrpn_id_lsb,
                    }),
                }
            })
            .unwrap(),
    }
}

fn data_entry_handler_lsb(
    channel: &mut Channel,
    ram: &mut RAM,
    value: u8,
) -> Result<(), MidiError> {
    let rcv_rpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_rpn != 0;
    match channel.data_entry_select {
        DataEntrySelect::None => Ok(()),
        DataEntrySelect::RPN => rcv_rpn
            .then(|| {
                match RPNType::from((channel.controller.rpn_id_msb, channel.controller.rpn_id_lsb))
                {
                    RPNType::PitchbendSensitivity => Ok(channel.pitchbend_cents = value),
                    RPNType::FineTuning => Ok(channel.fine_lsb = value),
                    _ => Ok(()),
                }
            })
            .unwrap(),
        DataEntrySelect::NRPN => Ok(()),
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
