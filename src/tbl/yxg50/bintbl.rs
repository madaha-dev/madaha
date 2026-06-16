use std::fs;

use crate::tbl::yxg50::{
    decrypt, drum_note_param::DrumNoteParam, errors::YXG50Errors, sample_meta::SampleMeta,
};

macro_rules! bad_tbl_file_error {
    () => {
        YXG50Errors::LoadBinTBLFailed {
            reason: "incomplete bin tbl file".to_string(),
        }
    };
}

#[derive(Debug)]
pub struct BinTbl {
    /// offset = 0x00000000, length = 0x0010, MU50 ?MB V2.0
    header: Box<[char]>,
    /// offset = 0x00000010, length = 0x000F, SXGWAVE?.TBL, not used for madaha
    wave_table_name: Box<[char]>,
    /// offset = 0x0000001F, length = 0x0001, used by veg.by, decoded=true, encoded=false
    decrypted: bool,
    /// offset = 0x00000020, length = 0x0044, 17 data segs length
    seg_length_table: Box<[usize]>,

    // length depends on seg_length
    pub gs_drum_kit_table: Box<[u8]>, // offset = 0x00000064, length = 0x0080, index to drum_map_table as GS
    pub xg_drum_kit_table: Box<[u8]>, // offset = 0x000000E4, length = 0x0080, index to drum_map_table as XG
    pub xg_sfx_kit_table: Box<[u8]>, // offset = 0x00000164, length = 0x0080, index to drum_map_table as XG SFX
    pub gm2_drum_kit_table: Box<[u8]>, // offset = 0x000001E4, length = 0x0080, index to drum_map_table as GM2
    pub drum_map_table: Box<[Box<[u16]>]>, // offset = 0x00000264, length = 0x1F000, 31 drum kits with 128 note key index to drum_note_param_table
    pub drum_note_param_table: Box<[DrumNoteParam]>, // offset = 0x00002164, length = 0x2490
    pub sfx_index_table: Box<[u16]>,       // offset = 0x000045F4, length = 0x00AE
    pub gs_bank_msb_table: Box<[u8]>, // offset = 0x000046A2, length = 0x0080, GS bank MSB table, value << 7
    pub xg_bank_msb_table: Box<[u8]>, // offset = 0x00004722, length = 0x0080, XG bank MSB table, value << 7
    pub xg_bank_lsb_table: Box<[u8]>, // offset = 0x000047A2, length = 0x0080, XG bank LSB table
    pub seg10: Box<[u8]>,             // offset = 0x00004822, length = 0x0080
    pub seg11: Box<[u16]>,            // offset = 0x000048A2, length = 0x1800
    pub seg12: Box<[u16]>,            // offset = 0x000060A2, length = 0x3800
    pub seg13: Box<[u16]>,            // offset = 0x000098A2, length = 0xFB2A
    pub seg14: Box<[u16]>,            // offset = 0x000193CC, length = 0x5CAE
    pub seg15: Box<[u16]>,            // offset = 0x0001F07A, length = 0x01EC
    pub sample_meta: Box<[SampleMeta]>, // offset = 0x0001F266, length = var

    pub wave_data: Box<[u8]>, // offset = 0x00000020, length = 0x0088, 17 data segs length8], // wave data
}

