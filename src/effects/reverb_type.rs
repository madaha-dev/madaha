use num_enum::{IntoPrimitive, TryFromPrimitive};
use crate::{impl_xg_effect_type, merge_data};


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

impl_xg_effect_type!(XGReverbType, NoEffect);