use super::DEFAULT_DATA_SEG;

pub fn calc_checksum(data: &Box<[u8]>) -> u8 {
    let data_seg = data.get(2..data.len() - 1).unwrap_or(DEFAULT_DATA_SEG);
    let sum: u32 = data_seg.iter().map(|&b| b as u32).sum();
    ((!sum + 1) & 0x7F) as u8
}