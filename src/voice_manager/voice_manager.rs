use std::sync::Arc;

use crate::config::SoundModuleConfig;
use crate::voice_manager::DrumSetupEntry;

use super::parser::parse_syxg50;
use super::program::Program;

use libmadaha::LoadError;
use libmadaha::LoadedModule;
use libmadaha::load;
use wd_log::log_debug_ln;

pub const DRUM_BANK_MSB_GS: usize = 0x7B; // internal
pub const DRUM_BANK_MSB_GM2: usize = 0x78;
pub const DRUM_BANK_MSB_XG: usize = 0x7F;
pub const SFX_BANK_MSB_XG: usize = 0x7E;

/// Sparse voice table: [msb][lsb][prog] → actual voice (Vec heap allocation + Arc sharing)
/// Slots with the same prevoice index share the same Program (melody parsing memoization)
pub type Instruments = Vec<Vec<Vec<Option<Arc<Program>>>>>;

#[derive(Debug)]
pub struct VoiceManager {
    // Okay, gs bank and xg bank are not conflict, just one voice table
    // XG Drums, bank msb = 0x7F
    // XG SFX, bank msb = 0x7E
    // GM2 Drums, bank msb = 0x78
    // GM2 Ins, bank msb = 0x79
    // GS Drums, i will set it to 0x7B(internal)
    pub instruments: Instruments,
}

impl VoiceManager {
    pub fn load_tbl(cfg: &SoundModuleConfig) -> Result<Self, LoadError> {
        let m = load(
            cfg.module_type,
            cfg.tbl_bin_file.clone(),
            cfg.tbl_data_file.clone(),
        )?;

        match m {
            LoadedModule::Syxg50(p) => Ok(Self {
                instruments: parse_syxg50(&p),
            }), // FUTURE: more format.
        }
    }

    pub fn get_program(
        &self,
        bank_msb: u8,
        bank_lsb: u8,
        program: u8,
    ) -> Option<std::sync::Arc<Program>> {
        self.instruments[(bank_msb & 0x7F) as usize][(bank_lsb & 0x7F) as usize]
            [(program & 0x7F) as usize]
            .clone()
    }

    pub fn get_drum_setup(&self, bank_msb: u8, program: u8) -> Option<[DrumSetupEntry; 79]> {
        log_debug_ln!("drum_setup, bank_msb={}", bank_msb);

        if matches!(
            bank_msb as usize,
            DRUM_BANK_MSB_XG | DRUM_BANK_MSB_GS | DRUM_BANK_MSB_GM2
        ) {
            self.instruments[bank_msb as usize][0][program as usize]
                .as_ref()
                .map(|p| {
                    log_debug_ln!("drum setup got: {:?}", p);
                    p.to_drum_setup_entry()
                })
        } else {
            None
        }
    }
}

#[test]
fn voice_manager_get_piano_sample() {
    const MSB: u8 = 0;
    const LSB: u8 = 0;
    const PRG: u8 = 0;
    const NOTE: usize = 60; // C3

    let config = SoundModuleConfig {
        module_type: libmadaha::SoundModuleType::Syxg50,
        tbl_bin_file: "/home/user/Projects/yxg50/VST/Yamaha/sxgbin41.tbl".to_string(),
        tbl_data_file: "/home/user/Projects/yxg50/VST/Yamaha/Sxgwave4.tbl".to_string(),
    };

    let vm = VoiceManager::load_tbl(&config).unwrap();

    let pg = vm.get_program(MSB, LSB, PRG).unwrap();

    let key = pg.as_ref()[NOTE].as_ref().unwrap();

    let (_, _, sample) = key.layers[0].unwrap();

    let pcm = sample.pcm.as_ref().unwrap();
    let pcm: Box<[u8]> = pcm.iter().map(|p| p.to_le_bytes()).flatten().collect();

    std::fs::write("/tmp/madaha_voice_manager_piano_c3_60.dmp".to_string(), pcm).unwrap();
}
