use std::sync::Arc;

use crate::midi::MIDICallbackEffects;
use crate::midi::consts::{DRUM_CHANNEL_ID, PITCH_BEND_MIDDLE};
use crate::midi::interface::{EventParser, PitchGetter};
use crate::midi::note::Note;
use crate::midi::part::backup::BackupSets;
use crate::double_buffer::DoubleBuffered;
use crate::midi::ram::{ MemoryAddr};
use crate::midi::ram::interface::Memory;
use crate::midi::ram::xg::multi_part::MultiPart;
use crate::midi::ram::xg::multi_part_ext::MultiPartExt;
use crate::voice_manager::{DRUM_BANK_MSB_GS, Program, VoiceManager};

use super::controller::{Controller, ControllerCallback};
use super::entry_select::DataEntrySelect;
use super::nrpn::nrpn_to_addr;
use super::part_engine::PartEngine;
use super::rpn::RPN;

#[derive(Debug, Clone)]
pub struct Part {
    // One-to-one correspondence with the RAM memory in RAM
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
    pub program_entry: Option<std::sync::Arc<Program>>,

    pub ram: Arc<DoubleBuffered<MultiPart>>,
    pub ram_ext: Arc<DoubleBuffered<MultiPartExt>>,
    pub insertion_effects: Vec<u8>,

    pub cat_value: u8,
    pub pat_values: [u8; 0x80],
}

