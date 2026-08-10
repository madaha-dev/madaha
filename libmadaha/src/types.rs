use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::yxg50;

#[derive(Debug, Deserialize, EnumString, PartialEq, Clone, Copy, Serialize)]
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
    /// Playback sample rate of the module's waveform engine.
    ///
    /// S-YXG50 stores its PCM content recorded at 22050Hz but plays it back
    /// at 44100Hz 1:1 (a ×2 speedup — the data files are pre-composed so that
    /// baseKey-relative playback comes out at the right pitch without
    /// resampling). The DDS step must therefore use 44100 as the source rate.
    pub fn get_sample_rate(self) -> f32 {
        match self {
            _ => 44100.0,
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
