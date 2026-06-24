use crate::tbl::transform_byte;

const INIT_KEY: u8 = 0x5D;

// TODO: test needed.
pub fn decrypt(data: &mut Box<[u8]>) {
    transform_byte(INIT_KEY, data);
}