impl BinTbl {
    pub fn read_bin_tbl(path: &String) -> Result<Self, YXG50Errors> {
        let content = fs::read(path).map_err(|e| YXG50Errors::LoadBinTBLFailed {
            reason: e.to_string(),
        })?;

        let header = content
            .get(0x00..0x10)
            .ok_or(YXG50Errors::LoadBinTBLFailed {
                reason: "bad bin tbl header".to_string(),
            })?
            .iter()
            .map(|&b| b as char)
            .collect();

        let wave_table_name: Box<[char]> = content
            .get(0x10..0x1F)
            .ok_or(bad_tbl_file_error!())?
            .iter()
            .map(|&b| b as char)
            .collect();
        let decrypted = *content.get(0x1F).ok_or(bad_tbl_file_error!())? == 1;

        let seg_length_bytes = content.get(0x20..0x64).ok_or(bad_tbl_file_error!())?;
        let seg_length: Box<[usize]> = seg_length_bytes
            .chunks_exact(4)
            .take(17)
            .map(|chunk| usize::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let mut start_addr: usize = 0x64;

        macro_rules! load_seg {
            ($idx:expr) => {
                Self::load_data_seg(&content, &mut start_addr, seg_length[$idx])?
            };
        }

        let gs_drum_kit_table = load_seg!(0);
        let xg_drum_kit_table = load_seg!(1);
        let xg_sfx_kit_table = load_seg!(2);
        let gm2_drum_kit_table = load_seg!(3);

        let drum_map_table = load_seg!(4)
            .chunks_exact(256)
            .map(|i| {
                i.chunks_exact(2)
                    .take(128)
                    .map(|j| u16::from_le_bytes(j.try_into().unwrap()))
                    .collect()
            })
            .collect();

        let drum_note_param_table = load_seg!(5)
            .chunks_exact(30)
            .map(|c| {
                DrumNoteParam::from_byte(c)
                    .ok_or(bad_tbl_file_error!())
                    .unwrap()
            })
            .collect();

        let sfx_index_table = load_seg!(6)
            .chunks_exact(2)
            .enumerate()
            .map(|(_, chunk)| u16::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let gs_bank_msb_table = load_seg!(7);
        let xg_bank_msb_table = load_seg!(8);
        let xg_bank_lsb_table = load_seg!(9);
        let seg10 = load_seg!(10);
        let seg11 = load_seg!(11)
            .chunks_exact(2)
            .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
            .collect();
        let seg12 = load_seg!(12)
            .chunks_exact(2)
            .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
            .collect();
        let seg13 = load_seg!(13)
            .chunks_exact(2)
            .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
            .collect();
        let seg14 = load_seg!(14)
            .chunks_exact(2)
            .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
            .collect();
        let seg15 = load_seg!(15)
            .chunks_exact(2)
            .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
            .collect();
        let sample_meta: Box<[SampleMeta]> = load_seg!(16)
            .chunks_exact(16)
            .map(|c| {
                SampleMeta::from_bytes(c)
                    .ok_or(bad_tbl_file_error!())
                    .unwrap()
            })
            .collect();

        let wave_table_path: String = wave_table_name.iter().collect();
        Ok(Self {
            header: header,
            wave_table_name,
            decrypted,
            seg_length_table: seg_length,
            gs_drum_kit_table,
            xg_drum_kit_table,
            xg_sfx_kit_table,
            gm2_drum_kit_table,
            drum_map_table,
            drum_note_param_table,
            sfx_index_table,
            gs_bank_msb_table,
            xg_bank_msb_table,
            xg_bank_lsb_table,
            seg10,
            seg11,
            seg12,
            seg13,
            seg14,
            seg15,
            sample_meta,
            wave_data: Self::load_wave_data(&wave_table_path.to_ascii_lowercase(), decrypted)?,
        })
    }

    fn load_data_seg(
        content: &Vec<u8>,
        start_addr: &mut usize,
        length: usize,
    ) -> Result<Box<[u8]>, YXG50Errors> {
        let data = content
            .get(*start_addr..*start_addr + length)
            .ok_or(bad_tbl_file_error!())?;
        *start_addr += length;
        Ok(data.into())
    }

    fn load_wave_data(path: &String, decrypted: bool) -> Result<Box<[u8]>, YXG50Errors> {
        let content = fs::read(path).map_err(|e| YXG50Errors::LoadWaveTBLFailed {
            reason: e.to_string(),
        })?;

        let wave_data = if decrypted {
            content.into_boxed_slice()
        } else {
            decrypt(content.into_boxed_slice())
        };
        Ok(wave_data)
    }

    pub fn get_sample(&self, index: usize) -> Result<&[u8], YXG50Errors> {
        let meta = match self.sample_meta.get(index) {
            Some(meta) => meta,
            None => {
                return Err(YXG50Errors::NoSuchSample { id: index }); // TODO: 404 not found.
            }
        };
        let start_addr = meta.loop_start - meta.start_point_offset;
        let end_addr = meta.loop_start + meta.loop_length + 1;

        Ok(&self.wave_data[start_addr..end_addr])
    }
}
