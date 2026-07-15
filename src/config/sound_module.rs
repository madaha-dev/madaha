use serde::Deserialize;
use strum_macros::EnumString;

use crate::config::{
    SoundModuleType::Auto, interface::ConfigObject, sound_module_errors::SoundModuleError,
};

fn default_sound_module_type() -> SoundModuleType {
    SoundModuleType::Auto
}

#[derive(Debug, Deserialize, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum SoundModuleType {
    Auto,
    #[serde(rename = "s-yxg50", alias = "xg50", alias = "xg100")]
    Syxg50,
    #[serde(rename = "syxg2006le", alias = "xg2006", alias = "xg2006le")]
    Syxg2006le,
    #[serde(rename = "wingroove", alias = "wg")]
    WinGroove,
    NotSupported,
}

#[derive(Debug, Deserialize)]
pub struct SoundModuleConfig {
    /// type for tbl file
    #[serde(default = "default_sound_module_type")]
    pub module_type: SoundModuleType,

    /// bin tbl file, such as `sxgbin21.tbl` `sxgbnw6l.tbl`
    pub tbl_bin_file: String,
    /// wave tbl file
    pub tbl_data_file: String,

    /// wingroove tpd file
    pub wingroove_tpd_file: String,
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
                if self.wingroove_tpd_file.is_empty() {
                    return Err(SoundModuleError::NoWinGrooveTPDFile);
                }
            }
            SoundModuleType::NotSupported => return Err(SoundModuleError::NotSupportedSoundModule),
            Auto => return Ok(()),
        }

        Ok(())
    }
}
