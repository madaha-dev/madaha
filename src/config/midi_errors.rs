use std::fmt;

#[derive(Debug)]
pub enum MidiConfigError {
    BadPolyReplicant {
        value: u16,
    },
    BadScoringConfig {
        reason: &'static str,
    },
    InvalidPolyphony {
        poly_phony: u16,
    },
    InvalidDeviceID {
        dev_id: u8,
    },
    PolyphonyOutOfRange {
        max: u16,
        limit_lower: u16,
        limit_upper: u16,
    },
}

impl fmt::Display for MidiConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadPolyReplicant { value } => write!(
                f,
                "poly replicant should not less than 100, current {}",
                value
            ),
            Self::BadScoringConfig { reason } => write!(f, "bad scoring config, {}", reason),
            Self::PolyphonyOutOfRange {
                max,
                limit_lower,
                limit_upper,
            } => {
                write!(
                    f,
                    "polyphony {} out of limit range [{}, {}]",
                    max, limit_lower, limit_upper
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

impl std::error::Error for MidiConfigError {}
