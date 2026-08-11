use crate::audio::interface::Audio;

/// Voice unit lifecycle management.
/// Sample advancement goes through [`Audio::tick`].
pub trait ToneGeneratorInterface: Audio {
    #[allow(dead_code)]
    fn reset(&mut self);

    // Set stage to Release
    fn release(&mut self);

    // Force stop.
    fn kill(&mut self);
}
