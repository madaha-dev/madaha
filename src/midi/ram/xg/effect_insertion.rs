use crate::midi::effect_params::default_data::xg_variation_data;
use crate::midi::effect_params::interface::EffectType;
use crate::midi::effect_params::variation_type::XGVariationType;
use crate::midi::ram::interface::Memory;
use crate::midi::ram::{MemoryAddr, RAMCallbackEffects};
use crate::midi::{errors::MidiError, ram::xg::effects::interface::EffectRAM};
use crate::{get_14bit, get_lsb, get_msb};
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

/*
If effect type does not require MSB, accept parameters with addresses 02 to 0B, and ignore parameters with addresses from 30 to 42.
If effect type requires MSB, accept parameters with addresses 30 to 42 and ignore parameters with addresses 02 to 0B.
Bulk transmissions that include effect-type information will always send parameters at addresses 02 to 0B, but
 if the effect type requires the MSB, the bulk receiving side shall ignore parameters at addresses 02 to 0B.
At present, the folloiwng four effect types require MSBs.
Delay L,C,R、 Delay L,R、 Echo、 Cross Delay
*Data range varies according to effect-type value.
 */
/// Insertion effect parameters (hi addr 0x03).
///
/// Per-part insertion effect configuration including type selection,
/// up to 16 effect parameters (single- or double-byte depending on effect
/// type), part assignment, and controller control depths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectInsertion {
    // hi addr 0x03
    // mid addr for channel (0x00-0x0F)

    // lo addr 0x00
    /// Insertion effect type MSB
    pub ins_effect_type_msb: u8,
    /// Insertion effect type LSB
    pub ins_effect_type_lsb: u8,
    /// Insertion effect parameter 1 (single-byte)
    pub ins_effect_param1: u8,
    /// Insertion effect parameter 2 (single-byte)
    pub ins_effect_param2: u8,
    /// Insertion effect parameter 3 (single-byte)
    pub ins_effect_param3: u8,
    /// Insertion effect parameter 4 (single-byte)
    pub ins_effect_param4: u8,
    /// Insertion effect parameter 5 (single-byte)
    pub ins_effect_param5: u8,
    /// Insertion effect parameter 6 (single-byte)
    pub ins_effect_param6: u8,
    /// Insertion effect parameter 7 (single-byte)
    pub ins_effect_param7: u8,
    /// Insertion effect parameter 8 (single-byte)
    pub ins_effect_param8: u8,
    /// Insertion effect parameter 9 (single-byte)
    pub ins_effect_param9: u8,
    /// Insertion effect parameter 10 (single-byte)
    pub ins_effect_param10: u8,
    /// Insertion effect part assignment (0x7F = OFF)
    pub ins_effect_part: u8,
    /// Modulation wheel → insertion effect control depth
    pub mw_ins_control_depth: u8,
    /// Pitch bend → insertion effect control depth
    pub bend_ins_control_depth: u8,
    /// Channel aftertouch → insertion effect control depth
    pub cat_ins_control_depth: u8,
    /// Assignable controller 1 → insertion effect control depth
    pub ac1_ins_control_depth: u8,
    /// Assignable controller 2 → insertion effect control depth
    pub ac2_ins_control_depth: u8,
    /// Control bank 1 → insertion effect control depth
    pub cbc1_ins_control_depth: u8,
    /// Control bank 2 → insertion effect control depth
    pub cbc2_ins_control_depth: u8,

    // lo addr 0x20
    /// Insertion effect parameter 11 (single-byte)
    pub ins_effect_param11: u8,
    /// Insertion effect parameter 12 (single-byte)
    pub ins_effect_param12: u8,
    /// Insertion effect parameter 13 (single-byte)
    pub ins_effect_param13: u8,
    /// Insertion effect parameter 14 (single-byte)
    pub ins_effect_param14: u8,
    /// Insertion effect parameter 15 (single-byte)
    pub ins_effect_param15: u8,
    /// Insertion effect parameter 16 (single-byte)
    pub ins_effect_param16: u8,

    // lo addr 0x30, MSB/LSB pairs for params that require MSB
    /// Insertion effect parameter 1 MSB
    pub ins_effect_param1_msb: u8,
    /// Insertion effect parameter 1 LSB
    pub ins_effect_param1_lsb: u8,
    /// Insertion effect parameter 2 MSB
    pub ins_effect_param2_msb: u8,
    /// Insertion effect parameter 2 LSB
    pub ins_effect_param2_lsb: u8,
    /// Insertion effect parameter 3 MSB
    pub ins_effect_param3_msb: u8,
    /// Insertion effect parameter 3 LSB
    pub ins_effect_param3_lsb: u8,
    /// Insertion effect parameter 4 MSB
    pub ins_effect_param4_msb: u8,
    /// Insertion effect parameter 4 LSB
    pub ins_effect_param4_lsb: u8,
    /// Insertion effect parameter 5 MSB
    pub ins_effect_param5_msb: u8,
    /// Insertion effect parameter 5 LSB
    pub ins_effect_param5_lsb: u8,
    /// Insertion effect parameter 6 MSB
    pub ins_effect_param6_msb: u8,
    /// Insertion effect parameter 6 LSB
    pub ins_effect_param6_lsb: u8,
    /// Insertion effect parameter 7 MSB
    pub ins_effect_param7_msb: u8,
    /// Insertion effect parameter 7 LSB
    pub ins_effect_param7_lsb: u8,
    /// Insertion effect parameter 8 MSB
    pub ins_effect_param8_msb: u8,
    /// Insertion effect parameter 8 LSB
    pub ins_effect_param8_lsb: u8,
    /// Insertion effect parameter 9 MSB
    pub ins_effect_param9_msb: u8,
    /// Insertion effect parameter 9 LSB
    pub ins_effect_param9_lsb: u8,
    /// Insertion effect parameter 10 MSB
    pub ins_effect_param10_msb: u8,
    /// Insertion effect parameter 10 LSB
    pub ins_effect_param10_lsb: u8,
}

