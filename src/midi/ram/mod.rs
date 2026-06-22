use super::errors::MidiError;
use crate::midi::engine::MidiResetMode;

pub mod interface;
mod roland; // for GS
pub mod types;
mod yamaha; // for XG

pub use types::{EffectData, MemoryAddr};

#[derive(Debug)]
pub struct RAM {
    pub reset_mode: MidiResetMode,
    xg: yamaha::RAM,
    gs: roland::RAM,
}

impl RAM {
    pub fn new(reset_mode: MidiResetMode) -> Self {
        Self {
            reset_mode,
            xg: yamaha::RAM::new(),
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
