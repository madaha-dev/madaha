use std::ops::{Index, IndexMut};

use crate::midi::MIDICallbackEffects;

use super::super::errors::MidiError;
use super::MemoryAddr;

pub trait Memory: Index<usize> + IndexMut<usize> {
    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError>;
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<MIDICallbackEffects>, MidiError>;
    fn reset(&mut self);
    fn hook_check(&self, _addr: MemoryAddr, _value: u8) -> bool {
        true
    }
    fn hook_pre_exec(&self, _addr: MemoryAddr, _value: u8) -> Vec<MIDICallbackEffects> {
        vec![]
    }
    fn hook_post_exec(&self, _addr: MemoryAddr) -> Vec<MIDICallbackEffects> {
        vec![]
    }
}
