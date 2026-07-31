use super::super::errors::MidiError;
use super::MemoryAddr;
use super::RAMCallbackEffects;
use std::ops::{Index, IndexMut};

pub trait Memory: Index<usize> + IndexMut<usize> {
    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError>;
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError>;
    fn reset(&mut self);
    fn hook_check(&self, _addr: MemoryAddr, _value: u8) -> bool {
        true
    }
    fn hook_pre_exec(&self, _addr: MemoryAddr) -> Vec<RAMCallbackEffects> {
        vec![RAMCallbackEffects::NoEffect]
    }
    fn hook_post_exec(&self, _addr: MemoryAddr) -> Vec<RAMCallbackEffects> {
        vec![RAMCallbackEffects::NoEffect]
    }
}
