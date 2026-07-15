use std::fmt;

#[derive(Debug)]
pub enum AudioConfigError {
    PolyphonyOutOfRange { max: u16, limit_lower: u16, limit_upper: u16 },
    BadSampleRate { sample_rate: u32 },
    InvalidPolyphony { poly_phony: u16 },
    InvalidDeviceID {dev_id: u8},
}

impl fmt::Display for AudioConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolyphonyOutOfRange { max, limit_lower, limit_upper } => {
                write!(f, "polyphony {} out of limit range [{}, {}]", max, limit_lower, limit_upper)
            }
            Self::BadSampleRate { sample_rate } => {
                write!(
                    f,
                    "bad sample rate {}, available for 22050, 44100, 48000, 96000, 192000",
                    sample_rate
                )
            }
            Self::InvalidPolyphony { poly_phony } => {
                write!(
                    f,
                    "polyphony {} not valid, should be a multiple of 16",
                    poly_phony
                )
            }
            Self::InvalidDeviceID { dev_id } => {
                write!(f, "device ID {} should equal to 16 or bigger ", dev_id)
            }
        }
    }
}

impl std::error::Error for AudioConfigError {}
