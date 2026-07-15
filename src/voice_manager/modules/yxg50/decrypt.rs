use crate::voice_manager::transform_byte;

const INIT_KEY: u8 = 0x5D;

// TODO: test needed.
pub fn decrypt(data: &mut Vec<u8>) {
    transform_byte(INIT_KEY, data);
}
