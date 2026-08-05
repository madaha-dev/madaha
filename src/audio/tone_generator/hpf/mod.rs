/// High-pass filter (HPF, XG optional module)
///
/// Implementation: high-pass output of a two-pole state-variable filter (SVF)
///   low = v2, band = v1, high = input - k*v1 - v2
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
    /// Q value
    pub resonance: f32,

    // SVF state
    ic1eq: f32,
    ic2eq: f32,
    // Coefficient cache
    a0: f32,
    a1: f32,
    k: f32,
}

impl HPF {
    pub fn new() -> Self {
        Self {
            mod_offset: 0.0,
            cutoff: 100.0,
            resonance: 1.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            a0: 0.0,
            a1: 0.0,
            k: 0.0,
        }
    }

    pub fn set_params(&mut self, cutoff_hz: f32, q: f32, sample_rate: f32) {
        self.cutoff = cutoff_hz.max(1.0);
        self.resonance = q.max(0.1);
        let w = 2.0 * std::f32::consts::PI * self.cutoff / sample_rate;
        let g = w.tan();
        self.k = 1.0 / self.resonance;
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a0 = g;
    }

    /// Process one sample, return high-pass output
    pub fn tick(&mut self, input: f32) -> f32 {
        let v3 = input - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a0 * v3;
        let v2 = self.ic2eq + self.a1 * (self.a0 * v1);
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        input - self.k * v1 - v2
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
