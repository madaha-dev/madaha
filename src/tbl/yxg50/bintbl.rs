#[derive(Debug)]
pub struct BinTbl {
    pub header: [char; 0x10],                // 0x00000000 MU50 ?MB V2.0
    pub wave_table: [char; 0xF],             // 0x00000010 SXGWAVE?.TBL, not used for madaha
    pub decoded: bool, // 0x0000001F used by veg.by 3, decoded=true, encoded=false
    pub seg_length: [usize; 17], // 0x00000020 17 data segs length
    pub gs_drum_kit_table: [u8; 0x80], // 0x00000064
    pub xg_drum_kit_table: [u8; 0x80], // 0x000000E4
    pub xg_sfx_kit_table: [u8; 0x80], // 0x00000164
    pub gm2_drum_kit_table: [u8; 0x80], // 0x000001E4
    pub drum_map_table: [[u16; 128]; 31], // 0x00000264
    pub drum_default_param: [[u8; 30]; 312], // 0x00002164
    pub sfx_index_table: [u8; 0xAE], // 0x000045F4
    pub seg07: [u8; 0x80], // 0x000046A2
    pub seg08: [u8; 0x80], // 0x00004722
    pub seg09: [u8; 0x80], // 0x000047A2
    pub seg10: [u8; 0x80], // 0x00004822
    pub seg11: [u16; 0x1800 / 2], // 0x000048A2
    pub seg12: [u16; 0x3800 / 2], // 0x000060A2
    pub seg13: [u16; 0xFB2A / 2], // 0x000098A2
    pub seg14: [u16; 0x5CAE / 2], // 0x000193CC
    pub seg15: [u16; 0x01EC / 2], // 0x0001F07A
    pub seg16: Box<[u8]>, // 0x0x0001F266

    pub wave_data: Box<[u8]>, // wave data
}
