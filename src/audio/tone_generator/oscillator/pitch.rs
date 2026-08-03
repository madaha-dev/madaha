use std::time::Duration;

use super::super::interface::ToneGeneratorInterface;

use crate::midi::channel::Channel;
use crate::midi::note::Note;
use crate::midi::ram::xg::multi_part::MultiPart;

#[derive(Debug)]
pub struct Pitch {
    pub note: u8,

    pub note_in_cent: f32,
}

impl Pitch {
    pub fn new() -> Self {
        Self {
            note: 0xFF,
            note_in_cent: -1.0,
        }
    }
}

impl ToneGeneratorInterface for Pitch {
    fn reset(&mut self) {
        *self = Self::new()
    }

    fn kill(&mut self) {}

    fn step(&mut self, _elapsed: Duration) -> f32 {
        self.note_in_cent
    }
}
