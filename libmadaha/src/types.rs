use serde::Deserialize;
use strum_macros::EnumString;

use crate::yxg50;

#[derive(Debug, Deserialize, EnumString, PartialEq, Clone, Copy)]
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

impl SoundModuleType {
    pub fn get_sample_rate(self) -> f32 {
        match self {
            _ => 22050.0,
        }
    }
}

#[derive(Debug)]
pub enum LoadedModule {
    Syxg50(yxg50::bintbl::BinTbl),
    // TODO:
    // SYXG2006LE()
    // WinGroove()
}
