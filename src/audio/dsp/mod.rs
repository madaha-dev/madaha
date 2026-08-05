/// Effect DSP common infrastructure
pub mod chorus_effect;
pub mod core;
pub mod distortion_effects;
pub mod dynamics;
pub mod eq_effects;
pub mod misc_effects;
pub mod modulation_effects;
pub mod multi_eq;
pub mod params;
pub mod reverb_effect;
pub mod variation_effect;
pub mod wah_effects;
pub mod harmony_effect;
pub mod xg20_effects;

pub use chorus_effect::build_chorus;
pub use distortion_effects::DistortionEffect;
pub use dynamics::{CompressorEffect, NoiseGateEffect};
pub use eq_effects::{ThreeBandEqEffect, TwoBandEqEffect};
pub use misc_effects::{KaraokeEffect, PitchChangeEffect, VoiceCancelEffect};
pub use modulation_effects::{ModEffectKind, build_modulation};
pub use multi_eq::MultiEqDsp;
pub use reverb_effect::{ReverbEffect, build_reverb};
pub use variation_effect::build_variation;

/// Effect processor interface
///
/// Input and output are both stereo (L, R). `sample_rate` is passed in at construction time.
pub trait EffectProcessor {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32);

    /// Real-time parameter modulation from a controller source
    /// (`source`: 0=MW, 1=Bend, 2=CAT, 3=PAT, 4=AC1, 5=AC2, 6=CBC1, 7=CBC2;
    /// `value`: normalized -1..1). The `ins/variation control depth` is
    /// already folded into `value` by the caller. Default: no modulation.
    fn modulate(&mut self, _source: u8, _value: f32) {}

    /// Active notes feeding a harmony/vocoder effect (XG2.0 Harmony family).
    /// Collected by the render loop from the active voices; default: no-op.
    fn set_active_notes(&mut self, _notes: &[u8]) {}
}

/// Thru effect (Thru / NoEffect)
#[derive(Debug, Default)]
pub struct Thru;

impl EffectProcessor for Thru {
    #[inline]
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        input
    }
}
