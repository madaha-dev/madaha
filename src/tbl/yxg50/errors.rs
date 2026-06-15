use std::fmt;

#[derive(Debug)]
pub enum YXG50Errors {
    NoSuchSample { id: usize },
}

impl std::error::Error for YXG50Errors {}

impl fmt::Display for YXG50Errors{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchSample { id } => write!(f, "no such samlpe id={}", id)
        }
    }
}