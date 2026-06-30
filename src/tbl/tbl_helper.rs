use super::yxg50;
use super::yxg2006le;
use crate::config::TBLType;
use crate::engine::MidiResetMode;
use crate::engine::voice::voice_manager::VoiceManager;
use std::fs;

pub struct TBLHelper {
    tbl_type: TBLType,
}

impl TBLHelper {
    fn new() -> Self {
        Self {
            tbl_type: TBLType::NotSupported,
        }
    }

    fn check_header(&mut self, header: &[u8]) {
        // TODO check header magic here.
        if yxg50::check_header(&header[0..4]) {
            self.tbl_type = TBLType::Syxg50;
            return;
        }

        if yxg2006le::check_header(header) {
            self.tbl_type = TBLType::Syxg2006le;
            return;
        }
    }

    pub fn load_tbl(bin_tbl: &String, wave_tbl: &String) -> VoiceManager {
        let _helper = Self::new();

        let bin_tbl = fs::read(bin_tbl).unwrap().into_boxed_slice();
        let mut wav_tbl = fs::read(wave_tbl).unwrap();

        let header = &bin_tbl[0..8];
        _helper.check_header(header);

        match _helper.tbl_type {
            TBLType::Syxg50 => {
                yxg50::decrypt(&mut wav_tbl);
            }
            TBLType::Syxg2006le => {
                yxg2006le::decrypt(&mut wav_tbl);
            }
            _ => panic!("bad format"),
        }

        // TODO: load all parameters.
        VoiceManager {
            sample_data: wav_tbl.as_slice(),
            reset_mode: MidiResetMode::GM,
        }
    }
}
