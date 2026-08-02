use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bend {
    pub pitch_control: u8,
    pub filter_control: u8,
    pub amplitude_control: u8,
    pub lfo_pmod_depth: u8,
    pub lfo_fmod_depth: u8,
    pub lfo_amod_depth: u8,
}

impl Bend {
    pub const fn new() -> Self {
        Self {
            pitch_control: 0x42,
            filter_control: 0x40,
            amplitude_control: 0x40,
            lfo_pmod_depth: 0x40,
            lfo_fmod_depth: 0x40,
            lfo_amod_depth: 0x40,
        }
    }
}

impl Index<usize> for Bend {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 0x23 => &self.pitch_control,
            1 | 0x24 => &self.filter_control,
            2 | 0x25 => &self.amplitude_control,
            3 | 0x26 => &self.lfo_pmod_depth,
            4 | 0x27 => &self.lfo_fmod_depth,
            5 | 0x28 => &self.lfo_amod_depth,

            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for Bend {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 0x23 => &mut self.pitch_control,
            1 | 0x24 => &mut self.filter_control,
            2 | 0x25 => &mut self.amplitude_control,
            3 | 0x26 => &mut self.lfo_pmod_depth,
            4 | 0x27 => &mut self.lfo_fmod_depth,
            5 | 0x28 => &mut self.lfo_amod_depth,

            _ => panic!("Bend: index {} out of bounds", index),
        }
    }
}

impl Memory for Bend {
    fn reset(&mut self) {
        self.pitch_control = 0x42;
        self.filter_control = 0x40;
        self.amplitude_control = 0x40;
        self.lfo_pmod_depth = 0x40;
        self.lfo_fmod_depth = 0x40;
        self.lfo_amod_depth = 0x40;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=5|0x23..=0x28) {
            return Err(err);
        }

        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=5|0x23..=0x28) {
            return Err(err);
        }

        self[addr as usize] = value;

        Ok(vec![])
    }
}
