use super::super::errors::MidiError;
use super::MemoryAddr;

pub trait Memory {
    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError>;
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError>;
    fn reset(&mut self);
}