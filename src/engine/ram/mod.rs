use super::errors::MidiError;
use crate::engine::{engine::MidiResetMode, ram::gs::gs_xg_addr_remap};
use std::ops::{Index, IndexMut};
use wd_log::log_warn_ln;

pub mod controller;
mod gs;
pub mod interface;
pub mod types;
mod xg; // for XG // for GS, mapper to XG

pub use types::MemoryAddr;

#[derive(Debug)]
pub struct RAM {
    pub reset_mode: MidiResetMode,
    xg: xg::RAM,
}

impl RAM {
    pub fn new(reset_mode: MidiResetMode, xg_drum_data: &'static Box<[u8]>) -> Self {
        Self {
            reset_mode,
            xg: xg::RAM::new(xg_drum_data),
        }
    }
}

impl interface::Memory for RAM {
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        match self.reset_mode {
            MidiResetMode::XG => self.xg.set(addr, value),
            MidiResetMode::GS => {
                let addr = match gs_xg_addr_remap(addr) {
                    Some(r) => r,
                    None => return Err(err),
                };
                self.xg.set(addr, value)
            }
            _ => Err(err),
        }
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        match self.reset_mode {
            MidiResetMode::XG => self.xg.get(addr),
            MidiResetMode::GS => {
                let addr = match gs_xg_addr_remap(addr) {
                    Some(r) => r,
                    None => return Err(err),
                };
                self.xg.get(addr)
            }
            _ => Err(err),
        }
    }

    fn reset(&mut self) {
        match self.reset_mode {
            MidiResetMode::XG => self.xg.reset(),
            _ => {
                self.xg.reset();
                // TODO: GS Parameter reset.
            }
        }
    }
}

impl Index<usize> for RAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        log_warn_ln!("Use index() for RAM not recommended, or cause panic");
        match self.reset_mode {
            MidiResetMode::XG => &self.xg[index],
            MidiResetMode::GS => &self.xg[index],
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log_warn_ln!("Use index_mut() for RAM not recommended, or cause panic");
        match self.reset_mode {
            MidiResetMode::XG => &mut self.xg[index],
            MidiResetMode::GS => &mut self.xg[index],
            _ => panic!("RAM: index {} out of bounds", index),
        }
    }
}
