/// Pan (声像)
///
/// 等功率 pan law: left = cos(θ), right = sin(θ), θ = pan/127 × π/2
/// XG Spec: 数据值 0 选择"随机声像", 每个 Key On 应用随机 pan
///
/// sin/cos 用 4096 项 SINE_TABLE 查表加速:
///   sin(θ) → idx = θ/2π × 4096; cos(θ) = sin(θ+π/2)
///
/// 对齐说明:
/// - Part 08 pp 0E (MultiPart.pan, 0=random, 64=center)
/// - 鼓: DrumSetup 3n rr 04 (per-note pan), 待鼓路径接入
use crate::fast_sine::{fast_cos, fast_sin};
use crate::utils::random_xorshift;

#[derive(Debug)]
pub struct Pan {
    /// 左声道增益
    pub left: f32,
    /// 右声道增益
    pub right: f32,
    /// 随机数状态 (pan=0 时)
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

    /// 设置声像: 0 = random, 1-127 → L63..C..R63
    pub fn set(&mut self, pan: u8) {
        if pan == 0 {
            self.random();
            return;
        }
        let t = (pan as f32 - 1.0) / 126.0; // 0..1 (1=最左, 127=最右)
        let theta = t * std::f32::consts::FRAC_PI_2;
        self.left = fast_cos(theta);
        self.right = fast_sin(theta);
    }

    /// 随机声像 (XG: pan=0)
    pub fn random(&mut self) {
        let theta = random_xorshift(&mut self.random_state) * std::f32::consts::FRAC_PI_2;
        self.left = fast_cos(theta);
        self.right = fast_sin(theta);
    }

    /// 应用声像到单声道信号 → (L, R)
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
        pan.set(1); // 最左
        assert!((pan.left - 1.0).abs() < 1e-5);
        assert!(pan.right.abs() < 1e-5);
        pan.set(127); // 最右
        assert!(pan.left.abs() < 1e-5);
        assert!((pan.right - 1.0).abs() < 1e-5);
    }

    #[test]
    fn random_pan() {
        let mut pan = Pan::new();
        pan.set(0); // random
        let (l, r) = pan.apply(1.0);
        // 等功率: l² + r² ≈ 1
        let power = l * l + r * r;
        assert!((power - 1.0).abs() < 1e-3, "power={power}");
    }
}
