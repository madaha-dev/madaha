/// Distortion effects: Distortion / Overdrive / AmpSimulator / AuralExciter
///
/// - Distortion/Overdrive: soft-clip waveshaping + LPF + output level
/// - AmpSimulator: waveshaping + AMP_TYPE simple EQ
/// - AuralExciter: HPF → harmonic enhancement → mix in
use crate::midi::effect_params::effect_obj::{
    aural_exiceter_param, distortion_param, guitar_amp_simulator_param,
};

use super::core::biquad::{Biquad, make_biquad};
use super::core::eq_chain::EqChain;
use super::params::{dry_wet, p16};
use super::EffectProcessor;

/// Soft clip (tanh approx): input × drive, output -1..1
#[inline]
fn soft_clip(x: f32, drive: f32) -> f32 {
    let x = x * drive;
    x.tanh()
}

/// 2006LE CalcDistortion segmented shaper (0x555cc): 4-stage clamp cascade + accumulate
///   v = in×drive; out = Σ clamp(stage_k, ±1)×g_k  with per-stage re-gain (edge)
#[inline]
fn shaper_2006(x: f32, drive: f32, edge: f32) -> f32 {
    let v = x * drive;
    let mut acc = 0.0;
    let mut stage = v;
    let mut g = 1.0;
    for _ in 0..4 {
        let c = stage.clamp(-1.0, 1.0);
        acc += c * g;
        g *= 0.5;
        stage = c * (1.0 + edge);
    }
    acc + stage.clamp(-1.0, 1.0) * g
}

pub struct DistortionEffect {
    drive: f32,
    edge: f32,
    lpf: Biquad,
    output: f32,
    dry: f32,
    wet: f32,
    /// EQ (LOW 2/3, MID 7/8/9)
    eq: EqChain,
    sample_rate: f32,
}

impl DistortionEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            drive: 1.0,
            edge: 0.0,
            lpf: Biquad::new(),
            output: 1.0,
            dry: 0.0,
            wet: 1.0,
            eq: EqChain::new(),
            sample_rate,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], _distortion: bool) {
        // DRIVE (0-127) → 1..32
        self.drive = 1.0 + p16(params, distortion_param::DRIVE) as f32 / 127.0 * 31.0;
        // EDGE: 0-127 → clipping hardness (0=soft, 127=hard)
        self.edge = p16(params, distortion_param::EDGE) as f32 / 127.0;
        // LPF_CUTOFF (0-127) → Hz (0 = off)
        let lpf_cutoff = p16(params, distortion_param::LPF_CUTOFF);
        self.lpf = if lpf_cutoff > 0 {
            make_biquad(-6.0, lpf_cutoff as f32 * 100.0, 0.707, false, self.sample_rate)
        } else {
            Biquad::new()
        };
        // EQ_LOW_FREQ/GAIN(2/3), EQ_MID_FREQ/GAIN/WIDTH(7/8/9)
        self.eq.set_distortion_layout(params, self.sample_rate);
        // OUTPUT_LEVEL (0-127) → linear
        self.output = (p16(params, distortion_param::OUTPUT_LEVEL) as f32 / 127.0)
            .max(0.01)
            .min(2.0);
        let (d, w) = dry_wet(p16(params, distortion_param::DRY_WET));
        self.dry = d;
        self.wet = w;
    }
}

impl EffectProcessor for DistortionEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // Input EQ → waveshaping (edge controls hardness: 0=tanh soft, 1=hard limit)
        let l = self.eq.tick(l);
        let r = self.eq.tick(r);
        // 2006LE segmented shaper (drive + edge control the cascade)
        let clip_l = shaper_2006(l, self.drive, self.edge);
        let clip_r = shaper_2006(r, self.drive, self.edge);
        // Output gain
        let out_l = self.lpf.tick(clip_l) * self.output;
        let out_r = self.lpf.tick(clip_r) * self.output;
        (l * self.dry + out_l * self.wet, r * self.dry + out_r * self.wet)
    }
}

// ──────────────────────── Amp Simulator ────────────────────────
pub struct AmpSimEffect {
    drive: f32,
    /// AMP_TYPE(2): 0=Flat, 1=Combo, 2=Stack, 3=Twin (tone shaping)
    amp_type: u8,
    /// EDGE(11): clipping hardness
    edge: f32,
    output: f32,
    lpf: Biquad,
    dry: f32,
    wet: f32,
    sample_rate: f32,
}

