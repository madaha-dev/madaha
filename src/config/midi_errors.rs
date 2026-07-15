use std::fmt;

#[derive(Debug)]
pub enum MidiConfigError {
    BadPolyReplicant { value: u16 },
    BadScoringConfig { reason: &'static str },
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
        }
    }
}

impl std::error::Error for MidiConfigError {}
