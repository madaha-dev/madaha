/// Low-pass filter (DCF, Digital Controlled Filter)
use std::f32::consts::PI;
///
/// Implementation: Chamberlin two-pole state-variable filter, aligned with the
/// S-YXG2006LE reference (`CDCFUnit::Generate`, verified in Ghidra):
///
/// ```text
///   y1 = (x − K·y1 − y2)·f + y1      (band-pass state)
///   y2 = y1·f + y2                    (low-pass state, output)
///   K  = clamp(3 − 2f, …, 2.0)        (frequency-dependent damping)
/// ```
///
/// The damping `K = 3 − 2f` keeps the filter stable without the Q-based
/// resonance peak of a Simper SVF (the old `Q = 0.5 + param/127·9.5` mapping
/// gave Q ≈ 5.3 at the 64 center → ~5× gain → clipping). Resonance is applied
/// as an upper bound on K (`ExchangeResonanceToLinear`: resonance 64 → 4.0,
/// so the automatic damping is left untouched at the center).
///
/// Alignment notes (S-YXG50 data):
/// - `SampleMeta.filter_cutoff` (Element[13], 64 = center) → base cutoff
/// - `SampleMeta.filter_resonance` (Element[14], 64 = center) → K bound
/// - Part 08 pp 18/19 (Filter Cutoff/Resonance relative offset) → note-on snapshot
/// - FEG (Filter EG) + LFO.lpf output → modulates cutoff parameter
pub mod feg;

pub use feg::FEG;

#[derive(Debug)]
pub struct LPF {
    /// Cutoff frequency (Hz)
    pub cutoff: f32,
    /// Resonance parameter (0-127, 64 = center) → K upper bound
    pub resonance: f32,

    // Chamberlin SVF state
    ic1eq: f32, // band-pass state (y1)
    ic2eq: f32, // low-pass state (y2, output)
    // Coefficients (recomputed when cutoff changes)
    f: f32,     // cutoff coefficient ≈ 2·sin(π·fc/fs)
    k_min: f32, // damping K upper bound from resonance
}

impl LPF {
    pub fn new() -> Self {
        Self {
            cutoff: 1000.0,
            resonance: 64.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            f: 0.0,
            k_min: 4.0,
        }
    }

    /// Set parameters and recompute coefficients (call when cutoff/resonance changes)
    pub fn set_params(&mut self, cutoff_hz: f32, resonance: f32, sample_rate: f32) {
        self.cutoff = cutoff_hz.max(1.0);
        self.resonance = resonance;
        let fc = (self.cutoff / sample_rate).min(0.49);
        // Chamberlin: f = 2·sin(π·fc/fs)
        self.f = 2.0 * (PI * fc).sin();
        // Resonance → damping bound (2006LE ExchangeResonanceToLinear):
        //   param 0 → 1.0 (damping forced, no resonance peak)
        //   param 64 → 4.0 (automatic damping untouched)
        //   param 127 → 15.75 (damping free)
        self.k_min = exchange_resonance_to_linear(self.resonance as i16);
    }

