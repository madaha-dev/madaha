use crate::config::Config;
use crate::voice_manager::DrumSetupEntry;

use super::bank::Bank;
use super::parser::parse_syxg50;
use super::program::Program;

use libmadaha::LoadError;
use libmadaha::LoadedModule;
use libmadaha::SoundWave;
use libmadaha::load;

pub const DRUM_BANK_MSB_GS: usize = 0x7B; // internal
pub const DRUM_BANK_MSB_GM2: usize = 0x78;
pub const DRUM_BANK_MSB_XG: usize = 0x7F;
pub const SFX_BANK_MSB_XG: usize = 0x7E;

pub type Instruments = [[Bank; 128]; 128];

#[derive(Debug)]
pub struct VoiceManager {
    pub sound_wave: SoundWave,
    // Okay, gs bank and xg bank are not conflict, just one voice table
    // XG Drums, bank msb = 0x7F
    // XG SFX, bank msb = 0x7E
    // GM2 Drums, bank msb = 0x78
    // GM2 Ins, bank msb = 0x79
    // GS Drums, i will set it to 0x7B(internal)
    pub instruments: Instruments,
}

impl VoiceManager {
    pub fn load_tbl(cfg: &Config) -> Result<Self, LoadError> {
        let m = load(
            cfg.sound_module.module_type,
            cfg.sound_module.tbl_bin_file.clone(),
            cfg.sound_module.tbl_data_file.clone(),
        )?;

        match m {
            LoadedModule::Syxg50(p, w) => Ok(Self {
                sound_wave: w,
                instruments: parse_syxg50(&p),
            }), // FUTURE: more format.
        }
    }

    pub fn get_program(&self, bank_msb: u8, bank_lsb: u8, program: u8) -> Program {
        self.instruments[(bank_msb & 0x7F) as usize][(bank_lsb & 0x7F) as usize]
            [(program & 0x7F) as usize]
    }

    pub fn get_drum_setup(&self, bank_msb: u8, program: u8) -> Option<[DrumSetupEntry; 79]> {
        if matches!(
            bank_msb as usize,
            DRUM_BANK_MSB_XG | DRUM_BANK_MSB_GS | DRUM_BANK_MSB_GM2
        ) {
            Some(self.instruments[bank_msb as usize][0][program as usize].to_drum_setup_entry())
        } else {
            None
        }
    }
}
