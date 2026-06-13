use std::fmt;

#[derive(Debug)]
pub enum MidiError {
    UnknownByteStream { bytes: Vec<u8> },
    EventParseError { event_id: u8 },
    IncompletMessage { bytes: Vec<u8> },
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
        }
    }
}