impl EffectRAM for EffectInsertion {
    fn new() -> Self {
        let mut data = Self {
            ins_effect_type_msb: 0,
            ins_effect_type_lsb: 0,
            ins_effect_param1: 0,
            ins_effect_param2: 0,
            ins_effect_param3: 0,
            ins_effect_param4: 0,
            ins_effect_param5: 0,
            ins_effect_param6: 0,
            ins_effect_param7: 0,
            ins_effect_param8: 0,
            ins_effect_param9: 0,
            ins_effect_param10: 0,
            ins_effect_part: 0x7F, // OFF
            mw_ins_control_depth: 0x40,
            bend_ins_control_depth: 0x40,
            cat_ins_control_depth: 0x40,
            ac1_ins_control_depth: 0x40,
            ac2_ins_control_depth: 0x40,
            cbc1_ins_control_depth: 0x40,
            cbc2_ins_control_depth: 0x40,
            ins_effect_param11: 0,
            ins_effect_param12: 0,
            ins_effect_param13: 0,
            ins_effect_param14: 0,
            ins_effect_param15: 0,
            ins_effect_param16: 0,
            ins_effect_param1_msb: 0,
            ins_effect_param1_lsb: 0,
            ins_effect_param2_msb: 0,
            ins_effect_param2_lsb: 0,
            ins_effect_param3_msb: 0,
            ins_effect_param3_lsb: 0,
            ins_effect_param4_msb: 0,
            ins_effect_param4_lsb: 0,
            ins_effect_param5_msb: 0,
            ins_effect_param5_lsb: 0,
            ins_effect_param6_msb: 0,
            ins_effect_param6_lsb: 0,
            ins_effect_param7_msb: 0,
            ins_effect_param7_lsb: 0,
            ins_effect_param8_msb: 0,
            ins_effect_param8_lsb: 0,
            ins_effect_param9_msb: 0,
            ins_effect_param9_lsb: 0,
            ins_effect_param10_msb: 0,
            ins_effect_param10_lsb: 0,
        };
        data.load_parameter(XGVariationType::Distortion, xg_variation_data::DISTORTION);
        data
    }
    fn load_parameter<T>(&mut self, effect_type: T, default_data: [u16; 16])
    where
        T: EffectType + 'static,
    {
        let (msb, lsb) = effect_type.to_tuple();
        self[0x00] = msb;
        self[0x01] = lsb;

        match msb {
            // double byte parameter (MSB/LSB) at 0x30-0x43
            0x5..=0x8 => {
                for i in 0..10 {
                    self[0x30 + i * 2] = get_msb!(default_data[i]);
                    self[0x31 + i * 2] = get_lsb!(default_data[i]);
                }
            }
            // single byte parameter at 0x02-0x0B
            _ => {
                for i in 0..10 {
                    self[0x02 + i] = default_data[i] as u8;
                }
            }
        }

        // param11-16 are always single byte at 0x20-0x25
        for i in 0..6 {
            self[0x20 + i] = (default_data[10 + i] & 0x7F) as u8;
        }
    }
    fn get_parameter<T>(&mut self, effect_type: T, param_index: u8) -> Option<u16>
    where
        T: EffectType + Debug + 'static,
    {
        match param_index {
            11..=16 => Some(self[(param_index as usize) - 11 + 0x20] as u16),
            1..=10 => {
                let (msb, _) = effect_type.to_tuple();
                let idx = param_index as usize;
                match msb {
                    0x5..=0x8 => Some(get_14bit!(self[idx * 2 + 0x2E], self[idx * 2 + 0x2F])),
                    _ => Some(self[idx + 0x01] as u16),
                }
            }
            _ => None,
        }
    }
}

