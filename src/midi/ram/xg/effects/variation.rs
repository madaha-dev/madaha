use std::fmt::Debug;

use crate::midi::effect_params::default_data::xg_variation_data;
use crate::midi::effect_params::{interface::EffectType, variation_type::XGVariationType};
use crate::midi::errors::MidiError;
use crate::midi::ram::interface::Memory;
use crate::midi::ram::xg::effects::interface::EffectRAM;
use crate::midi::ram::{MemoryAddr, MIDICallbackEffects};
use crate::{get_14bit, get_lsb, get_msb};
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variation {
    // start from 0x40
    /// Variation type MSB
    pub type_msb: u8,
    /// Variation type LSB
    pub type_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param1_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param1_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param2_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param2_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param3_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param3_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param4_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param4_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param5_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param5_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param6_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param6_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param7_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param7_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param8_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param8_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param9_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param9_lsb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param10_msb: u8,
    /// Variation parameter 1-10 MSB/LSB (14-bit)
    pub param10_lsb: u8,
    /// Variation return level
    pub variation_return: u8,
    /// Variation panpot
    pub variation_pan: u8,
    /// Variation send to reverb level
    pub send_to_reverb: u8,
    /// Variation send to chorus level
    pub send_to_chorus: u8,
    // 0=Insertion, 1=System
    /// Connection mode (0=insertion, 1=system)
    pub connection: u8,
    /// Part number for insertion mode
    pub part: u8,
    /// Modulation wheel → variation control depth
    pub mw_variation_control_depth: u8,
    /// Pitch bend → variation control depth
    pub bend_variation_control_depth: u8,
    /// Channel aftertouch → variation control depth
    pub cat_variation_control_depth: u8,
    /// Assignable controller 1 → variation control depth
    pub ac1_variation_control_depth: u8,
    /// Assignable controller 2 → variation control depth
    pub ac2_variation_control_depth: u8,
    /// CBC1 → variation control depth
    pub cbc1_variation_control_depth: u8,
    /// CBC2 → variation control depth
    pub cbc2_variation_control_depth: u8,

    // start from 0x70
    /// Variation parameter 11-16 (varies by variation type)
    pub param11: u8,
    /// Variation parameter 11-16 (varies by variation type)
    pub param12: u8,
    /// Variation parameter 11-16 (varies by variation type)
    pub param13: u8,
    /// Variation parameter 11-16 (varies by variation type)
    pub param14: u8,
    /// Variation parameter 11-16 (varies by variation type)
    pub param15: u8,
    /// Variation parameter 11-16 (varies by variation type)
    pub param16: u8,
}

