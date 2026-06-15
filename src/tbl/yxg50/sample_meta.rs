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

#[inline(always)]
pub fn sample_meta_addr(input: [u8; 3]) -> usize {
    (input[0] as usize) << 16 | (input[1] as usize) << 8 | input[2] as usize
}
