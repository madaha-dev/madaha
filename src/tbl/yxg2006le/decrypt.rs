use crate::tbl::transform_byte;

const INIT_KEY: u8 = 0x5B;

pub fn decrypt(data: &mut Box<[u8]>) {
    transform_byte(INIT_KEY, data);
    for chunk in data.chunks_exact_mut(2) {
        chunk.swap(0, 1);
    }
}
