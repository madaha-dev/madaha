pub mod chorus;
pub mod reverb;
pub mod variation;

use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use chorus::Chorus;
use reverb::Reverb;
use std::ops::{Index, IndexMut};
use variation::Variation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectData {
    pub reverb: Reverb,
    pub chorus: Chorus,
    pub variation: Variation,
}

impl EffectData {
    pub fn new() -> Self {
        Self {
            reverb: Reverb::new(),
            chorus: Chorus::new(),
            variation: Variation::new(),
        }
    }
}

impl Index<usize> for EffectData {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            1..=4 | 0x30 => &self.reverb[index],
            0x21..=0x25 | 0x38 => &self.chorus[index],
            0x40 | 0x41 => &self.variation[index],

            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for EffectData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            1..=4 | 0x30 => &mut self.reverb[index],
            0x21..=0x25 | 0x38 => &mut self.chorus[index],
            0x40 | 0x41 => &mut self.variation[index],

            _ => panic!("EffectData: index {} out of bounds", index),
        }
    }
}

impl Memory for EffectData {
    fn reset(&mut self) {
        self.reverb.reset();
        self.chorus.reset();
        self.variation.reset();
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 1..=4 | 0x30 | 0x21..=0x25 | 0x38 | 0x40 | 0x41 ) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 1..=4 | 0x30 | 0x21..=0x25 | 0x38 | 0x40 | 0x41 ) {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }
}
