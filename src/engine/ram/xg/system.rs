use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct System {
    /// Master tuning nibble 1 (MSB of 16-bit value, see `get_master_tune`/`set_master_tune`)
    pub master_tune1: u8,
    /// Master tuning nibble 2
    pub master_tune2: u8,
    /// Master tuning nibble 3
    pub master_tune3: u8,
    /// Master tuning nibble 4 (LSB of 16-bit value)
    pub master_tune4: u8,
    /// Master volume (0–127)
    pub master_volume: u8,
    /// Reserved byte (unused)
    pub _reserved1: u8,
    /// Master transpose (−24~+24 semitones, 0x40 = center)
    pub transpose: u8,
    /// Drum setup reset flag (SysEx address 0x7D)
    pub drum_setup_reset: u8,
    /// XG system on flag (SysEx address 0x7E)
    pub xg_system_on: u8,
    /// All parameter reset flag (SysEx address 0x7F)
    pub all_parameter_reset: u8,
}

impl System {
    pub fn new() -> Self {
        Self {
            master_tune1: 0,
            master_tune2: 4,
            master_tune3: 0,
            master_tune4: 0,
            master_volume: 0x7F,
            transpose: 0x40,
            _reserved1: 0,
            drum_setup_reset: 0,
            xg_system_on: 0,
            all_parameter_reset: 0,
        }
    }

    pub fn get_master_tune(&self) -> u16 {
        self.master_tune4 as u16 & 0xF
            | (self.master_tune3 as u16 & 0xF) << 4
            | (self.master_tune2 as u16 & 0xF) << 8
            | (self.master_tune1 as u16 & 0xF) << 12
    }

    pub fn set_master_tune(&mut self, value: u16) {
        self.master_tune4 = (value & 0xF) as u8;
        self.master_tune3 = ((value >> 4) & 0xF) as u8;
        self.master_tune2 = ((value >> 8) & 0xF) as u8;
        self.master_tune1 = ((value >> 12) & 0xF) as u8;
    }
}

impl Index<usize> for System {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.master_tune1,
            1 => &self.master_tune2,
            2 => &self.master_tune3,
            3 => &self.master_tune4,
            4 => &self.master_volume,
            5 => &self._reserved1,
            6 => &self.transpose,

            0x7D => &self.drum_setup_reset,
            0x7E => &self.xg_system_on,
            0x7F => &self.all_parameter_reset,

            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for System {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.master_tune1,
            1 => &mut self.master_tune2,
            2 => &mut self.master_tune3,
            3 => &mut self.master_tune4,
            4 => &mut self.master_volume,
            5 => &mut self._reserved1,
            6 => &mut self.transpose,

            0x7D => &mut self.drum_setup_reset,
            0x7E => &mut self.xg_system_on,
            0x7F => &mut self.all_parameter_reset,

            _ => panic!("System: index {} out of bounds", index),
        }
    }
}

impl Memory for System {
    fn reset(&mut self) {
        self.master_tune1 = 0;
        self.master_tune2 = 4;
        self.master_tune3 = 0;
        self.master_tune4 = 0;
        self.master_volume = 0x7F;
        self.transpose = 0x40;
        self._reserved1 = 0;
        self.drum_setup_reset = 0;
        self.xg_system_on = 0;
        self.all_parameter_reset = 0;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=6|0x7D..=0x7F) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0..=6|0x7D..=0x7F) {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }
}
