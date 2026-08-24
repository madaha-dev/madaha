use std::fs;
use std::sync::Arc;

use super::DrumSetupEntry;
use crate::config::SoundModuleConfig;

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
    debug_mode: bool,
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
                debug_mode: false,
            }), // FUTURE: more format.
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug_mode = debug
    }

    pub fn get_program(
        &self,
        bank_msb: u8,
        bank_lsb: u8,
        program: u8,
    ) -> Option<std::sync::Arc<Program>> {
        let prg = self.instruments[(bank_msb & 0x7F) as usize][(bank_lsb & 0x7F) as usize]
            [(program & 0x7F) as usize]
            .clone();

        if self.debug_mode
            && let Some(pg) = prg.as_ref()
        {
            let data = pg.dump_sample(0x69);
            let bytes: Vec<u8> = data.iter().flat_map(|f| f.to_le_bytes()).collect();
            let path = format!("/tmp/madaha_prog_dump_0x69_{bank_msb}_{bank_lsb}_{program}.dmp");
            if fs::write(&path, bytes).is_ok() {
                log_debug_ln!("dumpfile write into {}", path);
            }
        }

        prg
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
    use std::fs;

    use libmadaha::SoundModuleType;

    const MSB: u8 = 0;
    const LSB: u8 = 0;
    const PRG: u8 = 0;
    const NOTE: usize = 60; // C3

    let config = SoundModuleConfig {
        module_type: SoundModuleType::Syxg50,
        tbl_bin_file: "/home/user/Projects/yxg50/VST/Yamaha/sxgbin41.tbl".to_string(),
        tbl_data_file: "/home/user/Projects/yxg50/VST/Yamaha/Sxgwave4.tbl".to_string(),
    };

    let vm = VoiceManager::load_tbl(&config).unwrap();

    let pg = vm.get_program(MSB, LSB, PRG).unwrap();

    let key = pg.as_ref()[NOTE].as_ref().unwrap();

    let (_, _, sample) = key.layers[0].unwrap();

    let pcm = sample.pcm.as_ref().unwrap();
    let pcm: Box<[u8]> = pcm.iter().map(|p| p.to_le_bytes()).flatten().collect();

    fs::write("/tmp/madaha_voice_manager_piano_c3_60.dmp".to_string(), pcm).unwrap();
}

/// 鼓音色 PCM 应随 SampleMeta 一起加载（历史 bug：`From<&YXG50DrumSetupEntry>`
/// 把 `pcm` 置 None，鼓键无声）。
#[test]
fn voice_manager_get_drum_sample() {
    use libmadaha::SoundModuleType;

    // XG Standard Kit（bank 127, program 0），kick = note 35
    const MSB: u8 = 0x7F;
    const PRG: u8 = 0;
    const NOTE: usize = 35;

    let config = SoundModuleConfig {
        module_type: SoundModuleType::Syxg50,
        tbl_bin_file: "/home/user/Projects/yxg50/VST/Yamaha/sxgbin41.tbl".to_string(),
        tbl_data_file: "/home/user/Projects/yxg50/VST/Yamaha/Sxgwave4.tbl".to_string(),
    };

    let vm = VoiceManager::load_tbl(&config).unwrap();

    let pg = vm.get_program(MSB, 0, PRG).unwrap();

    let key = pg.as_ref()[NOTE].as_ref().expect("kick drum key missing");
    let (_, _, sample) = key.layers[0].expect("kick drum layer missing");

    let pcm = sample
        .pcm
        .as_ref()
        .expect("drum PCM must be loaded, not None");
    assert!(!pcm.is_empty(), "drum PCM must be non-empty");

    // PCM 长度必须等于 loop_point + loop_length（切片语义）
    let expected = sample.get_length();
    assert_eq!(pcm.len(), expected, "drum PCM length mismatch");
}

/// SFX 音色（走 prevoice 波形路径）应加载 PCM，并带 drum_setup 参数（Phase 2）。
#[test]
fn voice_manager_get_sfx_sample() {
    use libmadaha::SoundModuleType;

    let config = SoundModuleConfig {
        module_type: SoundModuleType::Syxg50,
        tbl_bin_file: "/home/user/Projects/yxg50/VST/Yamaha/sxgbin41.tbl".to_string(),
        tbl_data_file: "/home/user/Projects/yxg50/VST/Yamaha/Sxgwave4.tbl".to_string(),
    };

    let vm = VoiceManager::load_tbl(&config).unwrap();

    // XG SFX Kit 1 = bank 0x7E, program 0
    let pg = vm
        .get_program(SFX_BANK_MSB_XG as u8, 0, 0)
        .expect("SFX Kit 1 program missing");

    let mut sfx_found = 0usize;
    for i in 0..128 {
        let Some(key) = pg[i].as_ref() else {
            continue;
        };
        // SFX 键（sfx_instruments 覆盖写入）带 drum_setup；旋律键（melody_instruments）不带
        if key.drum_setup.is_none() {
            continue;
        }
        let (_, _, sample) = key.layers[0].expect("SFX layer missing");
        let pcm = sample
            .pcm
            .as_ref()
            .expect("SFX PCM must be loaded, not None");
        assert!(!pcm.is_empty(), "SFX PCM must be non-empty");
        sfx_found += 1;
    }
    assert!(sfx_found > 0, "SFX Kit 1 must contain at least one SFX key");
}
