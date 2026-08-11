use std::ops::{Index, IndexMut};

use wd_log::log_warn_ln;

use crate::midi::MIDICallbackEffects;
use crate::midi::errors::MidiError;
use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::plugin::PluginType;

// This message specifies the part to which the plugin specified by Address Mid, Low is assigned.
#[derive(Debug, Clone)]
pub struct PluginPartAssign([[usize; 0x80]; 0x80]);

impl PluginPartAssign {
    pub fn new() -> Self {
        Self([[0x7F; 0x80]; 0x80])
    }
}

impl Index<usize> for PluginPartAssign {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        log_warn_ln!("never use usize for PluginPartAssign");
        let l = index & 0x7F;
        let m = (index >> 7) & 0x7F;
        &self.0[m][l]
    }
}

impl Index<MemoryAddr> for PluginPartAssign {
    type Output = usize;
    fn index(&self, index: MemoryAddr) -> &Self::Output {
        &self.0[index[1] as usize][index[2] as usize]
    }
}

impl IndexMut<usize> for PluginPartAssign {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log_warn_ln!("never use usize for PluginPartAssign");
        let l = index & 0x7F;
        let m = (index >> 7) & 0x7F;
        &mut self.0[m][l]
    }
}

impl IndexMut<MemoryAddr> for PluginPartAssign {
    fn index_mut(&mut self, index: MemoryAddr) -> &mut Self::Output {
        &mut self.0[index[1] as usize][index[2] as usize]
    }
}

impl Memory for PluginPartAssign {
    fn reset(&mut self) {
        *self = Self::new()
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let l = addr[2];
        if l > 0x7F {
            Err(MidiError::BadMemoryAddress { bytes: addr.into() })
        } else {
            Ok(self[addr] as u8)
        }
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        if self.hook_check(addr, value) {
            let mut value = value;
            let r = self.hook_pre_exec(addr, &mut value);
            self[addr] = value as usize;
            Ok(r)
        } else {
            Ok(vec![])
        }
    }

    fn hook_check(&self, _addr: MemoryAddr, _value: u8) -> bool {
        // NOTE: XG Spec supports 16 parts, here we have 64 parts!
        if _addr[2] > 0x7F || !matches!(_value, 0x00..0x40 | 0x7F) {
            false
        } else {
            true
        }
    }

    fn hook_pre_exec(&self, _addr: MemoryAddr, _value: &mut u8) -> Vec<MIDICallbackEffects> {
        match _value {
            0x00..0x40 => vec![MIDICallbackEffects::SetPluginForPart {
                part_id: *_value as usize,
                plugin: PluginType::from(_addr[1]),
            }],
            0x7F => vec![MIDICallbackEffects::SetPluginForPart {
                part_id: self[_addr],
                plugin: PluginType::OFF,
            }],
            _ => vec![],
        }
    }
}
