use crate::tbl::yxg2006le::drum_setup::DrumSetupEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TBLStruct {
    pub header: [char; 24],
    pub offset_table: [u32; 10],

    /// Points to seg1, `[msb][lsb]`
    pub bank_ptrs: [[u8; 128]; 128],
    /// Points to seg2, `[bank-id][prog]`
    pub prog_ptrs: [[u16; 128]; 87],
    /// seg_02
    pub seg_02: SegWithIndex,
    /// seg_03:
    pub seg_03: SegWithIndex,
    /// seg_04:
    pub seg_04: [[u8; 128]; 4],
    /// seg_05:
    pub seg_05: Box<[u8]>,

    pub seg_06: Box<[u8]>,
    /// drum_setup
    pub seg_07: [DrumSetupEntry; 1422],
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegWithIndex {
    pub index: Box<[u32]>,
    pub data: Box<[u8]>,
}
