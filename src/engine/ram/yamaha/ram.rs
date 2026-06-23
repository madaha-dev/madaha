use std::ops::{Index, IndexMut};

use wd_log::log_warn_ln;

use super::effects::EffectData;
use crate::engine::{
    consts::DRUM_CHANNEL_ID,
    errors::MidiError,
    ram::{
        MemoryAddr,
        interface::Memory,
        yamaha::{
            display_bitmap::DisplayBitmap, drum_setup::DrumSetup, effects::interface::EffectRAM,
            multi_part::MultiPart, system::System,
        },
    },
};

/// XG hardware memory emulate
/// but we never response bulk dump 23333
#[derive(Debug)]
pub struct RAM {
    pub system: System,                    // SysEx 00 00 ??
    pub effect: [EffectData; 16],          // SysEx 02 ?? ??
    pub display_letter: [u8; 0x20],        // SysEx 06 00 ??, text display
    pub display_bitmap: DisplayBitmap,     // SysEx 07 ?? ??, bitmap display
    pub multi_part: [MultiPart; 16],       // SysEx 08 ?? ??
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
            0x02 => &self.effect[m & 0xF][l],
            0x06 => &self.display_letter[l & 0x1F],
            0x08 => &self.multi_part[m & 0xF][l],
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
            0x02 => &mut self.effect[m & 0xF][l],
            0x06 => &mut self.display_letter[l & 0x1F],
            0x08 => &mut self.multi_part[m & 0xF][l],
            _ => panic!("RAM: index {} out of bounds", index),
        }
    }
}

impl Memory for RAM {
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        match addr[0] {
            0x00 => return self.system.set(addr, value),
            0x02 => return self.set_effect(addr, value),
            0x06 => return self.set_text(addr, value),
            0x07 => return self.display_bitmap.set(addr, value),
            0x08 => return self.set_multipart(addr, value),
            0x30..0x3F => return self.set_drumsetup(addr, value),

            _ => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        match addr[0] {
            0x00 => return self.system.get(addr),
            0x02 => return self.get_effect(addr),
            0x08 => return self.get_multipart(addr),
            0x30..0x3F => return self.get_drumsetup(addr),

            _ => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
    }

    fn reset(&mut self) {
        self.system.reset();
        self.effect = default_effect_ram();
        for i in 0..16 {
            self.multi_part[i].reset();
        }
    }
}

impl RAM {
    pub fn new(drum_data: &'static Box<[u8]>) -> RAM {
        Self {
            system: System::new(),
            effect: [EffectData::new(); 16],
            display_letter: [0; 0x20],
            display_bitmap: DisplayBitmap::new(),
            multi_part: {
                let mut data = [MultiPart::new(0); 16];
                data[DRUM_CHANNEL_ID] = MultiPart::new(DRUM_CHANNEL_ID);
                data
            },
            drum_setup: [[DrumSetup::new(drum_data); 74]; 16],
        }
    }

    fn set_effect(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let addr_m = addr[1] as usize;
        let effect_table = match self.effect.get_mut(addr_m) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
        effect_table.set(addr, value)
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

    fn get_effect(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let addr_m = addr[1] as usize;
        let effect_table = match self.effect.get(addr_m) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };

        effect_table.get(addr)
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