impl AmpSimEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            drive: 1.0,
            amp_type: 0,
            edge: 0.0,
            output: 1.0,
            lpf: Biquad::new(),
            dry: 0.0,
            wet: 1.0,
            sample_rate,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16]) {
        self.drive = 1.0 + p16(params, guitar_amp_simulator_param::DRIVE) as f32 / 127.0 * 20.0;
        self.amp_type = p16(params, guitar_amp_simulator_param::AMP_TYPE).min(3) as u8;
        self.edge = p16(params, guitar_amp_simulator_param::EDGE) as f32 / 127.0;
        self.output = (p16(params, guitar_amp_simulator_param::OUTPUT_LEVEL) as f32 / 127.0)
            .max(0.01)
            .min(2.0);
        let lpf_cutoff = p16(params, guitar_amp_simulator_param::LPF_CUTOFF);
        self.lpf = if lpf_cutoff > 0 {
            make_biquad(-6.0, lpf_cutoff as f32 * 100.0, 0.707, false, self.sample_rate)
        } else {
            Biquad::new()
        };
        let (d, w) = dry_wet(p16(params, guitar_amp_simulator_param::DRY_WET));
        self.dry = d;
        self.wet = w;
    }
}

impl EffectProcessor for AmpSimEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // AMP_TYPE tone shaping: 0=soft saturation, 1=combo mid emphasis, 2=stack hard clip, 3=twin bright
        let (drive_k, bright) = match self.amp_type {
            0 => (1.0, 1.0),
            1 => (1.2, 0.9),
            2 => (1.5, 1.0),
            _ => (1.0, 1.2),
        };
        let drive = self.drive * drive_k;
        // EDGE: 0=tanh, 1=hard limit
        let clip = |x: f32| -> f32 {
            let s = soft_clip(x, drive);
            s * (1.0 - self.edge) + x.clamp(-1.0, 1.0) * self.edge
        };
        let out_l = self.lpf.tick(clip(l)) * self.output * bright;
        let out_r = self.lpf.tick(clip(r)) * self.output * bright;
        (l * self.dry + out_l * self.wet, r * self.dry + out_r * self.wet)
    }
}

// ──────────────────────── Aural Exciter ────────────────────────
pub struct AuralExciterEffect {
    hpf: Biquad,
    drive: f32,
    mix: f32,
    sample_rate: f32,
}

impl AuralExciterEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            hpf: Biquad::new(),
            drive: 1.0,
            mix: 0.5,
            sample_rate,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16]) {
        let hpf_cutoff = p16(params, aural_exiceter_param::HPF_CUTOFF);
        self.hpf = if hpf_cutoff > 0 {
            // High-pass: first-order approx (inverted biquad high shelf)
            make_biquad(-12.0, hpf_cutoff as f32 * 100.0, 0.707, false, self.sample_rate)
        } else {
            Biquad::new()
        };
        self.drive = 1.0 + p16(params, aural_exiceter_param::DRIVE) as f32 / 127.0 * 8.0;
        self.mix = p16(params, aural_exiceter_param::MIX_LEVEL) as f32 / 127.0;
    }
}

impl EffectProcessor for AuralExciterEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // Harmonic enhancement: HPF → clipping creates harmonics → mix in
        let h_l = self.hpf.tick(l);
        let h_r = self.hpf.tick(r);
        let excite_l = (h_l * self.drive).tanh() * 2.0 - h_l;
        let excite_r = (h_r * self.drive).tanh() * 2.0 - h_r;
        (l + excite_l * self.mix, r + excite_r * self.mix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distortion_clips_amplitude() {
        let mut e = DistortionEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[distortion_param::DRIVE] = 127;
        p[distortion_param::OUTPUT_LEVEL] = 100;
        p[distortion_param::DRY_WET] = 1; // full wet
        e.set_params(&p, true);
        // Large signal clipped to a finite range
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let x = (phase * std::f32::consts::PI * 2.0).sin() * 2.0;
            phase = (phase + 440.0 / 44100.0) % 1.0;
            let (l, _) = e.process((x, x));
            peak = peak.max(l.abs());
        }
        assert!(peak > 0.5 && peak < 2.5, "peak={peak}");
    }

    #[test]
    fn aural_exciter_passes_low_band() {
        let mut e = AuralExciterEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[aural_exiceter_param::HPF_CUTOFF] = 20;
        p[aural_exiceter_param::MIX_LEVEL] = 0; // no mixing → passthrough
        e.set_params(&p);
        let out = e.process((0.25, 0.25));
        assert!((out.0 - 0.25).abs() < 1e-4);
    }
}
