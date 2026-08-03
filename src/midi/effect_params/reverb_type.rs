use super::interface::EffectType;
use crate::{get_lsb_u16_u8, get_msb_u16_u8, merge_data};
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

impl EffectType for XGReverbType {
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

    fn to_tuple(&self) -> (u8, u8) {
        let msb = get_msb_u16_u8!(*self);
        let lsb = get_lsb_u16_u8!(*self);
        (msb, lsb)
    }
}
