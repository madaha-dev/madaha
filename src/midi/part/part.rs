use crate::midi::consts::{DRUM_CHANNEL_ID, PITCH_BEND_MIDDLE};
use crate::midi::interface::EventParser;
use crate::midi::note::Note;
use crate::midi::ram::RAM;
use crate::midi::ram::interface::Memory;
use crate::midi::ram::{MemoryAddr, RAMCallbackEffects};
use crate::voice_manager::{DRUM_BANK_MSB_GS, Program, VoiceManager};

use super::controller::{Controller, ControllerCallback};
use super::entry_select::DataEntrySelect;
use super::nrpn::nrpn_to_addr;
use super::part_engine::PartEngine;
use super::rpn::RPN;

#[derive(Debug, Clone, Copy)]
pub struct Part {
    // 与 RAM 中的 RAM 内存一一对应
    pub id: usize,
    pub engine: PartEngine,
    pub controller: Controller,
    pub rpn: RPN,

    /// Current pitch bend value (14-bit, 0x2000=center)
    pub pitchbend: u16,

    /// Last played note number (for portamento)
    pub last_note: Option<Note>,

    /// Previous bank MSB (for drum mode detection)
    pub prev_bank_msb: u8,
    /// Previous bank LSB (for drum mode detection)
    pub prev_bank_lsb: u8,
    /// Previous program number (for drum mode detection)
    pub prev_program: u8,

    pub data_entry_select: DataEntrySelect,
    pub program_entry: Option<Program>,
}

impl Part {
    pub fn new(id: usize, vm: &VoiceManager, ram: &RAM) -> Self {
        let ram = ram.xg.multi_part[id];
        Self {
            id,
            engine: PartEngine::AWM2,
            controller: Controller::new(),
            rpn: RPN::new(),

            pitchbend: PITCH_BEND_MIDDLE,
            last_note: None,

            prev_bank_lsb: 0xFF,
            prev_bank_msb: 0xFF,
            prev_program: 0xFF,

            data_entry_select: DataEntrySelect::None,
            program_entry: if ram.rcv_channel == DRUM_CHANNEL_ID as u8 {
                Some(vm.get_program(DRUM_BANK_MSB_GS as u8, 0, 0))
            } else if ram.rcv_channel < 0x10 {
                Some(vm.get_program(0, 0, 0))
            } else {
                None
            },
        }
    }

    // in cents
    pub fn get_pitchbend(&self) -> f32 {
        self.rpn.get_pitch_bend_sensitivity() * (self.pitchbend as f32 - 8192.0) / 8192.0
    }
}

impl EventParser<&mut RAM> for Part {
    fn on_controller(&mut self, ram: &mut RAM, cc: u8, value: u8) -> Vec<RAMCallbackEffects> {
        if let Ok(callback) = self.controller.set(self.id as u8, ram, cc, value) {
            use ControllerCallback::*;
            match callback {
                EntryMSBChange(v) => match self.data_entry_select {
                    DataEntrySelect::RPN => {
                        let param = self.controller.get_rpn_param_id();
                        let value = self.rpn.get(param) & 0x7F | (v as u16) << 7;
                        self.on_rpn(ram, param, value);
                        vec![]
                    }
                    DataEntrySelect::NRPN => {
                        let param = self.controller.get_nrpn_param_id();
                        let value = (v as u16) << 7;
                        self.on_nrpn(ram, param, value)
                    }
                    DataEntrySelect::None => vec![],
                },
                EntryLSBChange(v) => {
                    if self.data_entry_select == DataEntrySelect::RPN {
                        let param = self.controller.get_rpn_param_id();
                        let value = self.rpn.get(param) & 0x3F80 | (v as u16);
                        self.on_rpn(ram, param, value);
                    }
                    vec![]
                }
                DataEntrySelectChange(v) => {
                    self.data_entry_select = v;
                    vec![]
                }
                RPNChange(u) => {
                    let param = self.controller.get_rpn_param_id();
                    let value = self.rpn.get(param) as i16 + u as i16 * 0x80;
                    self.on_rpn(ram, param, value as u16);
                    vec![]
                }
                RAMChange(addr, value) => ram.set(addr, value).unwrap_or(vec![]),

                _ => {
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    fn on_rpn(&mut self, ram: &mut RAM, param_id: u16, value: u16) -> Vec<RAMCallbackEffects> {
        (ram.xg.multi_part[self.id].rcv_switches.rcv_rpn != 0).then(|| match param_id {
            // Pitchbend sensitivity
            0x0000 => {
                let v_msb = (value >> 7) as u8;
                let v_lsb = (value & 0x7F) as u8;
                self.rpn.pitchbend_sensitivity = v_msb.min(0x7F);
                self.rpn.pitchbend_cents = v_lsb;
            }
            // Fine tuning
            0x0001 => {
                self.rpn.fine_msb = (value >> 7).min(0x7F) as u8;
                self.rpn.fine_lsb = (value & 0x7F) as u8;
            }
            // Coarse tuning
            0x0002 => {
                self.rpn.coarse = (value >> 7).min(0x7F) as u8;
            }
            // Tuning bank select
            0x0003 => {
                self.rpn.tuning_bank_select = (value >> 7).min(0x7F) as u8;
            }
            // Tuning prog select
            0x0004 => {
                self.rpn.tuning_prog_select = (value >> 7).min(0x7F) as u8;
            }
            _ => {
                // do nothing.
            }
        });

        vec![]
    }

    fn on_nrpn(&mut self, ram: &mut RAM, param_id: u16, value: u16) -> Vec<RAMCallbackEffects> {
        let value = (value >> 7).min(0x7F) as u8;

        let addr = nrpn_to_addr(self.id, ram, param_id);

        if let Some(addr) = addr {
            ram.set(addr, value).unwrap_or(vec![])
        } else {
            vec![]
        }
    }

    fn on_program_change(&mut self, ram: &mut RAM, prog: u8) -> Vec<RAMCallbackEffects> {
        let addr = MemoryAddr::new(0x08, self.id as u8, 0x03);
        ram.set(addr, prog).unwrap_or(vec![])
    }

    fn on_pitchbend(&mut self, ram: &mut RAM, value: u16) -> Vec<RAMCallbackEffects> {
        let ram = &ram.xg.multi_part[self.id];
        (ram.rcv_switches.rcv_pitch_bend != 0 && ram.part_mode == 0)
            .then(|| self.pitchbend = value);
        vec![]
    }

    fn on_cat(&mut self, ram: &mut RAM, pressure: u8) -> Vec<RAMCallbackEffects> {
        todo!()
    }

    fn on_pat(&mut self, ram: &mut RAM, note: Note, pressure: u8) -> Vec<RAMCallbackEffects> {
        todo!()
    }
}
