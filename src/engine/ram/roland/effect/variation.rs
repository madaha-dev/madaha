use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variation {
    pub output_level: u8,
    pub variation_type: u8,
}

impl Variation {
    pub fn new() -> Self {
        Self {
            output_level: 0,
            variation_type: 0,
        }
    }
}

impl Index<usize> for Variation {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            1 | 0x41 => &self.output_level,
            0 | 0x40 => &self.variation_type,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for Variation {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            1 | 0x41 => &mut self.output_level,
            0 | 0x40 => &mut self.variation_type,
            _ => panic!("Variation: index {} out of bounds", index),
        }
    }
}

impl Memory for Variation {
    fn reset(&mut self) {
        self.output_level = 0;
        self.variation_type = 0;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0 | 1 | 0x40 | 0x41) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0 | 1 | 0x40 | 0x41) {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }
}
