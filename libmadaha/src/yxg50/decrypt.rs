use super::super::descramble_wave;

const INIT_KEY: u8 = 0x5D;

// TODO: test needed.
pub fn decrypt(data: &mut Vec<u8>) {
    descramble_wave(INIT_KEY, data);
}
