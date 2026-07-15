use crate::config::Config;
use crate::engine::voice::program::Program;
use crate::engine::voice::voice::Voice;
use crate::voice_manager::tbl_helper::TBLHelper;

use super::samples::SampleData;

#[derive(Debug)]
pub struct VoiceManager {
    pub sample_data: SampleData,
    // Okay, gs bank and xg bank are not conflict, just one voice table
    pub voice: Voice,
}

impl VoiceManager {
    pub fn load_tbl(cfg: &Config) -> Self {
        TBLHelper::load_tbl(&cfg.tbl.tbl_bin_file, &cfg.tbl.tbl_data_file)
    }
    pub fn get_program(&self, bank_msb: u8, bank_lsb: u8, program: u8) -> Option<Program> {
        if bank_msb > 0x7F || bank_lsb > 0x7F || program > 0x7F {
            return None;
        }
        let bank = self.voice[bank_msb as usize][bank_lsb as usize]?;
        Some(bank[program as usize])
    }
}
