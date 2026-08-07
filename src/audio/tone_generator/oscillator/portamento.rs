use std::time::Duration;

use crate::audio::interface::Audio;
use crate::midi::effect_params::parameter_table::XG_PORTAMENTO_TIME;

use super::super::interface::ToneGeneratorInterface;

#[derive(Debug)]
pub struct Portamento {
    pub source_note: f32,
    pub target_note: f32,
    // from XG_PORTAMENTO_TIME table.
    pub portamento_time: f32,
    /// 1/portamento_time — precomputed so the per-frame tick only multiplies
    inv_time: f32,
    /// Elapsed glide time (accumulated; per-sample `elapsed` ticks are tiny)
    elapsed: f32,
}

impl ToneGeneratorInterface for Portamento {
    fn reset(&mut self) {
        *self = Self::new()
    }

    fn kill(&mut self) {}

    fn release(&mut self) {}
}

impl Portamento {
    pub fn new() -> Self {
        Self {
            source_note: -1.0,
            target_note: -1.0,
            portamento_time: XG_PORTAMENTO_TIME[0],
            inv_time: 1.0 / XG_PORTAMENTO_TIME[0],
            elapsed: 0.0,
        }
    }

    /// Called at note-on: no glide unless a source is set (portamento CC)
    pub fn begin(&mut self, source: f32, target: f32, time: f32) {
        self.source_note = source;
        self.target_note = target;
        self.portamento_time = time;
        self.inv_time = if time > 0.0 { 1.0 / time } else { 0.0 };
        self.elapsed = 0.0;
    }
}

impl Audio for Portamento {
    // output in cents, as delta
    fn tick(&mut self, elapsed: Duration) -> f32 {
        if self.portamento_time <= 0.0 {
            return 0.0;
        }
        self.elapsed += elapsed.as_secs_f32();
        if self.elapsed >= self.portamento_time {
            return 0.0;
        }
        (self.source_note - self.target_note) * (1.0 - self.elapsed * self.inv_time)
    }
}
