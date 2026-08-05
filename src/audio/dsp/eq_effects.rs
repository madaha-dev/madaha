/// EQ effects: 3BandEQ / 2BandEQ
///
/// - 3Band (mono): low shelf + mid peaking + high shelf
/// - 2Band (stereo): low shelf + high shelf
use crate::midi::effect_params::effect_obj::{three_band_eq_param, two_band_eq_param};

use super::core::biquad::{Biquad, make_biquad};
use super::params::p16;
use super::EffectProcessor;

/// gain param (64=0dB) → dB
#[inline]
fn gain_db(v: u16) -> f32 {
    (v as f32 - 64.0) / 64.0 * 12.0
}

/// freq param (0-127) → Hz (logarithmic approx: 20Hz..16kHz)
#[inline]
fn freq_hz(v: u16) -> f32 {
    let t = v.min(127) as f32 / 127.0;
    20.0 * 800f32.powf(t)
}

/// q param (0-127) → 0.1..12
#[inline]
fn q_param(v: u16) -> f32 {
    (v as f32 / 10.0).clamp(0.1, 12.0)
}

pub struct ThreeBandEqEffect {
    low: Biquad,
    mid: Biquad,
    high: Biquad,
}

impl ThreeBandEqEffect {
    pub fn new() -> Self {
        Self { low: Biquad::new(), mid: Biquad::new(), high: Biquad::new() }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.low = make_biquad(
            gain_db(p16(params, three_band_eq_param::EQ_LOW_GAIN)),
            freq_hz(p16(params, three_band_eq_param::EQ_LOW_FREQ)),
            0.707,
            false,
            sample_rate,
        );
        self.mid = make_biquad(
            gain_db(p16(params, three_band_eq_param::EQ_MID_GAIN)),
            freq_hz(p16(params, three_band_eq_param::EQ_MID_FREQ)),
            q_param(p16(params, three_band_eq_param::EQ_MID_WIDTH)),
            true,
            sample_rate,
        );
        self.high = make_biquad(
            gain_db(p16(params, three_band_eq_param::EQ_HIGH_GAIN)),
            freq_hz(p16(params, three_band_eq_param::EQ_HIGH_FREQ)),
            0.707,
            false,
            sample_rate,
        );
    }
}

impl EffectProcessor for ThreeBandEqEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let out_l = self.high.tick(self.mid.tick(self.low.tick(l)));
        let out_r = self.high.tick(self.mid.tick(self.low.tick(r)));
        (out_l, out_r)
    }
}

pub struct TwoBandEqEffect {
    low: Biquad,
    mid: Biquad,
    high: Biquad,
}

impl TwoBandEqEffect {
    pub fn new() -> Self {
        Self { low: Biquad::new(), mid: Biquad::new(), high: Biquad::new() }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.low = make_biquad(
            gain_db(p16(params, two_band_eq_param::EQ_LOW_GAIN)),
            freq_hz(p16(params, two_band_eq_param::EQ_LOW_FREQ)),
            0.707,
            false,
            sample_rate,
        );
        self.mid = make_biquad(
            gain_db(p16(params, two_band_eq_param::EQ_MID_GAIN)),
            freq_hz(p16(params, two_band_eq_param::EQ_MID_FREQ)),
            q_param(p16(params, two_band_eq_param::EQ_MID_WIDTH)),
            true,
            sample_rate,
        );
        self.high = make_biquad(
            gain_db(p16(params, two_band_eq_param::EQ_HIGH_GAIN)),
            freq_hz(p16(params, two_band_eq_param::EQ_HIGH_FREQ)),
            0.707,
            false,
            sample_rate,
        );
    }
}

impl EffectProcessor for TwoBandEqEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let l = self.mid.tick(self.low.tick(l));
        let r = self.mid.tick(self.low.tick(r));
        (self.high.tick(l), self.high.tick(r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_band_boosts_mid() {
        let mut e = ThreeBandEqEffect::new();
        let mut p = [0u16; 16];
        p[three_band_eq_param::EQ_MID_GAIN] = 127; // +12dB
        p[three_band_eq_param::EQ_MID_FREQ] = 70; // ~1kHz
        e.set_params(&p, 44100.0);
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let x = (phase * std::f32::consts::PI * 2.0).sin();
            phase = (phase + 1000.0 / 44100.0) % 1.0;
            let (l, _) = e.process((x, x));
            peak = peak.max(l.abs());
        }
        assert!(peak > 1.2, "peak={peak}");
    }

    #[test]
    fn two_band_zero_gain_bypass() {
        let mut e = TwoBandEqEffect::new();
        let mut p = [0u16; 16];
        p[two_band_eq_param::EQ_LOW_GAIN] = 64; // 0dB
        p[two_band_eq_param::EQ_HIGH_GAIN] = 64;
        p[two_band_eq_param::EQ_MID_GAIN] = 64;
        e.set_params(&p, 44100.0);
        let out = e.process((0.3, -0.3));
        assert!((out.0 - 0.3).abs() < 1e-4);
        assert!((out.1 + 0.3).abs() < 1e-4);
    }
}
