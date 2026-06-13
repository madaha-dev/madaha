use crate::{impl_xg_effect_type, merge_data};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum XGDistortion {
    Thru = merge_data!(0x40),

    Distortion = merge_data!(0x49),
    Overdrive = merge_data!(0x4A),

    ThreeBandEQ = merge_data!(0x4C),
}

impl_xg_effect_type!(XGDistortion, Thru);
