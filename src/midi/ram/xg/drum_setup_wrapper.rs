use std::ops::{Index, IndexMut};

use super::super::MemoryAddr;
use super::super::interface::Memory;
use super::drum_setup::DrumSetup;
use crate::midi::errors::MidiError;
use crate::midi::MIDICallbackEffects;
use crate::voice_manager::DrumSetupEntry;

#[derive(Debug, Clone, Copy)]
pub struct DrumSetupWrapper {
    pub program: u8,
    pub drum_setup: [DrumSetup; 79],
}

impl Index<usize> for DrumSetupWrapper {
    type Output = DrumSetup;
    fn index(&self, index: usize) -> &Self::Output {
        &self.drum_setup[index]
    }
}

impl IndexMut<usize> for DrumSetupWrapper {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.drum_setup[index]
    }
}

impl Memory for DrumSetupWrapper {
    fn reset(&mut self) {
        self.drum_setup.iter_mut().for_each(|ds| ds.reset());
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (_, m, _) = addr.split();
        let ds = self.drum_setup.get_mut(m as usize).ok_or(err)?;
        ds.set(addr, value)
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let m = addr[1];
        let ds = self.drum_setup.get(m as usize).ok_or(err)?;
        ds.get(addr)
    }
}

impl DrumSetupWrapper {
    pub fn new(drum_data: [DrumSetupEntry; 79]) -> Self {
        let drum_data = drum_data.map(|d| DrumSetup::from(d));
        Self {
            program: 0,
            drum_setup: drum_data,
        }
    }
}
