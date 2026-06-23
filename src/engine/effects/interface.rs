pub trait EffectType {
    fn get_type(msb: u8, lsb: u8) -> Self;
    fn to_tuple(&self) -> (u8, u8);
}
