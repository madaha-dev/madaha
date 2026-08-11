use std::ops::{Index, IndexMut};

use wd_log::log_warn_ln;

use crate::midi::MIDICallbackEffects;
use crate::midi::errors::MidiError;
use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;

// If the plugin specified by Address Mid has been assigned to the part specified by Address Low,
// the Part Note On messages designated by the data are ignored.
// (To do this, the part's RcvNote Messages must be turned off)
#[derive(Debug, Clone)]
pub struct PluginNoteFilter([[usize; 0x40]; 0x80]);

impl Index<usize> for PluginNoteFilter {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        log_warn_ln!("never use usize for PluginNoteFilter");
        let l = index & 0x3F;
        let m = (index >> 7) & 0x7F;
        &self.0[m][l]
    }
}

impl Index<MemoryAddr> for PluginNoteFilter {
    type Output = usize;
    fn index(&self, index: MemoryAddr) -> &Self::Output {
        &self.0[index[1] as usize][index[2] as usize]
    }
}

impl IndexMut<usize> for PluginNoteFilter {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log_warn_ln!("never use usize for PluginNoteFilter");
        let l = index & 0x3F;
        let m = (index >> 7) & 0x7F;
        &mut self.0[m][l]
    }
}

impl IndexMut<MemoryAddr> for PluginNoteFilter {
    fn index_mut(&mut self, index: MemoryAddr) -> &mut Self::Output {
        &mut self.0[index[1] as usize][index[2] as usize]
    }
}

impl Memory for PluginNoteFilter {
    fn reset(&mut self) {
        *self = Self::new()
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let l = addr[2];
        if l > 0x3F {
            Err(MidiError::BadMemoryAddress { bytes: addr.into() })
        } else {
            Ok(self[addr] as u8)
        }
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        if self.hook_check(addr, value) {
            self[addr] = value as usize;
        }

        Ok(vec![])
    }

    fn hook_check(&self, _addr: MemoryAddr, _value: u8) -> bool {
        matches!(_addr[2], 0x00..0x40) && self[_addr] != _value as usize
    }
}

impl PluginNoteFilter {
    pub fn new() -> Self {
        Self([[0x7F; 0x40]; 0x80])
    }
}