impl EffectRAM for Variation {
    fn new() -> Self {
        let (msb, lsb) = XGVariationType::DelayLCR.to_tuple();
        let default_data = xg_variation_data::DELAY_LCR;
        Self {
            type_msb: msb,
            type_lsb: lsb,
            param1_msb: get_msb!(default_data[0]),
            param1_lsb: get_lsb!(default_data[0]),
            param2_msb: get_msb!(default_data[1]),
            param2_lsb: get_lsb!(default_data[1]),
            param3_msb: get_msb!(default_data[2]),
            param3_lsb: get_lsb!(default_data[2]),
            param4_msb: get_msb!(default_data[3]),
            param4_lsb: get_lsb!(default_data[3]),
            param5_msb: get_msb!(default_data[4]),
            param5_lsb: get_lsb!(default_data[4]),
            param6_msb: get_msb!(default_data[5]),
            param6_lsb: get_lsb!(default_data[5]),
            param7_msb: get_msb!(default_data[6]),
            param7_lsb: get_lsb!(default_data[6]),
            param8_msb: get_msb!(default_data[7]),
            param8_lsb: get_lsb!(default_data[7]),
            param9_msb: get_msb!(default_data[8]),
            param9_lsb: get_lsb!(default_data[8]),
            param10_msb: get_msb!(default_data[9]),
            param10_lsb: get_lsb!(default_data[9]),
            variation_return: 0x40,
            variation_pan: 0x40,
            send_to_reverb: 0,
            send_to_chorus: 0,
            connection: 0,
            part: 0x7F,
            mw_variation_control_depth: 0x40,
            bend_variation_control_depth: 0x40,
            cat_variation_control_depth: 0x40,
            ac1_variation_control_depth: 0x40,
            ac2_variation_control_depth: 0x40,
            cbc1_variation_control_depth: 0x40,
            cbc2_variation_control_depth: 0x40,
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
        self[0x00] = msb;
        self[0x01] = lsb;
        for i in 0..10 {
            self[0x42 + i * 2] = get_msb!(default_data[i]);
            self[0x43 + i * 2] = get_lsb!(default_data[i]);
        }
        for i in 0..6 {
            self[0x70 + i] = (default_data[10 + i] & 0x7F) as u8;
        }
    }

    fn get_parameter<T>(&mut self, _effect_type: T, param_index: u8) -> Option<u16>
    where
        T: EffectType + Debug + 'static,
    {
        match param_index {
            1..=10 => {
                let i = param_index as usize;
                Some(get_14bit!(self[0x42 + i * 2], self[0x43 + i * 2]))
            }
            11..=16 => Some(self[(param_index as usize) - 11 + 0x70] as u16),
            _ => None,
        }
    }
}

impl Memory for Variation {
    fn reset(&mut self) {
        self.load_parameter(XGVariationType::DelayLCR, xg_variation_data::DELAY_LCR);
        self.variation_return = 0x40;
        self.variation_pan = 0x40;
        self.send_to_reverb = 0;
        self.send_to_chorus = 0;
        self.connection = 0;
        self.part = 0x7F;
        self.mw_variation_control_depth = 0x40;
        self.bend_variation_control_depth = 0x40;
        self.cat_variation_control_depth = 0x40;
        self.ac1_variation_control_depth = 0x40;
        self.ac2_variation_control_depth = 0x40;
        self.cbc1_variation_control_depth = 0x40;
        self.cbc2_variation_control_depth = 0x40;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x20|0x40..=0x62|0x30..=0x35|0x70..=0x75) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x20|0x40..=0x62|0x30..=0x35|0x70..=0x75) {
            return Err(err);
        }
        self[addr as usize] = value;
        Ok(vec![])
    }
}

impl std::ops::Index<usize> for Variation {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            // 0x40-based (absolute address)
            0x40 | 0x00 => &self.type_msb,
            0x41 | 0x01 => &self.type_lsb,
            0x42 | 0x02 => &self.param1_msb,
            0x43 | 0x03 => &self.param1_lsb,
            0x44 | 0x04 => &self.param2_msb,
            0x45 | 0x05 => &self.param2_lsb,
            0x46 | 0x06 => &self.param3_msb,
            0x47 | 0x07 => &self.param3_lsb,
            0x48 | 0x08 => &self.param4_msb,
            0x49 | 0x09 => &self.param4_lsb,
            0x4A | 0x0A => &self.param5_msb,
            0x4B | 0x0B => &self.param5_lsb,
            0x4C | 0x0C => &self.param6_msb,
            0x4D | 0x0D => &self.param6_lsb,
            0x4E | 0x0E => &self.param7_msb,
            0x4F | 0x0F => &self.param7_lsb,
            0x50 | 0x10 => &self.param8_msb,
            0x51 | 0x11 => &self.param8_lsb,
            0x52 | 0x12 => &self.param9_msb,
            0x53 | 0x13 => &self.param9_lsb,
            0x54 | 0x14 => &self.param10_msb,
            0x55 | 0x15 => &self.param10_lsb,
            0x56 | 0x16 => &self.variation_return,
            0x57 | 0x17 => &self.variation_pan,
            0x58 | 0x18 => &self.send_to_reverb,
            0x59 | 0x19 => &self.send_to_chorus,
            0x5A | 0x1A => &self.connection,
            0x5B | 0x1B => &self.part,
            0x5C | 0x1C => &self.mw_variation_control_depth,
            0x5D | 0x1D => &self.bend_variation_control_depth,
            0x5E | 0x1E => &self.cat_variation_control_depth,
            0x5F | 0x1F => &self.ac1_variation_control_depth,
            0x60 | 0x20 => &self.ac2_variation_control_depth,
            0x61 | 0x21 => &self.cbc1_variation_control_depth,
            0x62 | 0x22 => &self.cbc2_variation_control_depth,
            // 0x70-based (absolute address)
            0x70 | 0x30 => &self.param11,
            0x71 | 0x31 => &self.param12,
            0x72 | 0x32 => &self.param13,
            0x73 | 0x33 => &self.param14,
            0x74 | 0x34 => &self.param15,
            0x75 | 0x35 => &self.param16,
            _ => panic!("Variation: index {:#X} out of bounds", index),
        }
    }
}

