use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MW {
    pub pitch_control: u8,
    pub filter_control: u8,
    pub amplitude_control: u8,
    pub lfo_pmod_depth: u8,
    pub lfo_fmod_depth: u8,
    pub lfo_amod_depth: u8,
}

impl MW {
    pub const fn new() -> Self {
        Self {
            pitch_control: 0x40,
            filter_control: 0x40,
            amplitude_control: 0x40,
            lfo_pmod_depth: 0x0A,
            lfo_fmod_depth: 0x00,
            lfo_amod_depth: 0x00,
        }
    }
}

impl Index<usize> for MW {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 0x1D => &self.pitch_control,
            1 | 0x1E => &self.filter_control,
            2 | 0x1F => &self.amplitude_control,
            3 | 0x20 => &self.lfo_pmod_depth,
            4 | 0x21 => &self.lfo_fmod_depth,
            5 | 0x22 => &self.lfo_amod_depth,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for MW {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 0x1D => &mut self.pitch_control,
            1 | 0x1E => &mut self.filter_control,
            2 | 0x1F => &mut self.amplitude_control,
            3 | 0x20 => &mut self.lfo_pmod_depth,
            4 | 0x21 => &mut self.lfo_fmod_depth,
            5 | 0x22 => &mut self.lfo_amod_depth,
            _ => panic!("MW: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for MW {
    fn reset(&mut self) {
        self.pitch_control = 0x40;
        self.filter_control = 0x40;
        self.amplitude_control = 0x40;
        self.lfo_pmod_depth = 0x0A;
        self.lfo_fmod_depth = 0x0;
        self.lfo_amod_depth = 0x0;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=5|0x1D..=0x22) {
            return Err(err);
        }

        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=5|0x1D..=0x22) {
            return Err(err);
        }
        self[addr as usize] = value;
        Ok(vec![RAMCallbackEffects::NoEffect])
    }
}
