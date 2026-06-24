use std::{
    ops::{Index, IndexMut},
    usize,
};

use wd_log::log_warn_ln;

use super::effects::EffectData;
use crate::engine::{
    consts::DRUM_CHANNEL_ID,
    errors::MidiError,
    ram::{
        MemoryAddr,
        interface::Memory,
        xg::{
            display_bitmap::DisplayBitmap, drum_setup::DrumSetup, effect2::Effect2,
            effects::interface::EffectRAM, multi_eq::MultiEQ, multi_part::MultiPart,
            multi_part_ext::MultiPartExt, multi_part_vl::MultiPartVL, system::System,
        },
    },
};

/// XG hardware memory emulate
/// but we never response bulk dump 23333
#[derive(Debug)]
pub struct RAM {
    pub system: System,                     // SysEx 00 00 ??
    pub effect1: EffectData,                // SysEx 02 01 ??
    pub multi_eq: MultiEQ,                  // SysEx 02 40 ??
    pub effect2: [Effect2; 0x80],           // SysEx 03 ?? ??
    pub display_letter: [u8; 0x20],         // SysEx 06 00 ??, text display
    pub display_bitmap: DisplayBitmap,      // SysEx 07 ?? ??, bitmap display
    pub multi_part: [MultiPart; 16],        // SysEx 08 ?? ??
    pub multi_part_vl: [MultiPartVL; 16],   // SysEx 09 ?? ??
    pub multi_part_ext: [MultiPartExt; 16], // SysEx 0A ?? ??
    pub ad_part: MultiPart,                 // SysEx 10 00 ??

    pub drum_setup: [[DrumSetup; 74]; 16], // SysEx 3n ?? ??
}

impl Index<usize> for RAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        log_warn_ln!("Use index() for RAM not recommended, or cause panic");
        let l = index & 0xFF;
        let m = (index >> 8) & 0xFF;
        let h = (index >> 16) & 0xFF;
        match h {
            0x00 => &self.system[l],
            0x02 => match m {
                0x01 => &self.effect1[l],
                0x40 => &self.multi_eq[l],
                _ => &0xFF,
            },
            0x03 => &self.effect2[m & 0x7F][l],
            0x06 => &self.display_letter[l & 0x1F],
            0x08 => &self.multi_part[m & 0xF][l],
            0x09 => &self.multi_part_vl[m & 0xF][l],
            0x0A => &self.multi_part_ext[m & 0xF][l],
            0x10 => &self.ad_part[l],
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log_warn_ln!("Use index_mut() for RAM not recommended, or cause panic");
        let l = index & 0xFF;
        let m = (index >> 8) & 0xFF;
        let h = (index >> 16) & 0xFF;
        match h {
            0x00 => &mut self.system[l],
            0x02 => match m {
                0x01 => &mut self.effect1[l],
                0x40 => &mut self.multi_eq[l],
                _ => panic!("RAM: index {} out of bounds", index),
            },
            0x03 => &mut self.effect2[m & 0x7F][l],
            0x06 => &mut self.display_letter[l & 0x1F],
            0x08 => &mut self.multi_part[m & 0xF][l],
            0x09 => &mut self.multi_part_vl[m & 0xF][l],
            0x0A => &mut self.multi_part_ext[m & 0xF][l],
            0x10 => &mut self.ad_part[l],
            _ => panic!("RAM: index {} out of bounds", index),
        }
    }
}

impl Memory for RAM {
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (h, m, _) = addr.split();
        match h {
            0x00 => return self.system.set(addr, value),
            0x02 => match m {
                0x01 => return self.effect1.set(addr, value),
                0x40 => return self.multi_eq.set(addr, value),
                _ => return Err(err),
            },
            0x03 => return self.effect2[(m & 0x7F) as usize].set(addr, value),
            0x06 => return self.set_text(addr, value),
            0x07 => return self.display_bitmap.set(addr, value),
            0x08 => return self.set_multipart(addr, value),
            0x09 => return self.multi_part_vl[(m & 0xF) as usize].set(addr, value),
            0x0A => return self.multi_part_ext[(m & 0xF) as usize].set(addr, value),
            0x10 => return self.ad_part.set(addr, value),
            0x30..0x3F => return self.set_drumsetup(addr, value),

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
            0x03 => return self.effect2[(m & 0x7F) as usize].get(addr),
            0x08 => return self.get_multipart(addr),
            0x09 => return self.multi_part_vl[(m & 0xF) as usize].get(addr),
            0x0A => return self.multi_part_ext[(m & 0xF) as usize].get(addr),
            0x10 => return self.ad_part.get(addr),
            0x30..0x3F => return self.get_drumsetup(addr),

            _ => return Err(err),
        };
    }

    fn reset(&mut self) {
        self.system.reset();
        self.effect1.reset();
        self.multi_eq.reset();
        for i in 0..16 {
            self.multi_part[i].reset();
            self.multi_part_vl[i].reset();
            self.multi_part_ext[i].reset();
        }
        self.display_bitmap.reset();
        for i in 0..16 {
            for j in 0..74 {
                self.drum_setup[i][j].reset();
            }
        }
    }
}

impl RAM {
    pub fn new(drum_data: &'static Box<[u8]>) -> RAM {
        Self {
            system: System::new(),
            effect1: EffectData::new(),
            effect2: [Effect2::new(); 0x80],
            multi_eq: MultiEQ::new(),
            display_letter: [0; 0x20],
            display_bitmap: DisplayBitmap::new(),
            multi_part: {
                let mut data = [MultiPart::new(0); 16];
                data[DRUM_CHANNEL_ID] = MultiPart::new(DRUM_CHANNEL_ID);
                data
            },
            multi_part_vl: [MultiPartVL::new(); 16],
            multi_part_ext: [MultiPartExt::new(); 16],
            ad_part: MultiPart::new(0),
            drum_setup: [[DrumSetup::new(drum_data); 74]; 16],
        }
    }

    fn set_text(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let addr_l = addr[2] as usize;
        match self.display_letter.get_mut(addr_l) {
            Some(r) => *r = value,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
        Ok(())
    }

    fn set_multipart(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let channel = addr[1] as usize;
        let parameter_table = match self.multi_part.get_mut(channel) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
        parameter_table.set(addr, value)
    }

    fn set_drumsetup(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let setup = (addr[0] & 0x0F) as usize;
        let note = (addr[1] as usize).wrapping_sub(0x0D);
        if note >= 74 {
            return Err(MidiError::BadMemoryAddress { bytes: addr.into() });
        }

        self.drum_setup[setup][note].set(addr, value)
    }

    fn get_multipart(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let channel = addr[1] as usize;
        let parameter_table = match self.multi_part.get(channel) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };

        parameter_table.get(addr)
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
