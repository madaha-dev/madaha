use std::fmt;

#[derive(Debug, Clone)]
pub enum MidiError {
    UnknownByteStream { bytes: Box<[u8]> },
    EventParseError { event_id: u8 },
    IncompletMessage { bytes: Box<[u8]> },
    BadMemoryAddress {bytes: Box<[u8]>}
}

impl std::error::Error for MidiError {}

impl fmt::Display for MidiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventParseError { event_id } => {
                write!(f, "unknown event id: {}", *event_id)
            }
            Self::UnknownByteStream { bytes } => {
                write!(f, "unknown byte stream: {:?}", bytes)
            }
            Self::IncompletMessage { bytes } => {
                write!(f, "incomplet midi message: {:?}", bytes)
            }
            Self::BadMemoryAddress { bytes } => {
                write!(f, "bad memory address [{:?}]", bytes)
            }
        }
    }
}
