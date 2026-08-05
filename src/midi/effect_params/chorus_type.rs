use super::interface::EffectType;
use crate::merge_data;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
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

    // 2006LE extended types (uTG table indexes 0x10-0x14, independent params)
    ChorusExt1 = merge_data!(0x10),
    ChorusExt2 = merge_data!(0x11),
    ChorusExt3 = merge_data!(0x12),
    ChorusExt4 = merge_data!(0x13),
    ChorusExt5 = merge_data!(0x14),
}

impl EffectType for XGChorusType {
    fn no_effect() -> Self {
        Self::NoEffect
    }
}