impl Part {
    pub fn new(
        id: usize,
        vm: &VoiceManager,
        ram: Arc<DoubleBuffered<MultiPart>>,
        ram_ext: Arc<DoubleBuffered<MultiPartExt>>,
    ) -> Self {
        Self {
            id,
            engine: PartEngine::AWM2,
            controller: Controller::new(),
            rpn: RPN::new(ram.clone()),

            pitchbend: PITCH_BEND_MIDDLE,
            last_note: None,

            prev_melodic: BackupSets::new(),
            prev_rhythm: BackupSets::new(),

            data_entry_select: DataEntrySelect::None,
            program_entry: {
                let r = ram.snapshot();
                if r.rcv_channel == DRUM_CHANNEL_ID as u8 {
                    vm.get_program(DRUM_BANK_MSB_GS as u8, 0, 0)
                } else if r.rcv_channel < 0x10 {
                    vm.get_program(0, 0, 0)
                } else {
                    None
                }
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
        self.program_entry = vm.get_program(msb, lsb, prog);
    }

    pub fn reset(&mut self, vm: &VoiceManager) {
        let ram = self.ram.clone();
        let ram_ext = self.ram_ext.clone();

        *self = Self::new(self.id, vm, ram, ram_ext);
    }

    pub fn get_ram(&self) -> Arc<MultiPart> {
        self.ram.snapshot()
    }

    /// Drum channel detection: part_mode != 0 (rhythm setup) or rcv_channel == 9 (channel 10)
    pub fn is_drum_channel(&self) -> bool {
        let ram = self.ram.snapshot();
        ram.part_mode != 0 || ram.rcv_channel == 9
    }
}

impl PitchGetter for Part {
    fn get_coarse(&self) -> i8 {
        self.get_ram().get_coarse() + self.rpn.get_coarse()
    }

    fn get_delta_pitch(&self, _note: Note) -> f32 {
        self.get_ram().get_delta_pitch(_note)
            + self.get_pitchbend()
            + self.rpn.get_delta_pitch(_note)
    }
}

impl EventParser for Part {
    fn on_controller(&mut self, _channel: u8, cc: u8, value: u8) -> Vec<MIDICallbackEffects> {
        // YAMAHA: drum parts ignore these controllers
        // (BANK SELECT LSB, PORTAMENTO, SOFT PEDAL, MONO/POLY)
        if self.is_drum_channel() && matches!(cc, 5 | 32 | 65 | 67 | 84 | 126 | 127) {
            return vec![];
        }
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
                    self.on_rpn(_channel, param, value as u16)
                }
                RAMChange(addr, value) => {
                    let mut effects = vec![];
                    self.ram.write_with(|w| {
                        effects = w.set(addr, value).unwrap_or(vec![]);
                    });
                    effects
                }
                ResetAllController => {
                    vec![MIDICallbackEffects::ChannelResetAllController { part_id: self.id }]
                }
                PolyMonoChange(v) => {
                    let mut effects = vec![];
                    self.ram.write_with(|w| {
                        effects = w.set(MemoryAddr::new(0x08, self.id as u8, 0x5), v)
                            .unwrap_or(vec![]);
                    });
                    effects
                }
                AllNoteOFF => vec![MIDICallbackEffects::AllNotesOFF { part_id: self.id }],
                AllSoundOFF => vec![MIDICallbackEffects::AllSoundOFF { part_id: self.id }],
                _ => {
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    fn on_rpn(&mut self, _channel: u8, param_id: u16, value: u16) -> Vec<MIDICallbackEffects> {
        if self.ram.snapshot().rcv_switches.rcv_rpn != 0 {
            match param_id {
                // Pitchbend sensitivity (ignored on drum channels)
                0x0000 => {
                    if !self.is_drum_channel() {
                        let v_msb = (value >> 7) as u8;
                        let v_lsb = (value & 0x7F) as u8;
                        self.ram.write_with(|ram| {
                            ram.bend.pitch_control =
                                v_msb.wrapping_add(0x40).clamp(0x28, 0x58);
                        });
                        self.rpn.pitchbend_cents = v_lsb;
                    }
                }
                // Fine tuning
                0x0001 => {
                    self.rpn.fine_msb = (value >> 7).min(0x7F) as u8;
                    self.rpn.fine_lsb = (value & 0x7F) as u8;
                }
                // Coarse tuning
                0x0002 => {
                    self.rpn.coarse = (value >> 7).clamp(0x28, 0x58) as u8;
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

    fn on_nrpn(&mut self, _channel: u8, param_id: u16, value: u16) -> Vec<MIDICallbackEffects> {
        let value = (value >> 7).min(0x7F) as u8;

        nrpn_to_addr(self.id, &self.ram, param_id).map_or(vec![], |addr| {
            addr.iter()
                .map(|&a| {
                    let mut effects = vec![];
                    self.ram.write_with(|r| {
                        effects = r.set(a, value).unwrap_or(vec![]);
                    });
                    effects
                })
                .flatten()
                .collect()
        })
    }

    fn on_program_change(&mut self, _channel: u8, prog: u8) -> Vec<MIDICallbackEffects> {
        let addr = MemoryAddr::new(0x08, self.id as u8, 0x03);
        let ram = self.ram.snapshot();
        if ram.rcv_switches.rcv_note_message != 0 && ram.rcv_switches.rcv_program_change != 0 {
            let mut effects = vec![];
            self.ram.write_with(|ram| {
                effects = ram.set(addr, prog).unwrap_or(vec![]);
            });
            return effects;
        }
        vec![]
    }

    fn on_pitchbend(&mut self, _channel: u8, value: u16) -> Vec<MIDICallbackEffects> {
        let ram = self.ram.snapshot();
        // XG: pitch bend is ignored on drum channels (fixed drum pitch)
        if ram.rcv_switches.rcv_pitch_bend != 0 && !self.is_drum_channel() {
            self.pitchbend = value;
        }
        vec![]
    }

    fn on_cat(&mut self, _channel: u8, pressure: u8) -> Vec<MIDICallbackEffects> {
        let ram = self.ram.snapshot();
        if ram.rcv_switches.rcv_chan_aftertouch != 0 {
            self.cat_value = pressure;
        }
        vec![]
    }

    fn on_pat(&mut self, _channel: u8, note: Note, pressure: u8) -> Vec<MIDICallbackEffects> {
        let ram = self.ram.snapshot();
        // YAMAHA: POLY AFTER TOUCH has no effect on drum parts
        if ram.rcv_switches.rcv_poly_aftertouch != 0 && !self.is_drum_channel() {
            self.pat_values[note as usize] = pressure;
        }

        vec![]
    }
}
