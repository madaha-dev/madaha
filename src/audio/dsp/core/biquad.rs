/// RBJ Audio EQ Cookbook biquad (shared implementation extracted from tone_generator/eq)
use crate::fast_sine::{fast_cos, fast_sin};

#[derive(Debug, Clone)]
pub struct Biquad {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    pub fn new() -> Self {
        // Passthrough
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

impl Default for Biquad {
    fn default() -> Self {
        Self::new()
    }
}

/// RBJ biquad coefficients
/// - `peak=true`: peaking EQ
/// - `peak=false` and `freq < 1000Hz`: low shelf
/// - `peak=false` and `freq >= 1000Hz`: high shelf
/// 0dB gain → passthrough
pub fn make_biquad(gain_db: f32, freq: f32, q: f32, peak: bool, sample_rate: f32) -> Biquad {
    if gain_db.abs() < 1e-4 {
        return Biquad::new();
    }
    let a = 10f32.powf(gain_db / 40.0);
    let w = 2.0 * std::f32::consts::PI * freq / sample_rate;
    let cos_w = fast_cos(w);
    let alpha = fast_sin(w) / (2.0 * q);
    let sqrt_a = a.sqrt();

    let (b0, b1, b2, a0, a1, a2) = if peak {
        // peaking EQ
        (
            1.0 + alpha * a,
            -2.0 * cos_w,
            1.0 - alpha * a,
            1.0 + alpha / a,
            -2.0 * cos_w,
            1.0 - alpha / a,
        )
    } else if freq < 1000.0 {
        // low shelf
        (
            a * ((a + 1.0) - (a - 1.0) * cos_w + 2.0 * sqrt_a * alpha),
            2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w),
            a * ((a + 1.0) - (a - 1.0) * cos_w - 2.0 * sqrt_a * alpha),
            (a + 1.0) + (a - 1.0) * cos_w + 2.0 * sqrt_a * alpha,
            -2.0 * ((a - 1.0) + (a + 1.0) * cos_w),
            (a + 1.0) + (a - 1.0) * cos_w - 2.0 * sqrt_a * alpha,
        )
    } else {
        // high shelf
        (
            a * ((a + 1.0) + (a - 1.0) * cos_w + 2.0 * sqrt_a * alpha),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w),
            a * ((a + 1.0) + (a - 1.0) * cos_w - 2.0 * sqrt_a * alpha),
            (a + 1.0) - (a - 1.0) * cos_w + 2.0 * sqrt_a * alpha,
            2.0 * ((a - 1.0) - (a + 1.0) * cos_w),
            (a + 1.0) - (a - 1.0) * cos_w - 2.0 * sqrt_a * alpha,
        )
    };

    Biquad {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
        z1: 0.0,
        z2: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_gain_bypass() {
        let mut b = make_biquad(0.0, 1000.0, 1.0, true, 44100.0);
        for _ in 0..100 {
            let out = b.tick(1.0);
            assert!((out - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn peaking_boost_amplifies_center() {
        let mut b = make_biquad(12.0, 1000.0, 1.0, true, 44100.0);
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let input = (phase * 2.0 * std::f32::consts::PI).sin();
            phase = (phase + 1000.0f32 / 44100.0) % 1.0;
            peak = peak.max(b.tick(input).abs());
        }
        assert!(peak > 1.5, "peak={peak}");
    }

    #[test]
    fn shelf_cut_reduces_band() {
        // low shelf -12dB → high frequencies stay ~1, low frequencies attenuated
        let mut b = make_biquad(-12.0, 300.0, 1.0, false, 44100.0);
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let input = (phase * 2.0 * std::f32::consts::PI).sin();
            phase = (phase + 100.0f32 / 44100.0) % 1.0;
            peak = peak.max(b.tick(input).abs());
        }
        assert!(peak < 0.3, "peak={peak}");
    }
}
