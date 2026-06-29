/// Used for encrypted wave tbl file
pub fn transform_byte(start_key: u8, data: &mut Vec<u8>) {
    let mut key = start_key;

    for (i, b) in data.iter_mut().enumerate() {
        let tmp = *b ^ key ^ (i as u8);
        *b = (tmp >> 4) | (tmp << 4);
        key = !key;
    }
}
