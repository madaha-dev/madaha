use super::interface::Effect;
use super::super::ram::EffectData;
use crate::{get_msb_u16_u8, merge_data};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum XGChorusType {
    NoEffect = merge_data!(0x0),

    Chorus1 = merge_data!(0x41),
    Chorus2 = merge_data!(0x41, 0x1),
    Chorus3 = merge_data!(0x41, 0x2),
    Chorus4 = merge_data!(0x41, 0x8),

    Celeste1 = merge_data!(0x42),
    Celeste2 = merge_data!(0x42, 0x1),
    Celeste3 = merge_data!(0x42, 0x2),
    Celeste4 = merge_data!(0x42, 0x8),

    Flanger1 = merge_data!(0x43),
    Flanger2 = merge_data!(0x43, 0x1),
    Flanger3 = merge_data!(0x43, 0x8),

    Symphonic = merge_data!(0x44),
    Phaser = merge_data!(0x48),
}

impl Effect for XGChorusType {
    fn get_type(msb: u8, lsb: u8) -> Self {
        let full = merge_data!(msb as u16, lsb as u16);
        match Self::try_from(full) {
            Ok(r) => r,
            Err(_) => {
                let msb_only = merge_data!(msb as u16);
                Self::try_from(msb_only).unwrap_or(Self::NoEffect)
            }
        }
    }
    fn load_parameter(
        data: &mut EffectData,
        effect_group: usize,
        effect_type: Self,
        default_data: [u16; 16],
    ) {
        data[effect_group][0x20] = get_msb_u16_u8!(effect_type);
        for i in 0..10 {
            data[effect_group][0x22 + i] = default_data[i] as u8;
        }
        for i in 0..6 {
            data[effect_group][0x30 + i] = default_data[0xA + i] as u8;
        }
    }
}