    /// Process one sample, return low-pass output
    pub fn tick(&mut self, input: f32) -> f32 {
        // K = min(max(3 − 2f, 2 − f), k_min)  (2006LE Generate clamp order)
        let k_auto = (3.0 - 2.0 * self.f).max(2.0 - self.f);
        let k = k_auto.min(self.k_min).max(0.1);
        self.ic1eq = (input - k * self.ic1eq - self.ic2eq) * self.f + self.ic1eq;
        self.ic2eq = self.ic1eq * self.f + self.ic2eq;
        self.ic2eq
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// 0-127 parameter → cutoff frequency (Hz), logarithmic mapping 100Hz - 12kHz
    pub fn cutoff_param_to_hz(param: u8) -> f32 {
        let t = (param & 0x7F) as f32 / 127.0;
        100.0 * (120.0f32).powf(t)
    }

    /// 0-127 parameter → resonance (0-127, kept for API compatibility; the
    /// actual damping mapping lives in `set_params`)
    pub fn resonance_param_to_q(param: u8) -> f32 {
        (param & 0x7F) as f32
    }
}

/// 2006LE `ExchangeResonanceToLinear`: resonance parameter → damping bound.
/// Fixed-point log→linear map (verified in Ghidra, x86-32-cpu0x3 @ 0005bef2):
///
/// ```text
/// t = 0x20 − param
/// result = (0x40 − (t & 0x1f)) / 2^(((t >> 5) as i8 + 5) & 0x1f)
/// ```
pub(crate) fn exchange_resonance_to_linear(param: i16) -> f32 {
    let t = 0x20i32 - param as i32;
    if param as i32 <= -0xdf {
        return 0.0;
    }
    let man = 0x40 - (t & 0x1f);
    let exp = ((((t >> 5) as i8 as i32) + 5) & 0x1f) as u32;
    man as f32 / (1i32 << exp) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_mapping() {
        assert!((LPF::cutoff_param_to_hz(0) - 100.0).abs() < 0.1);
        assert!((LPF::cutoff_param_to_hz(127) - 12000.0).abs() < 1.0);
        assert!((LPF::cutoff_param_to_hz(64) - 100.0 * 120.0f32.powf(64.0 / 127.0)).abs() < 0.1);
    }

    #[test]
    fn lpf_passes_dc() {
        // DC signal should pass through lossless (cutoff 10kHz)
        let mut lpf = LPF::new();
        lpf.set_params(10000.0, 1.0, 44100.0);
        let mut out = 0.0;
        for _ in 0..100 {
            out = lpf.tick(1.0);
        }
        assert!((out - 1.0).abs() < 1e-3, "DC gain = {out}");
    }

    #[test]
    fn lpf_attenuates_high_freq() {
        // 10kHz square wave through 100Hz low-pass → heavily attenuated
        let mut lpf = LPF::new();
        lpf.set_params(100.0, 0.5, 44100.0);
        let mut out = 0.0;
        for i in 0..4410 {
            let input = if (i / 4) % 2 == 0 { 1.0 } else { -1.0 };
            out = lpf.tick(input);
        }
        assert!(out.abs() < 0.1, "output={out}");
    }
}

/// CutOff cutoff frequency calculation
///
/// Parameter-domain (0-127) addition → logarithmic frequency (multiplicative modulation, natural in audio):
/// ```
/// cutoff_param = base (VCE filter_cutoff)
///              + part_offset (08 pp 18, 64 = center)
///              + FEG.level × feg_depth (Filter EG Depth, 08 pp 71)
///              + LFO.lpf.output (LFO FMOD)
/// cutoff_hz = param_to_hz(clamp(cutoff_param, 0, 127))
/// ```
#[derive(Debug)]
pub struct CutOff {
    /// VCE base cutoff parameter (0-127)
    pub base: f32,
    /// Part 08 pp 18 relative offset (64=0)
    pub part_offset: f32,
    /// FEG modulation depth (08 pp 71, 64=0 → no effect)
    pub feg_depth: f32,
    /// LFO modulation depth on cutoff (0-127, 0=no effect)
    pub lfo_depth: f32,
    /// External modulation (MW/Bend/CAT/PAT filter control), in param units, updated each block
    pub mod_offset: f32,
}

impl CutOff {
    pub fn new() -> Self {
        Self {
            base: 64.0,
            part_offset: 0.0,
            feg_depth: 0.0,
            lfo_depth: 0.0,
            mod_offset: 0.0,
        }
    }

    /// Compute cutoff frequency (Hz), called every block
    pub fn compute_hz(&self, feg_level: f32, lfo_lpf: f32) -> f32 {
        let mut param = self.base + self.part_offset;
        param += feg_level * self.feg_depth;
        param += lfo_lpf * self.lfo_depth;
        param += self.mod_offset;
        LPF::cutoff_param_to_hz(param.round().clamp(0.0, 127.0) as u8)
    }

    /// FEG depth parameter (08 pp 71, 0-127, 64=0) → modulation range (param units)
    pub fn feg_depth_param(param: u8) -> f32 {
        param as f32 - 64.0 // -64..+63 param units
    }

    /// LFO FMOD depth (0-127) → cutoff param modulation range (±40 param ≈ frequency ~8x span)
    pub fn lfo_depth_param(param: u8) -> f32 {
        param as f32 / 127.0 * 40.0
    }
}
