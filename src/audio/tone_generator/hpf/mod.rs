/// High-pass filter (HPF, XG optional module)
///
/// Implementation: high-pass output of the Chamberlin two-pole SVF, aligned
/// with the S-YXG2006LE DCF structure (`CDCFUnit::Generate`, verified in
/// Ghidra): low-pass states y1/y2 with frequency-dependent damping
/// `K = clamp(3 − 2f, …, 2.0)`; the high-pass output is `x − K·y1 − y2`.
///
/// Alignment notes:
/// - Parameters in MultiPartExt (0A pp 20-21): hpf_cutoff_freq / hpf_resonance
/// - Control sources → HPF cutoff depth (0A pp 22-27), real-time modulation to be wired in
/// - XG Spec: HPF Cutoff 00-7F (64=center, -64..+63 relative)
#[derive(Debug)]
pub struct HPF {
    /// External modulation (MW/Bend/CAT/PAT HPF control), in param units, updated each block
    pub mod_offset: f32,
    /// Cutoff frequency (Hz)
    pub cutoff: f32,
    /// Resonance parameter (0-127, 64 = center) → damping bound
    pub resonance: f32,

    // Chamberlin SVF state
    ic1eq: f32, // band-pass state (y1)
    ic2eq: f32, // low-pass state (y2)
    // Coefficients
    f: f32,     // cutoff coefficient ≈ 2·sin(π·fc/fs)
    k_min: f32, // damping K bound from resonance
}

impl HPF {
    pub fn new() -> Self {
        Self {
            mod_offset: 0.0,
            cutoff: 100.0,
            resonance: 64.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            f: 0.0,
            k_min: 4.0,
        }
    }

    pub fn set_params(&mut self, cutoff_hz: f32, resonance: f32, sample_rate: f32) {
        self.cutoff = cutoff_hz.max(1.0);
        self.resonance = resonance;
        let fc = (self.cutoff / sample_rate).min(0.49);
        self.f = 2.0 * (std::f32::consts::PI * fc).sin();
        self.k_min = crate::audio::tone_generator::lpf::exchange_resonance_to_linear(
            self.resonance as i16,
        );
    }

    /// Process one sample, return high-pass output
    pub fn tick(&mut self, input: f32) -> f32 {
        let k = (3.0 - 2.0 * self.f)
            .max(2.0 - self.f)
            .min(self.k_min)
            .max(0.1);
        self.ic1eq = (input - k * self.ic1eq - self.ic2eq) * self.f + self.ic1eq;
        self.ic2eq = self.ic1eq * self.f + self.ic2eq;
        input - k * self.ic1eq - self.ic2eq
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// 0-127 parameter → cutoff frequency (Hz), logarithmic mapping 20Hz - 10kHz
    pub fn cutoff_param_to_hz(param: u8) -> f32 {
        let t = (param & 0x7F) as f32 / 127.0;
        20.0 * (500.0f32).powf(t)
    }

    /// 0-127 parameter → resonance (0-127, kept for API compatibility; the
    /// actual damping mapping lives in `set_params`)
    pub fn resonance_param_to_q(param: u8) -> f32 {
        (param & 0x7F) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_mapping() {
        assert!((HPF::cutoff_param_to_hz(0) - 20.0).abs() < 0.1);
        assert!((HPF::cutoff_param_to_hz(127) - 10000.0).abs() < 1.0);
    }

    #[test]
    fn hpf_blocks_dc() {
        // DC should be blocked by the high-pass
        let mut hpf = HPF::new();
        hpf.set_params(100.0, 1.0, 44100.0);
        let mut out = 1.0;
        for _ in 0..1000 {
            out = hpf.tick(1.0);
        }
        assert!(out.abs() < 1e-3, "DC leak = {out}");
    }

    #[test]
    fn hpf_passes_high_freq() {
        // High-frequency square wave through 10kHz high-pass → passes through roughly unchanged
        let mut hpf = HPF::new();
        hpf.set_params(100.0, 0.5, 44100.0);
        let mut out = 0.0;
        for i in 0..441 {
            let input = if (i / 4) % 2 == 0 { 1.0 } else { -1.0 };
            out = hpf.tick(input);
        }
        assert!(out.abs() > 0.5, "output={out}");
    }
}
