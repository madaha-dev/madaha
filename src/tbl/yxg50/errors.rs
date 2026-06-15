use std::{error, fmt};

#[derive(Debug)]
pub enum YXG50Errors {
    NoSuchSample { id: usize },
    LoadBinTBLFailed { reason: String },
    LoadWaveTBLFailed { reason: String },
}

impl error::Error for YXG50Errors {}

impl fmt::Display for YXG50Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchSample { id } => write!(f, "no such samlpe id={}", id),
            Self::LoadBinTBLFailed { reason } => {
                write!(f, "failed to load bin tbl file due to reason={}", reason)
            }
            Self::LoadWaveTBLFailed { reason } => {
                write!(f, "failed to load wave tbl file due to reason={}", reason)
            }
        }
    }
}