impl Index<usize> for EffectInsertion {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            // lo addr 0x00
            0x00 => &self.ins_effect_type_msb,
            0x01 => &self.ins_effect_type_lsb,
            0x02 => &self.ins_effect_param1,
            0x03 => &self.ins_effect_param2,
            0x04 => &self.ins_effect_param3,
            0x05 => &self.ins_effect_param4,
            0x06 => &self.ins_effect_param5,
            0x07 => &self.ins_effect_param6,
            0x08 => &self.ins_effect_param7,
            0x09 => &self.ins_effect_param8,
            0x0A => &self.ins_effect_param9,
            0x0B => &self.ins_effect_param10,
            0x0C => &self.ins_effect_part,
            0x0D => &self.mw_ins_control_depth,
            0x0E => &self.bend_ins_control_depth,
            0x0F => &self.cat_ins_control_depth,
            0x10 => &self.ac1_ins_control_depth,
            0x11 => &self.ac2_ins_control_depth,
            0x12 => &self.cbc1_ins_control_depth,
            0x13 => &self.cbc2_ins_control_depth,
            // lo addr 0x20
            0x20 => &self.ins_effect_param11,
            0x21 => &self.ins_effect_param12,
            0x22 => &self.ins_effect_param13,
            0x23 => &self.ins_effect_param14,
            0x24 => &self.ins_effect_param15,
            0x25 => &self.ins_effect_param16,
            // lo addr 0x30, MSB/LSB pairs
            0x30 => &self.ins_effect_param1_msb,
            0x31 => &self.ins_effect_param1_lsb,
            0x32 => &self.ins_effect_param2_msb,
            0x33 => &self.ins_effect_param2_lsb,
            0x34 => &self.ins_effect_param3_msb,
            0x35 => &self.ins_effect_param3_lsb,
            0x36 => &self.ins_effect_param4_msb,
            0x37 => &self.ins_effect_param4_lsb,
            0x38 => &self.ins_effect_param5_msb,
            0x39 => &self.ins_effect_param5_lsb,
            0x3A => &self.ins_effect_param6_msb,
            0x3B => &self.ins_effect_param6_lsb,
            0x3C => &self.ins_effect_param7_msb,
            0x3D => &self.ins_effect_param7_lsb,
            0x3E => &self.ins_effect_param8_msb,
            0x3F => &self.ins_effect_param8_lsb,
            0x40 => &self.ins_effect_param9_msb,
            0x41 => &self.ins_effect_param9_lsb,
            0x42 => &self.ins_effect_param10_msb,
            0x43 => &self.ins_effect_param10_lsb,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for EffectInsertion {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x00 => &mut self.ins_effect_type_msb,
            0x01 => &mut self.ins_effect_type_lsb,
            0x02 => &mut self.ins_effect_param1,
            0x03 => &mut self.ins_effect_param2,
            0x04 => &mut self.ins_effect_param3,
            0x05 => &mut self.ins_effect_param4,
            0x06 => &mut self.ins_effect_param5,
            0x07 => &mut self.ins_effect_param6,
            0x08 => &mut self.ins_effect_param7,
            0x09 => &mut self.ins_effect_param8,
            0x0A => &mut self.ins_effect_param9,
            0x0B => &mut self.ins_effect_param10,
            0x0C => &mut self.ins_effect_part,
            0x0D => &mut self.mw_ins_control_depth,
            0x0E => &mut self.bend_ins_control_depth,
            0x0F => &mut self.cat_ins_control_depth,
            0x10 => &mut self.ac1_ins_control_depth,
            0x11 => &mut self.ac2_ins_control_depth,
            0x12 => &mut self.cbc1_ins_control_depth,
            0x13 => &mut self.cbc2_ins_control_depth,
            0x20 => &mut self.ins_effect_param11,
            0x21 => &mut self.ins_effect_param12,
            0x22 => &mut self.ins_effect_param13,
            0x23 => &mut self.ins_effect_param14,
            0x24 => &mut self.ins_effect_param15,
            0x25 => &mut self.ins_effect_param16,
            0x30 => &mut self.ins_effect_param1_msb,
            0x31 => &mut self.ins_effect_param1_lsb,
            0x32 => &mut self.ins_effect_param2_msb,
            0x33 => &mut self.ins_effect_param2_lsb,
            0x34 => &mut self.ins_effect_param3_msb,
            0x35 => &mut self.ins_effect_param3_lsb,
            0x36 => &mut self.ins_effect_param4_msb,
            0x37 => &mut self.ins_effect_param4_lsb,
            0x38 => &mut self.ins_effect_param5_msb,
            0x39 => &mut self.ins_effect_param5_lsb,
            0x3A => &mut self.ins_effect_param6_msb,
            0x3B => &mut self.ins_effect_param6_lsb,
            0x3C => &mut self.ins_effect_param7_msb,
            0x3D => &mut self.ins_effect_param7_lsb,
            0x3E => &mut self.ins_effect_param8_msb,
            0x3F => &mut self.ins_effect_param8_lsb,
            0x40 => &mut self.ins_effect_param9_msb,
            0x41 => &mut self.ins_effect_param9_lsb,
            0x42 => &mut self.ins_effect_param10_msb,
            0x43 => &mut self.ins_effect_param10_lsb,
            _ => panic!("Effect2: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for EffectInsertion {
    fn reset(&mut self) {
        *self = EffectInsertion::new();
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00..=0x13 | 0x20..=0x25 | 0x30..=0x43) {
            return Err(err);
        }
        Ok(self[addr])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00..=0x13 | 0x20..=0x25 | 0x30..=0x43) {
            return Err(err);
        }
        self[addr] = value;
        Ok(vec![RAMCallbackEffects::NoEffect])
    }
}
