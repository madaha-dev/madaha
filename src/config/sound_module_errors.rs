use std::fmt;

#[derive(Debug)]
pub enum SoundModuleError {
    NoBinTBLFile,
    NoDataTBLFile,
    NoWinGrooveTPDFile,
    NotSupportedSoundModule,
}

impl std::error::Error for SoundModuleError {}

impl fmt::Display for SoundModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBinTBLFile => write!(
                f,
                "bin tbl path is empty, such as sxgbin21.tbl/sxgbin41.tbl/sxgbnw6l.tbl"
            ),
            Self::NoDataTBLFile => write!(
                f,
                "data tbl path is empty, such as sxgwave21.tbl/sxgwave41.tbl/sxgdat6l.tbl"
            ),
            Self::NoWinGrooveTPDFile => {
                write!(f, "wingroove tpd file path is empty, such as wingroove.tpd")
            }
            Self::NotSupportedSoundModule => {
                write!(f, "not supported sound module")
            }
        }
    }
}
