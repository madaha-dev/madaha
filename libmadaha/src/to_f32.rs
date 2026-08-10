#[allow(unused)]
pub fn u8_to_f32(b: u8) -> f32 {
    (b as f32 - 128.0) / 128.0
}

#[allow(unused)]
pub fn s16_to_f32(b: i16) -> f32 {
    b as f32 / i16::MAX as f32
}
