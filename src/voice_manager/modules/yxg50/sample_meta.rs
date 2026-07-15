// sample info for tbl dataseg16

#[derive(Debug)]
pub struct SampleMeta {
    /// velocity, 0 is the max
    pub velocity: u8,

    /// sample in note key
    pub base_key: u8,

    /// tone in cents
    pub tone: u8,

    /// negtive offset for base_addr, big-endian
    /// start_addr = loop_start - start_point_offset
    pub start_point_offset: usize,

    /// loop end point, big-endian
    /// loop_end = loop_start + loop_length
    pub loop_length: usize,

    /// base address for sample, but not the start, big-endian
    pub loop_start: usize,

    /// sample rate for sample, 0x80 = 22050
    pub sample_rate_for_sample: u8,

    /// sample rate for output, 0x00 = 44100
    pub _reserved: u8,

    /// key range fot this sample
    pub pitch_fine: u8,
    pub key_end: u8,
}

impl From<&[u8; 16]> for SampleMeta {
    fn from(data: &[u8; 16]) -> Self {
        Self {
            velocity: data[0],
            base_key: data[1],
            tone: data[2],
            start_point_offset: sample_meta_addr([data[3], data[4], data[5]]),
            loop_length: sample_meta_addr([data[6], data[7], data[8]]),
            loop_start: sample_meta_addr([data[9], data[10], data[11]]),
            sample_rate_for_sample: data[12],
            _reserved: data[13],
            pitch_fine: data[14],
            key_end: data[15],
        }
    }
}

impl SampleMeta {
    pub fn check_key(&self, note: u8) -> bool {
        note <= (self.key_end & 0x7F) && self.key_end & 0x80 != 0
    }
}

pub fn sample_meta_addr(input: [u8; 3]) -> usize {
    (input[0] as usize) << 16 | (input[1] as usize) << 8 | input[2] as usize
}
