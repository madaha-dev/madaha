use super::interface::Effect;
use super::super::ram::EffectData;
use crate::{get_msb_u16_u8, merge_data};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum XGReverbType {
    NoEffect = merge_data!(0x0),

    Hall1 = merge_data!(0x1),
    Hall2 = merge_data!(0x1, 0x1),

    Room1 = merge_data!(0x2),
    Room2 = merge_data!(0x2, 0x1),
    Room3 = merge_data!(0x2, 0x2),

    Stage1 = merge_data!(0x3),
    Stage2 = merge_data!(0x3, 0x1),

    Plate = merge_data!(0x4),

    WhiteRoom = merge_data!(0x10),
    Tunnel = merge_data!(0x11),
    Canyon = merge_data!(0x12),
    Basement = merge_data!(0x13),
}

impl Effect for XGReverbType {
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
        data[effect_group][0x00] = get_msb_u16_u8!(effect_type);
        for i in 0..10 {
            data[effect_group][0x02 + i] = default_data[i] as u8;
        }
        for i in 0..6 {
            data[effect_group][0x10 + i] = default_data[0xA + i] as u8;
        }
    }
}
