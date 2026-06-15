// TODO: test needed.
pub fn decrypt(data: Box<[u8]>) -> Box<[u8]> {
    let mut key: u8 = 0x5D;
    let mut result = Vec::with_capacity(data.len());

    for (i, &b) in data.iter().enumerate() {
        let middle_value = (i & 0xFFFF) as u16;
        let decoded = (middle_value ^ key as u16 ^ b as u16) << 4;
        let byte = ((decoded | (decoded >> 8)) ^ 0x5C) as u8;
        result.push(byte);
        key = !key;
    }

    result.into_boxed_slice()
}
