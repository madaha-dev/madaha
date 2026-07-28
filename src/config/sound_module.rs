use serde::Deserialize;

use crate::config::{interface::ConfigObject, sound_module_errors::SoundModuleError};
use libmadaha::SoundModuleType;

fn default_sound_module_type() -> SoundModuleType {
    SoundModuleType::Auto
}

#[derive(Debug, Deserialize, Clone)]
pub struct SoundModuleConfig {
    /// type for tbl file
    #[serde(default = "default_sound_module_type")]
    pub module_type: SoundModuleType,

    /// bin tbl file, such as `sxgbin21.tbl` `sxgbnw6l.tbl`
    /// or wingroove tpd file.
    pub tbl_bin_file: String,
    /// wave tbl file
    pub tbl_data_file: String,
}

impl ConfigObject<SoundModuleError> for SoundModuleConfig {
    fn check(&self) -> Result<(), SoundModuleError> {
        match self.module_type {
            SoundModuleType::Syxg2006le | SoundModuleType::Syxg50 => {
                if self.tbl_bin_file.is_empty() {
                    return Err(SoundModuleError::NoBinTBLFile);
                }

                if self.tbl_data_file.is_empty() {
                    return Err(SoundModuleError::NoDataTBLFile);
                }
            }
            SoundModuleType::WinGroove => {
                if self.tbl_bin_file.is_empty() {
                    return Err(SoundModuleError::NoWinGrooveTPDFile);
                }
            }
            SoundModuleType::NotSupported => return Err(SoundModuleError::NotSupportedSoundModule),
            SoundModuleType::Auto => return Ok(()),
        }

        Ok(())
    }
}
