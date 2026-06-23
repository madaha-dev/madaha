use crate::engine::effects::default_data::xg_reverb_data;
use crate::engine::effects::interface::EffectType;
use crate::engine::effects::reverb_type::XGReverbType;
use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use crate::engine::ram::yamaha::effects::interface::EffectRAM;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reverb {
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
    pub reverb_return: u8,
    pub reverb_pan: u8,
    pub _reserved1: u8,
    pub _reserved2: u8,
    pub param11: u8,
    pub param12: u8,
    pub param13: u8,
    pub param14: u8,
    pub param15: u8,
    pub param16: u8,
}

impl EffectRAM for Reverb {
    fn new() -> Self {
        let (msb, lsb) = XGReverbType::Hall1.to_tuple();
        let default_data = xg_reverb_data::HALL1;
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
            reverb_return: 0x40,
            reverb_pan: 0x40,
            _reserved1: 0xFF,
            _reserved2: 0xFF,
            param11: default_data[10] as u8,
            param12: default_data[11] as u8,
            param13: default_data[12] as u8,
            param14: default_data[13] as u8,
            param15: default_data[14] as u8,
            param16: default_data[15] as u8,
        }
    }
    fn load_parameter<T: EffectType>(&mut self, effect_type: T, default_data: [u16; 16]) {
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

impl Memory for Reverb {
    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        // Should check bondary, or panic
        if !matches!(addr, 0x00..=0x15) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x15) {
            return Err(err);
        }

        Ok(self[addr as usize] = value)
    }

    fn reset(&mut self) {
        self.load_parameter(XGReverbType::Hall1, xg_reverb_data::HALL1);
        self.reverb_return = 0x40;
        self.reverb_pan = 0x40;
    }
}

impl std::ops::Index<usize> for Reverb {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.type_msb,
            1 => &self.type_lsb,
            2 => &self.param1,
            3 => &self.param2,
            4 => &self.param3,
            5 => &self.param4,
            6 => &self.param5,
            7 => &self.param6,
            8 => &self.param7,
            9 => &self.param8,
            10 => &self.param9,
            11 => &self.param10,
            12 => &self.reverb_return,
            13 => &self.reverb_pan,
            14 => &self._reserved1,
            15 => &self._reserved2,
            16 => &self.param11,
            17 => &self.param12,
            18 => &self.param13,
            19 => &self.param14,
            20 => &self.param15,
            21 => &self.param16,
            _ => panic!("Reverb: index {} out of bounds", index),
        }
    }
}

impl std::ops::IndexMut<usize> for Reverb {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.type_msb,
            1 => &mut self.type_lsb,
            2 => &mut self.param1,
            3 => &mut self.param2,
            4 => &mut self.param3,
            5 => &mut self.param4,
            6 => &mut self.param5,
            7 => &mut self.param6,
            8 => &mut self.param7,
            9 => &mut self.param8,
            10 => &mut self.param9,
            11 => &mut self.param10,
            12 => &mut self.reverb_return,
            13 => &mut self.reverb_pan,
            14 => &mut self._reserved1,
            15 => &mut self._reserved2,
            16 => &mut self.param11,
            17 => &mut self.param12,
            18 => &mut self.param13,
            19 => &mut self.param14,
            20 => &mut self.param15,
            21 => &mut self.param16,
            _ => panic!("Reverb: index {} out of bounds", index),
        }
    }
}
