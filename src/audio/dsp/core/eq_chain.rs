/// Generic 3-band EQ chain (shared by XG effects: Chorus/Tremolo/Distortion/Wah, etc.)
///
/// Param layouts:
/// - Chorus layout: EQ_LOW_FREQ/GAIN(6/7), EQ_HIGH_FREQ/GAIN(8/9), EQ_MID_FREQ/GAIN/WIDTH(11/12/13)
/// - Distortion layout: EQ_LOW(2/3), EQ_MID(7/8/9) (no high band)
///
/// Frequencies looked up via XG_EQ_FREQ_TABLE (20Hz-20kHz), gain 64=0dB
use super::biquad::{Biquad, make_biquad};
use crate::midi::effect_params::parameter_table::XG_EQ_FREQ_TABLE;

#[derive(Debug)]
pub struct EqChain {
    low: Biquad,
    mid: Biquad,
    high: Biquad,
}

/// gain param (64=0dB) → dB
#[inline]
fn gain_db(v: u16) -> f32 {
    (v as f32 - 64.0) / 64.0 * 12.0
}

/// freq param → Hz (XG_EQ_FREQ_TABLE, 0-60 → 20Hz-20kHz)
#[inline]
fn freq_hz(v: u16) -> f32 {
    XG_EQ_FREQ_TABLE[(v.min(60)) as usize]
}

/// Q param → 0.1..12
#[inline]
fn q_param(v: u16) -> f32 {
    (v as f32 / 10.0).clamp(0.1, 12.0)
}

impl EqChain {
    pub fn new() -> Self {
        Self {
            low: Biquad::new(),
            mid: Biquad::new(),
            high: Biquad::new(),
        }
    }

    /// Generic 3-band setup (gain=64 → that band is bypassed)
    pub fn set_bands(
        &mut self,
        low_freq: u16,
        low_gain: u16,
        mid_freq: u16,
        mid_gain: u16,
        mid_width: u16,
        high_freq: u16,
        high_gain: u16,
        sample_rate: f32,
    ) {
        self.low = if low_gain != 64 {
            make_biquad(gain_db(low_gain), freq_hz(low_freq), 0.707, false, sample_rate)
        } else {
            Biquad::new()
        };
        self.mid = if mid_gain != 64 {
            make_biquad(
                gain_db(mid_gain),
                freq_hz(mid_freq),
                q_param(mid_width),
                true,
                sample_rate,
            )
        } else {
            Biquad::new()
        };
        self.high = if high_gain != 64 {
            make_biquad(gain_db(high_gain), freq_hz(high_freq), 0.707, false, sample_rate)
        } else {
            Biquad::new()
        };
    }

    /// Chorus layout: LOW(6/7), HIGH(8/9), MID(11/12/13)
    pub fn set_chorus_layout(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.set_bands(
            params[6],
            params[7],
            params[11],
            params[12],
            params[13],
            params[8],
            params[9],
            sample_rate,
        );
    }

    /// Distortion layout: LOW(2/3), MID(7/8/9), no high
    pub fn set_distortion_layout(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.set_bands(params[2], params[3], params[7], params[8], params[9], 0, 64, sample_rate);
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let x = self.low.tick(x);
        let x = self.mid.tick(x);
        self.high.tick(x)
    }

    pub fn reset(&mut self) {
        self.low.reset();
        self.mid.reset();
        self.high.reset();
    }
}

impl Default for EqChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_gain_bypass() {
        let mut eq = EqChain::new();
        eq.set_bands(10, 64, 30, 64, 64, 50, 64, 44100.0);
        let out = eq.tick(0.3);
        assert!((out - 0.3).abs() < 1e-4);
    }

    #[test]
    fn mid_boost_amplifies() {
        let mut eq = EqChain::new();
        eq.set_bands(10, 64, 35, 127, 64, 50, 64, 44100.0);
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let x = (phase * std::f32::consts::PI * 2.0).sin();
            phase = (phase + 1000.0 / 44100.0) % 1.0;
            peak = peak.max(eq.tick(x).abs());
        }
        assert!(peak > 1.2, "peak={peak}");
    }
}
