//! EBU R128 / ITU-R BS.1770-4 loudness meter + slow gain normalizer.
//!
//! Used as an output-side "dynamic loudness" stage: continuously estimates
//! the short-term (3 s) gated loudness of the rendered master bus and slowly
//! adapts a gain multiplier toward a target (default -14 LUFS). The gain moves
//! at most `max_step_db` per 400 ms block, so individual note transients are
//! preserved (the gain is too slow to react to a single attack); the separate
//! master peak limiter is still responsible for instantaneous peak protection.
//!
//! K-weighting: 2nd-order high-pass at 38 Hz (Butterworth) + 2nd-order
//! high-shelf +4 dB @ 1500 Hz (RBJ), matching the BS.1770-4 reference shape.
//! The AGC self-calibrates to its own meter, so small filter-shape differences
//! only shift the absolute calibration by a fraction of a dB.

use crate::fast_sine::{fast_cos, fast_sin};
use std::f32::consts::{FRAC_1_SQRT_2, PI};

use super::biquad::Biquad;

/// K-weighting filter (one per channel).
struct KWeighting {
    hp: Biquad,
    sh: Biquad,
}

impl KWeighting {
    pub fn new(fs: f32) -> Self {
        // 2nd-order high-pass @ 38 Hz, Butterworth Q = 1/√2 (RBJ cookbook).
        let w0 = 2.0 * PI * 38.0 / fs;
        let c = fast_cos(w0);
        let alpha = fast_sin(w0) / (2.0f32).sqrt();
        let a0 = 1.0 + alpha;
        let mut hp = Biquad::new();
        hp.b0 = (1.0 + c) / 2.0 / a0;
        hp.b1 = -(1.0 + c) / a0;
        hp.b2 = (1.0 + c) / 2.0 / a0;
        hp.a1 = -2.0 * c / a0;
        hp.a2 = (1.0 - alpha) / a0;

        // 2nd-order high-shelf +4 dB @ 1500 Hz (RBJ, Q ≈ 0.71).
        let sh = super::biquad::make_biquad(4.0, 1500.0, FRAC_1_SQRT_2, false, fs);

        Self { hp, sh }
    }

    #[inline]
    fn tick(&mut self, x: f32) -> f32 {
        self.sh.tick(self.hp.tick(x))
    }
}

/// Mean-square summed over channels inside a 400 ms block (linear energy).
struct BlockAccumulator {
    block_len: usize,
    acc_l: f64,
    acc_r: f64,
    pos: usize,
}

impl BlockAccumulator {
    fn new(fs: f32) -> Self {
        Self {
            block_len: ((fs * 0.4).round() as usize).max(1),
            acc_l: 0.0,
            acc_r: 0.0,
            pos: 0,
        }
    }

    /// Feed one (already K-weighted) sample; returns Some(block energy) when a
    /// block completes.
    #[inline]
    fn push(&mut self, kl: f32, kr: f32) -> Option<f64> {
        self.acc_l += (kl as f64) * (kl as f64);
        self.acc_r += (kr as f64) * (kr as f64);
        self.pos += 1;
        if self.pos >= self.block_len {
            let z = (self.acc_l + self.acc_r) / self.block_len as f64;
            self.acc_l = 0.0;
            self.acc_r = 0.0;
            self.pos = 0;
            Some(z)
        } else {
            None
        }
    }
}

/// Per-block energy → LUFS: Γ = -0.691 + 10·log10(z).
#[inline]
fn energy_to_lufs(z: f64) -> f32 {
    (-0.691 + 10.0 * z.max(1e-30).log10()) as f32
}

/// Short-term gated loudness over the last ~3 s window.
struct ShortTerm {
    /// Per-block energies (summed channels, linear), ring buffer.
    ring: Vec<f64>,
    pos: usize,
    filled: bool,
}

impl ShortTerm {
    fn new() -> Self {
        // R128 short-term window = 3 s → 400 ms blocks ≈ 7 full blocks.
        let st_blocks = ((3.0f32 / 0.4).floor() as usize).max(1);
        Self {
            ring: vec![0.0; st_blocks],
            pos: 0,
            filled: false,
        }
    }

    fn push(&mut self, z: f64) {
        self.ring[self.pos] = z;
        self.pos = (self.pos + 1) % self.ring.len();
        if self.pos == 0 {
            self.filled = true;
        }
    }

