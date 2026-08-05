use super::interface::EffectType;
use crate::merge_data;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
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

    WhiteRoom = merge_data!(0x9),
    Tunnel = merge_data!(0xA),
    Canyon = merge_data!(0xB),
    Basement = merge_data!(0xC),

    // 2006LE extended types (0x0D-0x12): uTG table values = aliases of the classic types
    ReverbExt1 = merge_data!(0xD), // = Room2 params
    ReverbExt2 = merge_data!(0xE), // = Room1 params
    ReverbExt3 = merge_data!(0xF), // = Room3 params
    ReverbExt4 = merge_data!(0x10), // = Hall1 params
    ReverbExt5 = merge_data!(0x11), // = Hall2 params
    ReverbExt6 = merge_data!(0x12), // = Plate params
}

impl EffectType for XGReverbType {
    fn no_effect() -> Self {
        Self::NoEffect
    }
}
