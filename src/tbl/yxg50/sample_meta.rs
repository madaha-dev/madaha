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
    pub start_point_offset: [u8; 3],

    /// loop end point, big-endian
    /// loop_end = loop_start + loop_length
    pub loop_length: [u8; 3],

    /// base address for sample, but not the start, big-endian
    pub loop_start: [u8; 3],

    /// sample rate for sample, 0x80 = 22050
    pub sample_rate_for_sample: u8,

    /// sample rate for output, 0x00 = 44100
    pub sample_rate_for_output: u8,

    /// key range fot this sample
    pub key_start: u8,
    pub key_end: u8,
}

impl SampleMeta {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }
        Some(Self {
            velocity: data[0],
            base_key: data[1],
            tone: data[2],
            start_point_offset: [data[3], data[4], data[5]],
            loop_length: [data[6], data[7], data[8]],
            loop_start: [data[9], data[10], data[11]],
            sample_rate_for_sample: data[12],
            sample_rate_for_output: data[13],
            key_start: data[14],
            key_end: data[15],
        })
    }
}

#[inline(always)]
pub fn sample_meta_addr(input: [u8; 3]) -> usize {
    (input[0] as usize) << 16 | (input[1] as usize) << 8 | input[2] as usize
}