impl std::ops::IndexMut<usize> for Variation {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            // 0x40-based (absolute address)
            0x40 | 0x00 => &mut self.type_msb,
            0x41 | 0x01 => &mut self.type_lsb,
            0x42 | 0x02 => &mut self.param1_msb,
            0x43 | 0x03 => &mut self.param1_lsb,
            0x44 | 0x04 => &mut self.param2_msb,
            0x45 | 0x05 => &mut self.param2_lsb,
            0x46 | 0x06 => &mut self.param3_msb,
            0x47 | 0x07 => &mut self.param3_lsb,
            0x48 | 0x08 => &mut self.param4_msb,
            0x49 | 0x09 => &mut self.param4_lsb,
            0x4A | 0x0A => &mut self.param5_msb,
            0x4B | 0x0B => &mut self.param5_lsb,
            0x4C | 0x0C => &mut self.param6_msb,
            0x4D | 0x0D => &mut self.param6_lsb,
            0x4E | 0x0E => &mut self.param7_msb,
            0x4F | 0x0F => &mut self.param7_lsb,
            0x50 | 0x10 => &mut self.param8_msb,
            0x51 | 0x11 => &mut self.param8_lsb,
            0x52 | 0x12 => &mut self.param9_msb,
            0x53 | 0x13 => &mut self.param9_lsb,
            0x54 | 0x14 => &mut self.param10_msb,
            0x55 | 0x15 => &mut self.param10_lsb,
            0x56 | 0x16 => &mut self.variation_return,
            0x57 | 0x17 => &mut self.variation_pan,
            0x58 | 0x18 => &mut self.send_to_reverb,
            0x59 | 0x19 => &mut self.send_to_chorus,
            0x5A | 0x1A => &mut self.connection,
            0x5B | 0x1B => &mut self.part,
            0x5C | 0x1C => &mut self.mw_variation_control_depth,
            0x5D | 0x1D => &mut self.bend_variation_control_depth,
            0x5E | 0x1E => &mut self.cat_variation_control_depth,
            0x5F | 0x1F => &mut self.ac1_variation_control_depth,
            0x60 | 0x20 => &mut self.ac2_variation_control_depth,
            0x61 | 0x21 => &mut self.cbc1_variation_control_depth,
            0x62 | 0x22 => &mut self.cbc2_variation_control_depth,
            // 0x70-based (absolute address)
            0x70 | 0x30 => &mut self.param11,
            0x71 | 0x31 => &mut self.param12,
            0x72 | 0x32 => &mut self.param13,
            0x73 | 0x33 => &mut self.param14,
            0x74 | 0x34 => &mut self.param15,
            0x75 | 0x35 => &mut self.param16,
            _ => panic!("Variation: index {:#X} out of bounds", index),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum Connection {
    /// Insertion mode (affects only the assigned part)
    #[default]
    Insertion,
    /// System mode (affects all parts)
    System,
}
