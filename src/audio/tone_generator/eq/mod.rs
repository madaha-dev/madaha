/// EQ (4-band: BASS / MID-BASS / MID-TREBLE / TREBLE)
///
/// 实现: RBJ Audio EQ Cookbook biquad
/// - Bass:  low shelving (shape=0) 或 peaking (shape=1), 32-2kHz
/// - Mid-Bass: peaking (XG Spec 无 MID shape 参数, Spec 标 NOT USED, 此处仍实现)
/// - Mid-Treble: peaking (同上)
/// - Treble: high shelving (shape=0) 或 peaking (shape=1), 100-16kHz
///
/// 对齐说明 (MultiPart 08 pp 72-7F):
/// - gain: 0-127 → -12..+12 dB (64=0dB, 直通)
/// - freq: XG_EQ_FREQ_TABLE 查表 (Spec Table #3)
/// - Q: 1-120 → 0.1-12.0
/// - shape: 0=shelving, 1=peaking (仅 BASS/TREBLE)
use crate::fast_sine::{fast_cos, fast_sin};
use crate::midi::effect_params::parameter_table::XG_EQ_FREQ_TABLE;

#[derive(Debug)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    fn new() -> Self {
        // 直通
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
    fn tick(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

#[derive(Debug)]
pub struct EQ {
    bass: Biquad,
    mid_bass: Biquad,
    mid_treble: Biquad,
    treble: Biquad,
    sample_rate: f32,
}

impl EQ {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bass: Biquad::new(),
            mid_bass: Biquad::new(),
            mid_treble: Biquad::new(),
            treble: Biquad::new(),
            sample_rate,
        }
    }

    /// 设置四段 EQ 参数
    /// `bass_gain_db / treble_gain_db`: -12..+12 dB
    /// `bass_freq / treble_freq`: Hz
    /// `bass_q / treble_q`: 0.1-12
    /// `bass_peak / treble_peak`: true=peaking, false=shelving
    /// `mid_*`: MID-BASS / MID-TREBLE 段 (固定 peaking, XG Spec NOT USED)
    pub fn set_params(
        &mut self,
        bass_gain_db: f32,
        bass_freq: f32,
        bass_q: f32,
        bass_peak: bool,
        mid_bass_gain_db: f32,
        mid_bass_freq: f32,
        mid_bass_q: f32,
        mid_treble_gain_db: f32,
        mid_treble_freq: f32,
        mid_treble_q: f32,
        treble_gain_db: f32,
        treble_freq: f32,
        treble_q: f32,
        treble_peak: bool,
    ) {
        self.bass = make_biquad(bass_gain_db, bass_freq, bass_q, bass_peak, self.sample_rate);
        self.mid_bass =
            make_biquad(mid_bass_gain_db, mid_bass_freq, mid_bass_q, true, self.sample_rate);
        self.mid_treble =
            make_biquad(mid_treble_gain_db, mid_treble_freq, mid_treble_q, true, self.sample_rate);
        self.treble =
            make_biquad(treble_gain_db, treble_freq, treble_q, treble_peak, self.sample_rate);
    }

    /// gain 参数 (0-127, 64=0dB) → dB (-12..+12)
    pub fn gain_param_to_db(param: u8) -> f32 {
        (param as f32 - 64.0) / 64.0 * 12.0
    }

    /// freq 参数 (0-127) → Hz (XG_EQ_FREQ_TABLE 查表)
    pub fn freq_param_to_hz(param: u8) -> f32 {
        XG_EQ_FREQ_TABLE[(param as usize).min(60)]
    }

    /// Q 参数 (1-120) → Q (0.1-12.0)
    pub fn q_param_to_q(param: u8) -> f32 {
        (param as f32 / 10.0).clamp(0.1, 12.0)
    }

    #[inline]
    pub fn tick(&mut self, input: f32) -> f32 {
        let x = self.bass.tick(input);
        let x = self.mid_bass.tick(x);
        let x = self.mid_treble.tick(x);
        self.treble.tick(x)
    }
}

/// RBJ biquad 系数
fn make_biquad(gain_db: f32, freq: f32, q: f32, peak: bool, sample_rate: f32) -> Biquad {
    if gain_db.abs() < 1e-4 {
        return Biquad::new(); // 0dB → 直通
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
        let mut eq = EQ::new(44100.0);
        eq.set_params(0.0, 100.0, 1.0, false, 0.0, 1000.0, 1.0, 0.0, 4000.0, 1.0, 0.0, 5000.0, 1.0, false);
        let mut out = 0.0;
        for _ in 0..100 {
            out = eq.tick(1.0);
        }
        assert!((out - 1.0).abs() < 1e-3, "out={out}");
    }

    #[test]
    fn bass_boost_amplifies_low() {
        let mut eq = EQ::new(44100.0);
        eq.set_params(12.0, 100.0, 1.0, false, 0.0, 1000.0, 1.0, 0.0, 4000.0, 1.0, 0.0, 5000.0, 1.0, false);
        // 低频正弦 (100Hz) 应被放大
        let mut out: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let input = (phase * 2.0 * std::f32::consts::PI).sin();
            phase = (phase + 100.0f32 / 44100.0) % 1.0;
            out = out.max(eq.tick(input).abs());
        }
        assert!(out > 1.2, "peak={out}");
    }

    #[test]
    fn mid_boost_amplifies_center_freq() {
        let mut eq = EQ::new(44100.0);
        eq.set_params(0.0, 100.0, 1.0, false, 12.0, 1000.0, 1.0, 0.0, 4000.0, 1.0, 0.0, 5000.0, 1.0, false);
        // 1kHz 正弦 (peaking 中心) 应被放大, 100Hz 应保持 ~1
        let mut mid_peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let input = (phase * 2.0 * std::f32::consts::PI).sin();
            phase = (phase + 1000.0f32 / 44100.0) % 1.0;
            mid_peak = mid_peak.max(eq.tick(input).abs());
        }
        assert!(mid_peak > 1.5, "mid_peak={mid_peak}");
    }

    #[test]
    fn freq_param_mapping() {
        assert!((EQ::freq_param_to_hz(0) - 20.0).abs() < 0.1);
        assert!((EQ::freq_param_to_hz(60) - 20000.0).abs() < 1.0);
        assert!((EQ::gain_param_to_db(64)).abs() < 1e-4);
        assert!((EQ::gain_param_to_db(127) - 11.8125).abs() < 1e-3);
        assert!((EQ::gain_param_to_db(0) + 12.0).abs() < 1e-3);
    }
}
