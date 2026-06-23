use std::ops::{Index, IndexMut};

use crate::engine::errors::MidiError;
use crate::engine::ram::{MemoryAddr, interface::Memory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrumSetup {
    pub level: u8,
    pub pan: u8,
    pub coarse: u8,
    pub fine: u8,
    pub reverb_send: u8,
    pub chorus_send: u8,
    pub eg_attack: u8,
    pub eg_release: u8,
}

impl DrumSetup {
    pub const fn new() -> Self {
        Self {
            level: 0x40,
            pan: 0x40,
            coarse: 0x40,
            fine: 0x40,
            reverb_send: 0x40,
            chorus_send: 0x40,
            eg_attack: 0x40,
            eg_release: 0x40,
        }
    }

    // setup_id, drum_key, param
    pub fn addr(addr: MemoryAddr) -> (u8, u8, u8) {
        (
            (addr[1] >> 3) & 0x01,
            (addr[1] & 0x07) << 4 | (addr[2] >> 3) & 0x0F,
            addr[2] & 0x07,
        )
    }
}

impl Memory for DrumSetup {
    fn reset(&mut self) {
        self.level = 0x40;
        self.pan = 0x40;
        self.coarse = 0x40;
        self.fine = 0x40;
        self.reverb_send = 0x40;
        self.chorus_send = 0x40;
        self.eg_attack = 0x40;
        self.eg_release = 0x40;
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (_, _, addr) = Self::addr(addr);
        if !matches!(addr, 0..=7) {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (_, _, addr) = Self::addr(addr);
        if !matches!(addr, 0..=7) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }
}

impl Index<usize> for DrumSetup {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.level,
            1 => &self.pan,
            2 => &self.coarse,
            3 => &self.fine,
            4 => &self.reverb_send,
            5 => &self.chorus_send,
            6 => &self.eg_attack,
            7 => &self.eg_release,

            _ => panic!("DrumSetup: index {} out of bounds", index),
        }
    }
}

impl IndexMut<usize> for DrumSetup {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.level,
            1 => &mut self.pan,
            2 => &mut self.coarse,
            3 => &mut self.fine,
            4 => &mut self.reverb_send,
            5 => &mut self.chorus_send,
            6 => &mut self.eg_attack,
            7 => &mut self.eg_release,

            _ => panic!("DrumSetup: index {} out of bounds", index),
        }
    }
}
