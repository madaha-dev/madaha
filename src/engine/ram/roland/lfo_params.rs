
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LFOParams {
    pub rate: u8,
    pub pitch_depth: u8,
    pub tvf_depth: u8,
    pub tva_depth: u8,
}

impl LFOParams {
    pub const fn new() -> Self {
        Self {
            rate: 0x40,
            pitch_depth: 0,
            tvf_depth: 0,
            tva_depth: 0,
        }
    }
}
