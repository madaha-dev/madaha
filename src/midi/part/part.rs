use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::midi::consts::{DRUM_CHANNEL_ID, PITCH_BEND_MIDDLE};
use crate::midi::interface::EventParser;
use crate::midi::note::Note;
use crate::midi::part::backup::BackupSets;
use crate::midi::ram::interface::Memory;
use crate::midi::ram::xg::multi_part::MultiPart;
use crate::midi::ram::xg::multi_part_ext::MultiPartExt;
use crate::midi::ram::{MemoryAddr, RAMCallbackEffects};
use crate::voice_manager::{DRUM_BANK_MSB_GS, Program, VoiceManager};

use super::controller::{Controller, ControllerCallback};
use super::entry_select::DataEntrySelect;
use super::nrpn::nrpn_to_addr;
use super::part_engine::PartEngine;
use super::rpn::RPN;

#[derive(Debug, Clone)]
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

    pub prev_melodic: BackupSets,
    pub prev_rhythm: BackupSets,

    pub data_entry_select: DataEntrySelect,
    pub program_entry: Option<Program>,

    pub ram: Arc<RwLock<MultiPart>>,
    pub ram_ext: Arc<RwLock<MultiPartExt>>,
    pub insertion_effects: Vec<u8>,

    pub cat_value: u8,
    pub pat_values: [u8; 0x80],
}

impl Part {
    pub fn new(
        id: usize,
        vm: &VoiceManager,
        ram: Arc<RwLock<MultiPart>>,
        ram_ext: Arc<RwLock<MultiPartExt>>,
    ) -> Self {
        Self {
            id,
            engine: PartEngine::AWM2,
            controller: Controller::new(),
            rpn: RPN::new(),

            pitchbend: PITCH_BEND_MIDDLE,
            last_note: None,

            prev_melodic: BackupSets::new(),
            prev_rhythm: BackupSets::new(),

            data_entry_select: DataEntrySelect::None,
            program_entry: if let Ok(r) = ram.read().map(|r| r.rcv_channel == DRUM_CHANNEL_ID as u8)
                && r
            {
                Some(vm.get_program(DRUM_BANK_MSB_GS as u8, 0, 0))
            } else if let Ok(r) = ram.read().map(|r| r.rcv_channel < 0x10)
                && r
            {
                Some(vm.get_program(0, 0, 0))
            } else {
                None
            },

            ram,
            ram_ext,

            insertion_effects: vec![],

            cat_value: 0,
            pat_values: [0; 0x80],
        }
    }

    // in cents
    pub fn get_pitchbend(&self) -> f32 {
        self.rpn.get_pitch_bend_sensitivity() * (self.pitchbend as f32 - 8192.0) / 8192.0
    }

    pub fn set_program(&mut self, vm: &VoiceManager, msb: u8, lsb: u8, prog: u8) {
        self.program_entry = Some(vm.get_program(msb, lsb, prog));
    }

    pub fn reset(&mut self, vm: &VoiceManager) {
        let ram = self.ram.clone();
        let ram_ext = self.ram_ext.clone();
        let cat = self.cat_value;
        let pat = self.pat_values;
        *self = Self::new(self.id, vm, ram, ram_ext);

        self.cat_value = cat;
        self.pat_values = pat;
    }

    pub fn get_ram(&self) -> Option<RwLockReadGuard<'_, MultiPart>> {
        self.ram.read().ok()
    }

    pub fn get_ram_mut(&self) -> Option<RwLockWriteGuard<'_, MultiPart>> {
        self.ram.write().ok()
    }
}

impl EventParser for Part {
    fn on_controller(&mut self, _channel: u8, cc: u8, value: u8) -> Vec<RAMCallbackEffects> {
        if let Ok(callback) = self.controller.set(self.id as u8, &self.ram, cc, value) {
            use ControllerCallback::*;
            match callback {
                EntryMSBChange(v) => match self.data_entry_select {
                    DataEntrySelect::RPN => {
                        let param = self.controller.get_rpn_param_id();
                        let value = self.rpn.get(param) & 0x7F | (v as u16) << 7;
                        self.on_rpn(_channel, param, value);
                        vec![]
                    }
                    DataEntrySelect::NRPN => {
                        let param = self.controller.get_nrpn_param_id();
                        let value = (v as u16) << 7;
                        self.on_nrpn(_channel, param, value)
                    }
                    DataEntrySelect::None => vec![],
                },
                EntryLSBChange(v) => {
                    if self.data_entry_select == DataEntrySelect::RPN {
                        let param = self.controller.get_rpn_param_id();
                        let value = self.rpn.get(param) & 0x3F80 | (v as u16);
                        self.on_rpn(_channel, param, value);
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
                    self.on_rpn(_channel, param, value as u16);
                    vec![]
                }
                RAMChange(addr, value) => self
                    .ram
                    .write()
                    .map(|mut w| w.set(addr, value).unwrap_or(vec![]))
                    .unwrap_or(vec![]),

                _ => {
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    fn on_rpn(&mut self, _channel: u8, param_id: u16, value: u16) -> Vec<RAMCallbackEffects> {
        if let Ok(r) = self.ram.read().map(|r| r.rcv_switches.rcv_rpn != 0)
            && r
        {
            match param_id {
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
            }
        }

        vec![]
    }

    fn on_nrpn(&mut self, _channel: u8, param_id: u16, value: u16) -> Vec<RAMCallbackEffects> {
        let value = (value >> 7).min(0x7F) as u8;

        nrpn_to_addr(self.id, &self.ram, param_id)
            .map(|addr| {
                addr.iter()
                    .map(|&a| {
                        self.ram
                            .write()
                            .map(|mut r| r.set(a, value).unwrap_or(vec![]))
                            .unwrap_or(vec![])
                    })
                    .flatten()
                    .collect()
            })
            .unwrap_or(vec![])
    }

    fn on_program_change(&mut self, _channel: u8, prog: u8) -> Vec<RAMCallbackEffects> {
        let addr = MemoryAddr::new(0x08, self.id as u8, 0x03);
        if let Ok(mut ram) = self.ram.write() {
            if ram.rcv_switches.rcv_note_message != 0 && ram.rcv_switches.rcv_program_change != 0 {
                return ram.set(addr, prog).unwrap_or(vec![]);
            }
        }
        vec![]
    }

    fn on_pitchbend(&mut self, _channel: u8, value: u16) -> Vec<RAMCallbackEffects> {
        if let Ok(ram) = self.ram.read() {
            (ram.rcv_switches.rcv_pitch_bend != 0 && ram.part_mode == 0)
                .then(|| self.pitchbend = value);
        }
        vec![]
    }

    fn on_cat(&mut self, _channel: u8, pressure: u8) -> Vec<RAMCallbackEffects> {
        if let Ok(ram) = self.ram.read()
            && ram.rcv_switches.rcv_chan_aftertouch != 0
        {
            self.cat_value = pressure;
        }
        vec![]
    }

    fn on_pat(&mut self, _channel: u8, note: Note, pressure: u8) -> Vec<RAMCallbackEffects> {
        if let Ok(ram) = self.ram.read()
            && ram.rcv_switches.rcv_poly_aftertouch != 0
        {
            self.pat_values[note as usize] = pressure;
        }

        vec![]
    }
}
