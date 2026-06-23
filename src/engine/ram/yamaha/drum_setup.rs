use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrumSetup {
    pub pitch_coarse: u8,
    pub pitch_fine: u8,
    pub level: u8,
    pub alternate_group: u8,
    pub pan: u8,
    pub reverb_send: u8,
    pub chorus_send: u8,
    pub variation_send: u8,
    pub key_assign: u8,
    pub rcv_note_off: u8,
    pub rcv_note_on: u8,
    pub filter_cutoff_freq: u8,
    pub filter_resonance: u8,
    pub eg_attack_rate: u8,
    pub eg_decay1_rate: u8,
    pub eg_decay2_rate: u8,

    pub _init_data: &'static Box<[u8]>,
}

impl DrumSetup {
    pub const fn new(data: &'static Box<[u8]>) -> Self {
        Self {
            pitch_coarse: 0x40,
            pitch_fine: 0x40,
            level: data[2],
            alternate_group: data[3],
            pan: data[4],
            reverb_send: data[5],
            chorus_send: data[6],
            variation_send: 0x7F,
            key_assign: 0x00,
            rcv_note_off: data[9],
            rcv_note_on: 0x01,
            filter_cutoff_freq: 0x40,
            filter_resonance: 0x40,
            eg_attack_rate: 0x40,
            eg_decay1_rate: 0x40,
            eg_decay2_rate: 0x40,

            _init_data: data,
        }
    }
}

impl Index<usize> for DrumSetup {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.pitch_coarse,
            1 => &self.pitch_fine,
            2 => &self.level,
            3 => &self.alternate_group,
            4 => &self.pan,
            5 => &self.reverb_send,
            6 => &self.chorus_send,
            7 => &self.variation_send,
            8 => &self.key_assign,
            9 => &self.rcv_note_off,
            10 => &self.rcv_note_on,
            11 => &self.filter_cutoff_freq,
            12 => &self.filter_resonance,
            13 => &self.eg_attack_rate,
            14 => &self.eg_decay1_rate,
            15 => &self.eg_decay2_rate,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for DrumSetup {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.pitch_coarse,
            1 => &mut self.pitch_fine,
            2 => &mut self.level,
            3 => &mut self.alternate_group,
            4 => &mut self.pan,
            5 => &mut self.reverb_send,
            6 => &mut self.chorus_send,
            7 => &mut self.variation_send,
            8 => &mut self.key_assign,
            9 => &mut self.rcv_note_off,
            10 => &mut self.rcv_note_on,
            11 => &mut self.filter_cutoff_freq,
            12 => &mut self.filter_resonance,
            13 => &mut self.eg_attack_rate,
            14 => &mut self.eg_decay1_rate,
            15 => &mut self.eg_decay2_rate,
            _ => panic!("DrumSetup: index {} out of bounds", index),
        }
    }
}

impl Memory for DrumSetup {
    fn reset(&mut self) {
        *self = DrumSetup::new(self._init_data);
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if addr > 15 {
            return Err(err);
        }
        Ok(self[addr])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if addr > 15 {
            return Err(err);
        }
        Ok(self[addr] = value)
    }
}
