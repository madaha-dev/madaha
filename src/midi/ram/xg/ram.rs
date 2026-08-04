use std::{
    ops::{Index, IndexMut},
    sync::{Arc, RwLock},
};

use super::effects::EffectData;
use crate::midi::{
    consts::MAX_PART_SIZE,
    errors::MidiError,
    ram::{
        MemoryAddr, MIDICallbackEffects,
        interface::Memory,
        xg::{
            display_bitmap::DisplayBitmap, drum_setup_wrapper::DrumSetupWrapper,
            effect_insertion::EffectInsertion, effects::interface::EffectRAM, multi_eq::MultiEQ,
            multi_part::MultiPart, multi_part_ext::MultiPartExt, multi_part_vl::MultiPartVL,
            system::System,
        },
    },
};
use crate::voice_manager::DrumSetupEntry;

/// XG hardware memory emulate
/// but we never response bulk dump 23333
#[derive(Debug)]
pub struct RAM {
    pub system: System,                                             // SysEx 00 00 ??
    pub effect1: EffectData,                                        // SysEx 02 01 ??
    pub multi_eq: MultiEQ,                                          // SysEx 02 40 ??
    pub effect_instertion: [EffectInsertion; 0x80],                 // SysEx 03 ?? ??
    pub display_letter: [u8; 0x20],                                 // SysEx 06 00 ??, text display
    pub display_bitmap: DisplayBitmap, // SysEx 07 ?? ??, bitmap display
    pub multi_part: [Arc<RwLock<MultiPart>>; MAX_PART_SIZE], // SysEx 08 ?? ??
    pub multi_part_vl: [MultiPartVL; MAX_PART_SIZE], // SysEx 09 ?? ??
    pub multi_part_ext: [Arc<RwLock<MultiPartExt>>; MAX_PART_SIZE], // SysEx 0A ?? ??
    pub drum_setup: [DrumSetupWrapper; 16], // SysEx 3n ?? ??

                                       // TODO: SysEx 0x70 0x71, for plugins
}

impl Index<usize> for RAM {
    type Output = u8;
    fn index(&self, _index: usize) -> &Self::Output {
        panic!("Use index() for RAM not recommended");
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, _index: usize) -> &mut Self::Output {
        panic!("Use index_mut() for RAM not recommended");
    }
}

impl Memory for RAM {
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (h, m, _) = addr.split();

        match h {
            0x00 => self.system.set(addr, value),
            0x02 => match m {
                0x01 => self.effect1.set(addr, value),
                0x40 => self.multi_eq.set(addr, value),
                _ => return Err(err),
            },
            0x03 => self.effect_instertion[(m & 0x7F) as usize].set(addr, value),
            0x06 => self.set_text(addr, value),
            0x07 => self.display_bitmap.set(addr, value),
            0x08 => self.set_multipart(addr, value),
            0x09 => self.multi_part_vl[(m as usize) & (MAX_PART_SIZE - 1)].set(addr, value),
            0x0A => self.set_multipart(addr, value),
            0x30..0x3F => self.set_drumsetup(addr, value),

            _ => return Err(err),
        }
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (h, m, _) = addr.split();

        match h {
            0x00 => return self.system.get(addr),
            0x02 => match m {
                0x01 => return self.effect1.get(addr),
                0x40 => return self.multi_eq.get(addr),
                _ => return Err(err),
            },
            0x03 => return self.effect_instertion[(m & 0x7F) as usize].get(addr),
            0x08 => return self.get_multipart(addr),
            0x09 => return self.multi_part_vl[(m as usize) & (MAX_PART_SIZE - 1)].get(addr),
            0x0A => return self.get_multipart(addr),
            0x30..0x3F => return self.get_drumsetup(addr),

            _ => return Err(err),
        };
    }

    fn reset(&mut self) {
        self.system.reset();
        self.effect1.reset();
        self.multi_eq.reset();
        self.multi_part
            .iter_mut()
            .for_each(|m| m.write().map(|mut m| m.reset()).unwrap_or(()));
        self.multi_part_vl.iter_mut().for_each(|m| m.reset());
        self.multi_part_ext
            .iter_mut()
            .for_each(|m| m.write().map(|mut m| m.reset()).unwrap_or(()));
        self.display_bitmap.reset();
        self.drum_setup.iter_mut().for_each(|ds| ds.reset());
    }
}

