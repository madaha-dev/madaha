//! Master-bus peak limiter.
//!
//! Fast-attack / slow-release peak limiter that tames hot transients (e.g. a
//! loud single voice whose restored low-frequency energy pushes the master bus
//! over ±1.0) without touching normal dynamics. Below `threshold` it is
//! transparent (unity gain), so it does not change calibrated loudness.

pub struct MasterLimiter {
    /// Current gain reduction (0..=1), recovered toward 1.0 on release.
    gain: f32,
    /// Peak level the limiter holds the output to.
    threshold: f32,
    /// One-pole release coefficient toward unity gain.
    release_coef: f32,
}

impl MasterLimiter {
    pub fn new(sample_rate: f32) -> Self {
        let release_seconds = 0.020;
        Self {
            gain: 1.0,
            // 用户选择 0.85（更早介入、更安全；密集处音头保护性压仍存在，
            // 缓解来自输出侧响度 AGC 的回补而非移除限幅）。
            threshold: 0.85,
            release_coef: (-1.0 / (sample_rate * release_seconds)).exp(),
        }
    }

    pub fn process(&mut self, l: f32, r: f32) -> (f32, f32) {
        let peak = l.abs().max(r.abs());
        if peak > 0.0 {
            let target = self.threshold / peak;
            // Instant attack (immediate gain drop), one-pole release.
            if target < self.gain {
                self.gain = target;
            } else {
                self.gain += (1.0 - self.gain) * (1.0 - self.release_coef);
            }
        } else {
            self.gain += (1.0 - self.gain) * (1.0 - self.release_coef);
        }
        (l * self.gain, r * self.gain)
    }

    /// Current gain reduction (0..=1); below threshold it returns to 1.0.
    pub fn gain(&self) -> f32 {
        self.gain
    }
}
