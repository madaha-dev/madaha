use crate::audio::interface::Audio;

/// 发声单元生命周期管理。
/// 采样推进统一走 [`Audio::tick`]。
pub trait ToneGeneratorInterface: Audio {
    fn reset(&mut self);

    // Set stage to Release
    fn release(&mut self);

    // Force stop.
    fn kill(&mut self);
}