impl RAM {
    pub fn new(drum_data: [DrumSetupEntry; 79]) -> RAM {
        Self {
            system: System::new(),
            effect1: EffectData::new(),
            effect_instertion: [EffectInsertion::new(); 0x80],
            multi_eq: MultiEQ::new(),
            display_letter: [0; 0x20],
            display_bitmap: DisplayBitmap::new(),
            multi_part: {
                let mut data = [MultiPart::new(0); MAX_PART_SIZE];
                data.iter_mut().enumerate().for_each(|(i, d)| {
                    if i < 0x10 {
                        d.rcv_channel = i as u8
                    }
                });
                data.map(|d| Arc::new(RwLock::new(d)))
            },
            multi_part_vl: [MultiPartVL::new(); MAX_PART_SIZE],
            multi_part_ext: [MultiPartExt::new(); MAX_PART_SIZE].map(|d| Arc::new(RwLock::new(d))),
            drum_setup: [DrumSetupWrapper::new(drum_data); 16],
        }
    }

    fn set_text(
        &mut self,
        addr: MemoryAddr,
        value: u8,
    ) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        let addr_l = addr[2] as usize;
        match self.display_letter.get_mut(addr_l) {
            Some(r) => *r = value,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
        Ok(vec![])
    }

    fn set_multipart(
        &mut self,
        addr: MemoryAddr,
        value: u8,
    ) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        let (t, part, _) = addr.split();
        match t {
            // Base
            0x08 => self.multi_part.get(part as usize).map(|r| {
                r.write()
                    .map(|mut r| r.set(addr, value))
                    .unwrap_or(Ok(vec![]))
            }),
            // Extend
            0x0A => self.multi_part_ext.get(part as usize).map(|r| {
                r.write()
                    .map(|mut r| r.set(addr, value))
                    .unwrap_or(Ok(vec![]))
            }),
            _ => None,
        }
        .ok_or(MidiError::BadMemoryAddress { bytes: addr.into() })?
    }

    fn set_drumsetup(
        &mut self,
        addr: MemoryAddr,
        value: u8,
    ) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        let setup = (addr[0] & 0x0F) as usize;
        let note = (addr[1] as usize).wrapping_sub(0x0D);
        if note >= 74 {
            return Err(MidiError::BadMemoryAddress { bytes: addr.into() });
        }

        self.drum_setup[setup][note].set(addr, value)
    }

    fn get_multipart(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let (t, part, _) = addr.split();

        match t {
            // Base
            0x08 => self.multi_part.get(part as usize).map(|r| {
                r.read()
                    .map(|r| r.get(addr))
                    .map_err(|e| MidiError::LockError {
                        reason: e.to_string(),
                    })?
            }),
            // Extend
            0x0A => self.multi_part_ext.get(part as usize).map(|r| {
                r.read()
                    .map(|r| r.get(addr))
                    .map_err(|e| MidiError::LockError {
                        reason: e.to_string(),
                    })?
            }),
            _ => None,
        }
        .ok_or(MidiError::BadMemoryAddress { bytes: addr.into() })?
    }

    fn get_drumsetup(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let setup = (addr[0] & 0x0F) as usize;
        let note = (addr[1] as usize).wrapping_sub(0x0D);
        if note >= 74 {
            return Err(MidiError::BadMemoryAddress { bytes: addr.into() });
        }

        self.drum_setup[setup][note].get(addr)
    }
}

fn default_effect_ram() -> [EffectData; 16] {
    [EffectData::new(); 16]
}
