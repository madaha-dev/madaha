/// Effect DSP common infrastructure
mod chorus_effect;
pub(crate) mod core;
mod distortion_effects;
mod dynamics;
mod eq_effects;
mod misc_effects;
mod modulation_effects;
mod multi_eq;
mod params;
mod reverb_effect;
mod variation_effect;
mod wah_effects;
mod harmony_effect;
mod xg20_effects;

pub use chorus_effect::build_chorus;
pub use multi_eq::MultiEqDsp;
pub use reverb_effect::build_reverb;
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
