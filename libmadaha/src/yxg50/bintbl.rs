use std::fs;

use super::{
    check_header, decrypt, drum_setup::DrumSetupEntry, pre_voice::Element, sample_meta::SampleMeta,
};

use crate::LoadedModule;
use crate::errors::LoadError;
use crate::interface::SoundModule;

macro_rules! bad_tbl_file_error {
    () => {
        LoadError::LoadBinTBLFailed {
            reason: "incomplete bin tbl file".to_string(),
        }
    };
}

#[derive(Debug)]
pub struct BinTbl {
    /// offset = 0x00000000, length = 0x0010, MU50 ?MB V2.0
    //header: Box<[char]>,
    /// offset = 0x00000010, length = 0x000F, SXGWAVE?.TBL, not used for madaha
    //wave_table_name: Box<[char]>,
    /// offset = 0x0000001F, length = 0x0001, used by veg.by, decoded=true, encoded=false
    //decrypted: bool,
    /// offset = 0x00000020, length = 0x0044, 17 data segs length
    //seg_length_table: Box<[usize]>,

    // length depends on seg_length
    pub gs_drum_kit_table: Box<[u8]>, // offset = 0x00000064, length = 0x0080, index to drum_map_table as GS
    pub xg_drum_kit_table: Box<[u8]>, // offset = 0x000000E4, length = 0x0080, index to drum_map_table as XG
    pub xg_sfx_kit_table: Box<[u8]>, // offset = 0x00000164, length = 0x0080, index to drum_map_table as XG SFX
    pub gm2_drum_kit_table: Box<[u8]>, // offset = 0x000001E4, length = 0x0080, index to drum_map_table as GM2
    pub drum_map_table: Box<[Box<[u16]>]>, // offset = 0x00000264, length = 0x1F000, 31 drum kits with 128 note key index to drum_note_param_table
    pub drum_note_param_table: Box<[DrumSetupEntry]>, // offset = 0x00002164, length = 0x2490
    pub sfx_index_table: Box<[u16]>,       // offset = 0x000045F4, length = 0x00AE
    pub gs_bank_msb_table: Box<[u8]>, // offset = 0x000046A2, length = 0x0080, GS bank MSB table, value << 7
    pub xg_bank_msb_table: Box<[u8]>, // offset = 0x00004722, length = 0x0080, XG bank MSB table, value << 7
    pub xg_bank_lsb_table: Box<[u8]>, // offset = 0x000047A2, length = 0x0080, XG bank LSB table
    pub xg_melody_voice_lsb_table: Box<[u8]>, // offset = 0x00004822, length = 0x0080
    pub gs_program_table: Box<[Box<[u16]>]>, // offset = 0x000048A2, length = 0x1800
    pub xg_program_table: Box<[Box<[u16]>]>, // offset = 0x000060A2, length = 0x3800
    pub base_prevoice: Box<[u8]>,     // offset = 0x000098A2, length = 0xFB2A
    pub extend_prevoice: Box<[u8]>,   // offset = 0x000193CC, length = 0x5CAE
    pub sample_meta_offset_table: Box<[u16]>, // offset = 0x0001F07A, length = 0x01EC
    pub sample_meta: Box<[SampleMeta]>, // offset = 0x0001F266, length = var
}