    /// Gated short-term loudness (absolute -70 LUFS + relative -10 LUFS).
    /// Returns None when the window has no audible blocks (silence).
    fn lufs(&self) -> Option<f32> {
        let n = if self.filled {
            self.ring.len()
        } else {
            self.pos
        };
        if n == 0 {
            return None;
        }
        // Absolute gate: -70 LUFS.
        let abs_thresh = 10f64.powf((-70.0 + 0.691) / 10.0);
        let mut sum = 0.0;
        let mut cnt = 0usize;
        for i in 0..n {
            let z = self.ring[i];
            if z > abs_thresh {
                sum += z;
                cnt += 1;
            }
        }
        if cnt == 0 {
            return None;
        }
        let z_abs = sum / cnt as f64;
        // Relative gate: -10 LU below the absolute-gated mean.
        let rel_thresh = z_abs * 10f64.powf(-10.0 / 10.0);
        let mut sum2 = 0.0;
        let mut cnt2 = 0usize;
        for i in 0..n {
            let z = self.ring[i];
            if z > rel_thresh {
                sum2 += z;
                cnt2 += 1;
            }
        }
        if cnt2 == 0 {
            return None;
        }
        Some(energy_to_lufs(sum2 / cnt2 as f64))
    }
}

/// Output-side dynamic loudness normalizer (slow AGC toward a target LUFS).
pub struct LoudnessNormalizer {
    kw_l: KWeighting,
    kw_r: KWeighting,
    acc: BlockAccumulator,
    st: ShortTerm,
    target_lufs: f32,
    gain: f32,
    max_step_db: f32,
    g_min: f32,
    g_max: f32,
    /// Freeze updates while the measured loudness is `None` (silence) to avoid
    /// pumping the gain up during pauses.
    frozen_silence: bool,
}

impl LoudnessNormalizer {
    pub fn new(sample_rate: f32) -> Self {
        // 初始增益 = 中性 1.0：输出即以固定基准（LUFS_GAIN 4.2）为起点，
        // AGC 随后缓慢双向调整（材料响则切、材料轻则抬）收敛到目标。
        Self {
            kw_l: KWeighting::new(sample_rate),
            kw_r: KWeighting::new(sample_rate),
            acc: BlockAccumulator::new(sample_rate),
            st: ShortTerm::new(),
            target_lufs: -14.0,
            gain: 1.0,
            // 尽量稳：每 400ms 块最多 ±0.5 dB —— 收敛慢但几乎听不到泵动。
            max_step_db: 0.5,
            // 宽范围：密集强音（基准可到 -3 LUFS）需切 -10dB+；轻音可抬 +15dB。
            g_min: 0.15,
            g_max: 8.0,
            frozen_silence: false,
        }
    }

    pub fn set_target(&mut self, target_lufs: f32) {
        self.target_lufs = target_lufs;
    }

    /// Feed one master-bus frame (BEFORE the adaptive gain is applied) and
    /// advance the AGC on block boundaries. Call once per output frame.
    pub fn push_frame(&mut self, l: f32, r: f32) {
        let kl = self.kw_l.tick(l);
        let kr = self.kw_r.tick(r);
        if let Some(z) = self.acc.push(kl, kr) {
            self.st.push(z);
            if let Some(lufs) = self.st.lufs() {
                let err = self.target_lufs - lufs;
                let step = err.clamp(-self.max_step_db, self.max_step_db);
                let m = (10f32).powf(step / 20.0);
                self.gain = (self.gain * m).clamp(self.g_min, self.g_max);
                self.frozen_silence = false;
            } else {
                // Silence: freeze (do not pump up in gaps).
                self.frozen_silence = true;
            }
        }
    }

    /// Current adaptive gain multiplier (1.0 → transparent at init calibration).
    pub fn gain(&self) -> f32 {
        self.gain
    }

    /// Current gated short-term loudness of whatever has been pushed (LUFS).
    /// Useful for verification; the AGC itself drives off this same meter.
    pub fn short_term(&self) -> Option<f32> {
        self.st.lufs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k_weighting_1khz_fullscale_matches_reference() {
        // 1 kHz full-scale sine → ≈ -3.01 LUFS per block (reference -3.01,
        // K-weighting at 1 kHz is ~0 dB). Tolerance generous.
        let fs = 44100.0;
        let kw_l = KWeighting::new(fs);
        let mut kw_l = kw_l;
        let mut acc = BlockAccumulator::new(fs);
        let mut zsum = 0.0f64;
        let mut n = 0usize;
        let mut phase: f32 = 0.0;
        let step = 1000.0 / fs;
        for _ in 0..(fs * 1.6) as usize {
            let input = (phase * 2.0 * PI).sin();
            phase = (phase + step) % 1.0;
            if let Some(z) = acc.push(kw_l.tick(input), 0.0) {
                zsum += z;
                n += 1;
            }
        }
        let lufs = energy_to_lufs(zsum / n as f64);
        let expected = -3.01;
        assert!(
            (lufs - expected).abs() < 0.5,
            "1kHz full-scale LUFS = {lufs} (expected ≈ {expected})"
        );
    }

    #[test]
    fn silence_freezes_gain() {
        let mut n = LoudnessNormalizer::new(44100.0);
        let g0 = n.gain();
        for _ in 0..(44100.0 * 2.0) as usize {
            n.push_frame(0.0, 0.0);
        }
        assert_eq!(n.gain(), g0, "gain must be frozen during silence");
    }
}
