use crate::config::TBLType;
use crate::engine::MidiResetMode;
use crate::engine::voice::voice_manager::VoiceManager;
use std::fs;

pub struct TBLHelper {}

impl TBLHelper {
    fn check_header(header: &[u8]) -> TBLType{
        // TODO check header magic here.
        if yxg50::check_header(&header[0..4]) {
             return TBLType::Syxg50
        }

        if yxg2006le::check_header(header) {
            return TBLType::Syxg2006le
        }

        TBLType::NotSupported
    }

    pub fn load_tbl(bin_tbl: &String, wave_tbl: &String) -> VoiceManager {
        let bin_tbl = fs::read(bin_tbl).unwrap();
        let mut wav_tbl = fs::read(wave_tbl).unwrap();

        let header = &bin_tbl[0..8];

        match Self::check_header(header) {
            TBLType::Syxg50 => {
                if bin_tbl[0x1F] == 0 {
                    yxg50::decrypt(&mut wav_tbl);
                }
            }
            TBLType::Syxg2006le => {
                yxg2006le::decrypt(&mut wav_tbl);
            }
            _ => panic!("bad format"),
        }

        // TODO: load all parameters.
        VoiceManager {
            sample_data: wav_tbl.as_slice(),
        }
    }
}
