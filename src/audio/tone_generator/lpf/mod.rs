/// Low-pass filter (DCF, Digital Controlled Filter)
///
/// Implementation: two-pole state-variable filter (SVF, Andrew Simper's stable version)
/// Parameters:
///   cutoff:  cutoff frequency (Hz), derived from VCE base + Part relative offset + FEG modulation
///   resonance: Q value (0.5 - 10)
///
/// Alignment notes (S-YXG50 data):
/// - `SampleMeta.filter_cutoff` (Element[13], 64 = center) → base cutoff
/// - `SampleMeta.filter_resonance` (Element[14], 64 = center) → Q
/// - Part 08 pp 18/19 (Filter Cutoff/Resonance relative offset) → note-on snapshot
/// - FEG (Filter EG) + LFO.lpf output → modulates cutoff parameter
pub mod feg;

pub use feg::FEG;

#[derive(Debug)]
pub struct LPF {
    /// Cutoff frequency (Hz)
    pub cutoff: f32,
    /// Q value
    pub resonance: f32,

    // SVF state
    ic1eq: f32,
    ic2eq: f32,
    // Coefficient cache (recomputed when cutoff/resonance changes)
    a0: f32,
    a1: f32,
    k: f32,
}

impl LPF {
    pub fn new() -> Self {
        Self {
            cutoff: 1000.0,
            resonance: 1.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            a0: 0.0,
            a1: 0.0,
            k: 0.0,
        }
    }

    /// Set parameters and recompute coefficients (call when cutoff/resonance changes)
    pub fn set_params(&mut self, cutoff_hz: f32, q: f32, sample_rate: f32) {
        self.cutoff = cutoff_hz.max(1.0);
        self.resonance = q.max(0.1);
        let w = 2.0 * std::f32::consts::PI * self.cutoff / sample_rate;
        let g = w.tan();
        self.k = 1.0 / self.resonance;
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a0 = g;
    }

    /// Process one sample, return low-pass output
    pub fn tick(&mut self, input: f32) -> f32 {
        let v3 = input - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a0 * v3;
        let v2 = self.ic2eq + self.a1 * (self.a0 * v1);
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        v2
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

    /// 0-127 parameter → Q value (0.5 - 10)
    pub fn resonance_param_to_q(param: u8) -> f32 {
        0.5 + (param & 0x7F) as f32 / 127.0 * 9.5
    }
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
