use std::time::Duration;

/// Unified tick interface for all sound-producing units (called per block/sample)
pub trait Audio {
    fn tick(&mut self, _elapsed: Duration) -> f32 {
        0.0
    }
}
