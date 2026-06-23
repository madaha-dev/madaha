use super::errors::MidiError;
use crate::engine::engine::MidiResetMode;
use std::ops::{Index, IndexMut};
use wd_log::log_warn_ln;

pub mod interface;
mod roland; // for GS
pub mod types;
mod yamaha; // for XG

pub use types::MemoryAddr;

#[derive(Debug)]
pub struct RAM {
    pub reset_mode: MidiResetMode,
    xg: yamaha::RAM,
    gs: roland::RAM,
}

impl RAM {
    pub fn new(reset_mode: MidiResetMode, xg_drum_data: &'static Box<[u8]>) -> Self {
        Self {
            reset_mode,
            xg: yamaha::RAM::new(xg_drum_data),
            gs: roland::RAM::new(),
        }
    }
}

impl interface::Memory for RAM {
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        match self.reset_mode {
            MidiResetMode::XG => self.xg.set(addr, value)?,
            _ => (),
        }
        Ok(())
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        match self.reset_mode {
            MidiResetMode::XG => Ok(self.xg.get(addr)?),
            _ => Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
    }

    fn reset(&mut self) {
        match self.reset_mode {
            MidiResetMode::XG => self.xg.reset(),
            _ => (),
        }
    }
}

impl Index<usize> for RAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        log_warn_ln!("Use index() for RAM not recommended, or cause panic");
        match self.reset_mode {
            MidiResetMode::XG => &self.xg[index],
            MidiResetMode::GS => &self.gs[index],
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log_warn_ln!("Use index_mut() for RAM not recommended, or cause panic");
        match self.reset_mode {
            MidiResetMode::XG => &mut self.xg[index],
            MidiResetMode::GS => &mut self.gs[index],
            _ => panic!("RAM: index {} out of bounds", index),
        }
    }
}
