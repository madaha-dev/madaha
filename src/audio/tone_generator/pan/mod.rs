/// Pan
///
/// Equal-power pan law: left = cos(θ), right = sin(θ), θ = pan/127 × π/2
/// XG Spec: data value 0 selects "random pan", a random pan is applied on every Key On
///
/// sin/cos accelerated by a 4096-entry SINE_TABLE lookup:
///   sin(θ) → idx = θ/2π × 4096; cos(θ) = sin(θ+π/2)
///
/// Alignment notes:
/// - Part 08 pp 0E (MultiPart.pan, 0=random, 64=center)
/// - Drum: DrumSetup 3n rr 04 (per-note pan), to be wired into the drum path
use crate::fast_sine::{fast_cos, fast_sin};
use crate::utils::random_xorshift;

#[derive(Debug)]
pub struct Pan {
    /// Left channel gain
    pub left: f32,
    /// Right channel gain
    pub right: f32,
    /// Random number state (when pan=0)
    random_state: u32,
}

impl Pan {
    pub fn new() -> Self {
        Self {
            left: 0.707_106_8, // cos(π/4) = center
            right: 0.707_106_8,
            random_state: 0x1234_5678,
        }
    }

    /// Set pan: 0 = random, 1-127 → L63..C..R63
    pub fn set(&mut self, pan: u8) {
        if pan == 0 {
            self.random();
            return;
        }
        let t = (pan as f32 - 1.0) / 126.0; // 0..1 (1=far left, 127=far right)
        let theta = t * std::f32::consts::FRAC_PI_2;
        self.left = fast_cos(theta);
        self.right = fast_sin(theta);
    }

    /// Random pan (XG: pan=0)
    pub fn random(&mut self) {
        let theta = random_xorshift(&mut self.random_state) * std::f32::consts::FRAC_PI_2;
        self.left = fast_cos(theta);
        self.right = fast_sin(theta);
    }

    /// Apply pan to a mono signal → (L, R)
    #[inline]
    pub fn apply(&self, mono: f32) -> (f32, f32) {
        (mono * self.left, mono * self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_pan() {
        let pan = Pan::new();
        assert!((pan.left - 0.7071068).abs() < 1e-5);
        assert!((pan.right - 0.7071068).abs() < 1e-5);
    }

    #[test]
    fn hard_left_right() {
        let mut pan = Pan::new();
        pan.set(1); // far left
        assert!((pan.left - 1.0).abs() < 1e-5);
        assert!(pan.right.abs() < 1e-5);
        pan.set(127); // far right
        assert!(pan.left.abs() < 1e-5);
        assert!((pan.right - 1.0).abs() < 1e-5);
    }

    #[test]
    fn random_pan() {
        let mut pan = Pan::new();
        pan.set(0); // random
        let (l, r) = pan.apply(1.0);
        // Equal power: l² + r² ≈ 1
        let power = l * l + r * r;
        assert!((power - 1.0).abs() < 1e-3, "power={power}");
    }
}
