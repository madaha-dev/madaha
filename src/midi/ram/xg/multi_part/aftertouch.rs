use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AfterTouch {
    pub pitch_control: u8,
    pub filter_control: u8,
    pub amplitude_control: u8,
    pub lfo_pmod_depth: u8,
    pub lfo_fmod_depth: u8,
    pub lfo_amod_depth: u8,
}

impl AfterTouch {
    pub const fn new() -> Self {
        Self {
            pitch_control: 0x40,
            filter_control: 0x40,
            amplitude_control: 0x40,
            lfo_pmod_depth: 0,
            lfo_fmod_depth: 0,
            lfo_amod_depth: 0,
        }
    }
}

impl Index<usize> for AfterTouch {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            // CAT: 0x4D-0x52, PAT: 0x53-0x58
            0 | 0x4D | 0x53 => &self.pitch_control,
            1 | 0x4E | 0x54 => &self.filter_control,
            2 | 0x4F | 0x55 => &self.amplitude_control,
            3 | 0x50 | 0x56 => &self.lfo_pmod_depth,
            4 | 0x51 | 0x57 => &self.lfo_fmod_depth,
            5 | 0x52 | 0x58 => &self.lfo_amod_depth,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for AfterTouch {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 0x4D | 0x53 => &mut self.pitch_control,
            1 | 0x4E | 0x54 => &mut self.filter_control,
            2 | 0x4F | 0x55 => &mut self.amplitude_control,
            3 | 0x50 | 0x56 => &mut self.lfo_pmod_depth,
            4 | 0x51 | 0x57 => &mut self.lfo_fmod_depth,
            5 | 0x52 | 0x58 => &mut self.lfo_amod_depth,
            _ => panic!("AfterTouch: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for AfterTouch {
    fn reset(&mut self) {
        self.pitch_control = 0x40;
        self.filter_control = 0x40;
        self.amplitude_control = 0x40;
        self.lfo_pmod_depth = 0;
        self.lfo_fmod_depth = 0;
        self.lfo_amod_depth = 0;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=5|0x4D..=0x58) {
            return Err(err);
        }

        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=5|0x4D..=0x58) {
            return Err(err);
        }
        self[addr as usize] = value;

        Ok(vec![RAMCallbackEffects::NoEffect])
    }
}
