use std::time::Duration;

use crate::audio::interface::Audio;

use super::super::interface::ToneGeneratorInterface;

use crate::midi::note::Note;

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

    pub fn play(&mut self, p: Note) {
        self.note = p as u8;
    }
}

impl ToneGeneratorInterface for Pitch {
    fn reset(&mut self) {
        *self = Self::new()
    }

    fn kill(&mut self) {}

    fn release(&mut self) {}
}

impl Audio for Pitch {
    fn tick(&mut self, _elapsed: Duration) -> f32 {
        self.note_in_cent
    }
}
