use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AC {
    pub controller_number: u8,
    pub pitch_control: u8,
    pub filter_control: u8,
    pub amplitude_control: u8,
    pub lfo_pmod_depth: u8,
    pub lfo_fmod_depth: u8,
    pub lfo_amod_depth: u8,
}

impl AC {
    pub const fn new() -> Self {
        Self {
            controller_number: 0x11,
            pitch_control: 0x40,
            filter_control: 0x40,
            amplitude_control: 0x40,
            lfo_pmod_depth: 0,
            lfo_fmod_depth: 0,
            lfo_amod_depth: 0,
        }
    }
}

impl Index<usize> for AC {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 0x59 | 0x60 => &self.controller_number,
            1 | 0x5A | 0x61 => &self.pitch_control,
            2 | 0x5B | 0x62 => &self.filter_control,
            3 | 0x5C | 0x63 => &self.amplitude_control,
            4 | 0x5D | 0x64 => &self.lfo_pmod_depth,
            5 | 0x5E | 0x65 => &self.lfo_fmod_depth,
            6 | 0x5F | 0x66 => &self.lfo_amod_depth,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for AC {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 0x59 | 0x60 => &mut self.controller_number,
            1 | 0x5A | 0x61 => &mut self.pitch_control,
            2 | 0x5B | 0x62 => &mut self.filter_control,
            3 | 0x5C | 0x63 => &mut self.amplitude_control,
            4 | 0x5D | 0x64 => &mut self.lfo_pmod_depth,
            5 | 0x5E | 0x65 => &mut self.lfo_fmod_depth,
            6 | 0x5F | 0x66 => &mut self.lfo_amod_depth,
            _ => panic!("AC: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for AC {
    fn reset(&mut self) {
        self.controller_number = 0x11;
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
        if !matches!(addr, 0..=6|0x59..=0x66) {
            return Err(err);
        }

        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=6|0x59..=0x66) {
            return Err(err);
        }

        self[addr as usize] = value;

        Ok(vec![RAMCallbackEffects::NoEffect])
    }
}
