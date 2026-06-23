use crate::engine::effects::chorus_type::XGChorusType;
use crate::engine::effects::default_data::xg_chorus_data;
use crate::engine::effects::interface::EffectType;
use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use crate::engine::ram::yamaha::effects::interface::EffectRAM;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chorus {
    pub type_msb: u8,
    pub type_lsb: u8,
    pub param1: u8,
    pub param2: u8,
    pub param3: u8,
    pub param4: u8,
    pub param5: u8,
    pub param6: u8,
    pub param7: u8,
    pub param8: u8,
    pub param9: u8,
    pub param10: u8,
    pub chorus_return: u8,
    pub chorus_pan: u8,
    pub send_to_reverb: u8,
    pub _reserved2: u8,
    pub param11: u8,
    pub param12: u8,
    pub param13: u8,
    pub param14: u8,
    pub param15: u8,
    pub param16: u8,
}

impl EffectRAM for Chorus {
    fn new() -> Self {
        let (msb, lsb) = XGChorusType::Chorus1.to_tuple();
        let default_data = xg_chorus_data::CHORUS1;
        Self {
            type_msb: msb,
            type_lsb: lsb,
            param1: default_data[0] as u8,
            param2: default_data[1] as u8,
            param3: default_data[2] as u8,
            param4: default_data[3] as u8,
            param5: default_data[4] as u8,
            param6: default_data[5] as u8,
            param7: default_data[6] as u8,
            param8: default_data[7] as u8,
            param9: default_data[8] as u8,
            param10: default_data[9] as u8,
            chorus_return: 0x40,
            chorus_pan: 0x40,
            send_to_reverb: 0x00,
            _reserved2: 0xFF,
            param11: default_data[10] as u8,
            param12: default_data[11] as u8,
            param13: default_data[12] as u8,
            param14: default_data[13] as u8,
            param15: default_data[14] as u8,
            param16: default_data[15] as u8,
        }
    }

    fn load_parameter<T>(&mut self, effect_type: T, default_data: [u16; 16])
    where
        T: EffectType,
    {
        let (msb, lsb) = effect_type.to_tuple();
        self.type_msb = msb;
        self.type_lsb = lsb;
        for i in 0..10 {
            self[0x02 + i] = default_data[i] as u8;
        }
        for i in 0..6 {
            self[0x10 + i] = default_data[0xA + i] as u8;
        }
    }
}

impl Memory for Chorus {
    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        // Should check bondary, or panic
        if !matches!(addr, 0x00..=0x15| 0x20..=0x35) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x15| 0x20..=0x35) {
            return Err(err);
        }

        Ok(self[addr as usize] = value)
    }

    fn reset(&mut self) {
        self.load_parameter(XGChorusType::Chorus1, xg_chorus_data::CHORUS1);
        self.chorus_return = 0x40;
        self.chorus_pan = 0x40;
        self.send_to_reverb = 0x00;
    }
}

impl std::ops::Index<usize> for Chorus {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 0x20 => &self.type_msb,
            1 | 0x21 => &self.type_lsb,
            2 | 0x22 => &self.param1,
            3 | 0x23 => &self.param2,
            4 | 0x24 => &self.param3,
            5 | 0x25 => &self.param4,
            6 | 0x26 => &self.param5,
            7 | 0x27 => &self.param6,
            8 | 0x28 => &self.param7,
            9 | 0x29 => &self.param8,
            10 | 0x2A => &self.param9,
            11 | 0x2B => &self.param10,
            12 | 0x2C => &self.chorus_return,
            13 | 0x2D => &self.chorus_pan,
            14 | 0x2E => &self.send_to_reverb,
            15 | 0x2F => &self._reserved2,
            16 | 0x30 => &self.param11,
            17 | 0x31 => &self.param12,
            18 | 0x32 => &self.param13,
            19 | 0x33 => &self.param14,
            20 | 0x34 => &self.param15,
            21 | 0x35 => &self.param16,
            _ => panic!("Chorus: index {} out of bounds", index),
        }
    }
}

impl std::ops::IndexMut<usize> for Chorus {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 0x20 => &mut self.type_msb,
            1 | 0x21 => &mut self.type_lsb,
            2 | 0x22 => &mut self.param1,
            3 | 0x23 => &mut self.param2,
            4 | 0x24 => &mut self.param3,
            5 | 0x25 => &mut self.param4,
            6 | 0x26 => &mut self.param5,
            7 | 0x27 => &mut self.param6,
            8 | 0x28 => &mut self.param7,
            9 | 0x29 => &mut self.param8,
            10 | 0x2A => &mut self.param9,
            11 | 0x2B => &mut self.param10,
            12 | 0x2C => &mut self.chorus_return,
            13 | 0x2D => &mut self.chorus_pan,
            14 | 0x2E => &mut self.send_to_reverb,
            15 | 0x2F => &mut self._reserved2,
            16 | 0x30 => &mut self.param11,
            17 | 0x31 => &mut self.param12,
            18 | 0x32 => &mut self.param13,
            19 | 0x33 => &mut self.param14,
            20 | 0x34 => &mut self.param15,
            21 | 0x35 => &mut self.param16,
            _ => panic!("Chorus: index {} out of bounds", index),
        }
    }
}
