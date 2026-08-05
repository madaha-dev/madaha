/// Dynamics effects: Compressor / NoiseGate
///
/// - Compressor: RMS detection + threshold/ratio gain reduction
/// - NoiseGate: gating below threshold
use crate::midi::effect_params::effect_obj::{compressor_param, noise_gate_param};
use crate::midi::effect_params::parameter_table::{
    XG_COMPRESSOR_ATTACK_TIME_TABLE, XG_COMPRESSOR_RATIO_TABLE, XG_COMPRESSOR_RELEASE_TIME_TABLE,
};

use super::params::p16;
use super::EffectProcessor;

pub struct CompressorEffect {
    attack: f32,
    release: f32,
    threshold: f32,
    ratio: f32,
    output: f32,
    /// Envelope (smoothed gain reduction)
    gain_env: f32,
    /// RMS envelope
    rms: f32,
    sample_rate: f32,
}

impl CompressorEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack: 0.01,
            release: 0.1,
            threshold: 0.5,
            ratio: 2.0,
            output: 1.0,
            gain_env: 1.0,
            rms: 0.0,
            sample_rate,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        let atk = p16(params, compressor_param::ATTACK);
        let rel = p16(params, compressor_param::RELEASE);
        let thr = p16(params, compressor_param::THRESHOLD);
        let ratio = p16(params, compressor_param::RATIO);
        self.sample_rate = sample_rate;
        // XG Spec Table #8/9/10: dedicated compressor tables
        let atk_idx = (atk as usize * 19 / 127).min(19);
        let rel_idx = (rel as usize * 15 / 127).min(15);
        self.attack = XG_COMPRESSOR_ATTACK_TIME_TABLE[atk_idx] / 1000.0;
        self.release = XG_COMPRESSOR_RELEASE_TIME_TABLE[rel_idx] / 1000.0;
        // Threshold (-60..0dB)
        self.threshold = 10f32.powf(-(60.0 * (127 - thr) as f32 / 127.0) / 20.0);
        // XG Spec Table #10: ratio 1..20
        self.ratio = XG_COMPRESSOR_RATIO_TABLE[(ratio as usize * 7 / 127).min(7)];
        self.output = (p16(params, compressor_param::OUTPUT_LEVEL) as f32 / 127.0)
            .max(0.01)
            .min(2.0);
    }
}

impl EffectProcessor for CompressorEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // RMS detection
        let inst = ((l * l + r * r) * 0.5).sqrt();
        self.rms += (inst - self.rms) * 0.01;
        // Gain calc: portion above threshold compressed by ratio
        let above = self.rms / self.threshold.max(1e-6);
        let gain = if above > 1.0 {
            // Compression amount: excess portion 1/ratio
            above.powf(1.0 / self.ratio - 1.0)
        } else {
            1.0
        };
        // Smoothing (attack/release time constants)
        let coeff = if gain < self.gain_env {
            // Compression occurring → attack
            1.0 - (-1.0 / (self.attack * self.sample_rate)).exp()
        } else {
            // Recovery → release
            1.0 - (-1.0 / (self.release * self.sample_rate)).exp()
        };
        self.gain_env += (gain - self.gain_env) * coeff;
        let g = self.gain_env * self.output;
        (l * g, r * g)
    }
}

pub struct NoiseGateEffect {
    attack: f32,
    release: f32,
    threshold: f32,
    /// RATIO(11): attenuation ratio when the gate is closed (1=fully closed, large=shallow close)
    ratio: f32,
    output: f32,
    /// Gate state (0-1 smoothed)
    gate: f32,
    sample_rate: f32,
}

impl NoiseGateEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack: 0.005,
            release: 0.05,
            threshold: 0.05,
            ratio: 1.0,
            output: 1.0,
            gate: 1.0,
            sample_rate,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        let atk = p16(params, noise_gate_param::ATTACK);
        let rel = p16(params, noise_gate_param::RELEASE);
        let thr = p16(params, noise_gate_param::THRESHOLD);
        // RATIO(11): 1-20 (close depth: 1=fully closed, 20=shallow attenuation)
        let ratio = p16(params, noise_gate_param::RATIO);
        self.ratio = if ratio > 0 {
            1.0 + ratio as f32 / 127.0 * 19.0
        } else {
            1.0
        };
        self.sample_rate = sample_rate;
        // XG Spec Table #8/9 (shared with compressor)
        let atk_idx = (atk as usize * 19 / 127).min(19);
        let rel_idx = (rel as usize * 15 / 127).min(15);
        self.attack = XG_COMPRESSOR_ATTACK_TIME_TABLE[atk_idx] / 1000.0;
        self.release = XG_COMPRESSOR_RELEASE_TIME_TABLE[rel_idx] / 1000.0;
        self.threshold = 10f32.powf(-(60.0 * (127 - thr) as f32 / 127.0) / 20.0);
        self.output = (p16(params, noise_gate_param::OUTPUT_LEVEL) as f32 / 127.0)
            .max(0.01)
            .min(2.0);
    }
}

impl EffectProcessor for NoiseGateEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let mag = ((l * l + r * r) * 0.5).sqrt();
        // Below threshold: gain = 1/ratio (shallow close) instead of 0 (fully closed)
        let target = if mag > self.threshold {
            1.0
        } else {
            1.0 / self.ratio
        };
        let coeff = if target > self.gate {
            1.0 - (-1.0 / (self.attack * self.sample_rate)).exp()
        } else {
            1.0 - (-1.0 / (self.release * self.sample_rate)).exp()
        };
        self.gate += (target - self.gate) * coeff;
        let g = self.gate * self.output;
        (l * g, r * g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compressor_reduces_loud_signal() {
        let mut e = CompressorEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[compressor_param::THRESHOLD] = 100; // low threshold (high value = low threshold)
        p[compressor_param::RATIO] = 127; // max compression
        p[compressor_param::ATTACK] = 127; // fast
        p[compressor_param::RELEASE] = 80;
        p[compressor_param::OUTPUT_LEVEL] = 127;
        e.set_params(&p, 44100.0);
        // Constant large signal → output much smaller than input
        let mut out = (0.0, 0.0);
        for _ in 0..4410 {
            out = e.process((1.0, 1.0));
        }
        assert!(out.0 < 1.0, "out={:?}", out);
        assert!(out.0 > 0.01, "out={:?}", out);
    }

    #[test]
    fn noise_gate_closes_on_silence() {
        let mut e = NoiseGateEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[noise_gate_param::THRESHOLD] = 90; // low threshold
        p[noise_gate_param::ATTACK] = 127;
        p[noise_gate_param::RELEASE] = 127;
        e.set_params(&p, 44100.0);
        // Loud first, then silent → gate closes
        for _ in 0..100 {
            e.process((1.0, 1.0));
        }
        let mut out = (1.0, 1.0);
        for _ in 0..4410 {
            out = e.process((0.0, 0.0));
        }
        assert!(out.0 < 0.05, "out={:?}", out);
    }
}
