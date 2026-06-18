pub trait Effect {
    fn get_type(msb: u8, lsb: u8) -> Self;
    fn load_parameter(
        data: &mut [[u8; 128]; 128],
        effect_group: usize,
        effect_type: Self,
        default_data: [u16; 16],
    );
}