impl SoundModule for BinTbl {
    fn load_data(paramfile: String, wavefile: String) -> Result<LoadedModule, LoadError> {
        let content = fs::read(paramfile).map_err(|e| LoadError::LoadBinTBLFailed {
            reason: e.to_string(),
        })?;

        if let Some(header) = content.get(0x00..0x10) {
            Self::check_header(header)?;
        }

        let decrypted = *content.get(0x1F).ok_or(bad_tbl_file_error!())? == 1;

        let wave_data = Self::load_wave_data(&wavefile, decrypted)?;

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
                    .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
                    .collect()
            })
            .collect();

        let drum_note_param_table = load_seg!(5)
            .chunks_exact(30)
            .map(|c| DrumSetupEntry::from(c))
            .collect();

        let sfx_index_table = load_seg!(6)
            .chunks_exact(2)
            .enumerate()
            .map(|(_, chunk)| u16::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let gs_bank_msb_table = load_seg!(7);
        let xg_bank_msb_table = load_seg!(8);
        let xg_bank_lsb_table = load_seg!(9);
        let xg_melody_voice_lsb_table = load_seg!(10);
        let gs_program_table = load_seg!(11)
            .chunks_exact(256)
            .map(|i| {
                i.chunks_exact(2)
                    .take(128)
                    .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
                    .collect()
            })
            .collect();
        let xg_program_table = load_seg!(12)
            .chunks_exact(256)
            .map(|i| {
                i.chunks_exact(2)
                    .take(128)
                    .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
                    .collect()
            })
            .collect();
        let base_prevoice = load_seg!(13);
        let extend_prevoice = load_seg!(14);
        let sample_meta_offset_table = load_seg!(15)
            .chunks_exact(2)
            .map(|i| u16::from_le_bytes(i.try_into().unwrap()))
            .collect();
        let sample_meta: Box<[SampleMeta]> = load_seg!(16)
            .chunks_exact(16)
            .map(|c| SampleMeta::from(c))
            .collect();

        Ok(LoadedModule::Syxg50(
            Self {
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
                xg_melody_voice_lsb_table,
                gs_program_table,
                xg_program_table,
                base_prevoice,
                extend_prevoice,
                sample_meta_offset_table,
                sample_meta,
            },
            wave_data,
        ))
    }

    fn check_header(header: &[u8]) -> Result<(), LoadError> {
        if !check_header(header) {
            return Err(LoadError::LoadBinTBLFailed {
                reason: "bad bin tbl header".to_string(),
            });
        }

        Ok(())
    }
}

impl BinTbl {
    pub fn get_program_index(&self, msb: u8, lsb: u8, prog: u8) -> usize {
        // Melody Voice
        if msb == 0x79 {
            let selector = self.xg_melody_voice_lsb_table[lsb as usize];
            return self.xg_program_table[selector as usize][prog as usize] as usize;
        }

        // XG first, then GS/GM
        let selector = self.xg_bank_msb_table[msb as usize];
        match selector {
            // XG instrument hit!
            1 => {
                let selector = self.xg_bank_lsb_table[(lsb + 1) as usize];
                return self.xg_program_table[selector as usize][prog as usize] as usize;
            }

            // XG SFX hit!
            0x2D => {
                return self.xg_program_table[selector as usize][prog as usize] as usize;
            }

            // Fallback to GS/GM
            _ => {
                let selector = self.gs_bank_msb_table[msb as usize];
                return self.gs_program_table[selector as usize][prog as usize] as usize;
            }
        }
    }

    pub fn get_drum(&self, index: usize, note: u8) -> Option<&DrumSetupEntry> {
        let index = self.drum_map_table[index][note as usize] as usize;
        if index == 0xFFFF {
            return None;
        }

        self.drum_note_param_table
            .get(index / 0x1E)
    }

    pub fn get_prevoice(&self, index: usize) -> Option<(Element, Option<Element>)> {
        const CHUNK_SIZE: usize = 78;
        let data = if index < 0x8000 {
            self.base_prevoice.get(index * 2..)?
        } else {
            let index = index - 0x8000;
            self.extend_prevoice.get(index * 2..)?
        };

        if data[1] == 0 {
            let data = data.get(2..CHUNK_SIZE + 2)?.as_array()?;
            return Some((Element::from(data), None));
        } else {
            let data: Box<[[u8; CHUNK_SIZE]]> = data
                .get(2..CHUNK_SIZE * (data[1] + 1) as usize + 2)?
                .chunks_exact(CHUNK_SIZE)
                .map(|i| i.try_into().unwrap())
                .collect();

            return Some((Element::from(&data[0]), Some(Element::from(&data[1]))));
        }
    }

    pub fn get_sample_meta(&self, sample_meta_list: &mut Vec<SampleMeta>, offset: usize)  {
        let sample_meta = self.sample_meta[offset as usize];
        sample_meta_list.push(sample_meta);
        if sample_meta.is_last() {
            return;
        }
        self.get_sample_meta(sample_meta_list, offset + 1);
    }

    fn load_data_seg(
        content: &Vec<u8>,
        start_addr: &mut usize,
        length: usize,
    ) -> Result<Box<[u8]>, LoadError> {
        let data = content
            .get(*start_addr..*start_addr + length)
            .ok_or(bad_tbl_file_error!())?;
        *start_addr += length;
        Ok(data.into())
    }

    fn load_wave_data(path: &String, decrypted: bool) -> Result<Box<[u8]>, LoadError> {
        let mut content = fs::read(path).map_err(|e| LoadError::LoadWaveTBLFailed {
            reason: e.to_string(),
        })?;
        if !decrypted {
            decrypt(&mut content);
        }
        Ok(content.into_boxed_slice())
    }
}
