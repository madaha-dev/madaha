use std::fmt;

#[derive(Debug)]
pub enum AudioConfigError {
    BadSampleRate { sample_rate: u32 },
    BadBufferSize { buffer_size: u32 },
    BadMasterVolume { master_volume: f32 },
}

impl fmt::Display for AudioConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMasterVolume { master_volume } => {
                write!(f, "bad master volume {master_volume}, expected 0.05..=4.0")
            }
            Self::BadSampleRate { sample_rate } => {
                write!(
                    f,
                    "bad sample rate {}, available for 22050, 44100, 48000, 96000, 192000",
                    sample_rate
                )
            }
            Self::BadBufferSize { buffer_size } => {
                write!(
                    f,
                    "bad buffer size {}, should be power of 2, and not less than 64",
                    buffer_size
                )
            }
        }
    }
}

impl std::error::Error for AudioConfigError {}

