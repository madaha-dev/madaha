use std::time::Duration;

pub trait ToneGeneratorInterface {
    fn reset(&mut self);
    fn step(&mut self, elapsed: Duration) -> f32;

    // Set EG to Release
    fn kill(&mut self);
}
