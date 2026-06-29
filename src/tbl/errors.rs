use std::{error, fmt};

#[derive(Debug)]
pub enum TBLErrors {
    NoSuchSample { id: usize },
    LoadBinTBLFailed { reason: String },
    LoadWaveTBLFailed { reason: String },
    InvalidSampleMeta { reason: String },
}

impl error::Error for TBLErrors {}

impl fmt::Display for TBLErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchSample { id } => write!(f, "no such samlpe id={}", id),
            Self::LoadBinTBLFailed { reason } => {
                write!(f, "failed to load bin tbl file due to reason={}", reason)
            }
            Self::LoadWaveTBLFailed { reason } => {
                write!(f, "failed to load wave tbl file due to reason={}", reason)
            }
            Self::InvalidSampleMeta { reason } => {
                write!(f, "invalid sample meta reason={}", reason)
            }
        }
    }
}
