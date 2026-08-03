use std::time::Duration;

use crate::midi::effect_params::parameter_table::XG_PORTAMENTO_TIME;

use super::super::interface::ToneGeneratorInterface;

#[derive(Debug)]
pub struct Portamento {
    pub source_note: f32,
    pub target_note: f32,
    // from XG_PORTAMENTO_TIME table.
    pub portamento_time: f32,
}

impl ToneGeneratorInterface for Portamento {
    fn reset(&mut self) {
        *self = Self::new()
    }

    fn kill(&mut self) {}

    // output in cents, as delta
    fn step(&mut self, elapsed: Duration) -> f32 {
        let elapsed = elapsed.as_secs_f32();

        if elapsed < self.portamento_time {
            (self.source_note - self.target_note) * (1.0 - elapsed / self.portamento_time)
        } else {
            0.0
        }
    }
}

impl Portamento {
    pub fn new() -> Self {
        Self {
            source_note: -1.0,
            target_note: -1.0,
            portamento_time: XG_PORTAMENTO_TIME[0],
        }
    }
}
