use serde::Deserialize;
use strum_macros::EnumString;

use crate::fast_sine::SINE_TABLE;

#[derive(Debug, Deserialize, EnumString, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InterpolatingMethods {
    /// Low cpu usage
    Linear,

    /// Mid cpu usage
    Hermite,

    /// High cpu usage
    #[serde(alias = "lanczos", alias = "lanczos-3")]
    Lanczos3,
}

impl InterpolatingMethods {
    /// 从 (loop_point, loop_length) 已知的采样中读取第 idx 个样本。
    /// - 循环采样: 超出循环区末端时折回循环区内
    /// - 一次性采样: 超出末尾返回 0, 负索引夹到 0
    #[inline]
    fn sample_at(pcm: &[f32], loop_point: usize, loop_length: usize, idx: i64) -> f32 {
        let len = pcm.len() as i64;
        if idx < 0 {
            return pcm[0];
        }
        let idx = if loop_length > 0 && idx >= (loop_point + loop_length) as i64 {
            loop_point as i64 + (idx - loop_point as i64) % loop_length as i64
        } else {
            idx
        };
        if idx >= len {
            return 0.0;
        }
        pcm[idx as usize]
    }

    /// 在 pos（f64, 样本单位）处插值采样。
    pub fn interpolate(
        &self,
        pcm: &[f32],
        loop_point: usize,
        loop_length: usize,
        pos: f64,
    ) -> f32 {
        let i = pos.floor() as i64;
        let f = (pos - i as f64) as f32;
        match self {
            InterpolatingMethods::Linear => {
                let s0 = Self::sample_at(pcm, loop_point, loop_length, i);
                let s1 = Self::sample_at(pcm, loop_point, loop_length, i + 1);
                s0 * (1.0 - f) + s1 * f
            }
            InterpolatingMethods::Hermite => {
                let p0 = Self::sample_at(pcm, loop_point, loop_length, i - 1);
                let p1 = Self::sample_at(pcm, loop_point, loop_length, i);
                let p2 = Self::sample_at(pcm, loop_point, loop_length, i + 1);
                let p3 = Self::sample_at(pcm, loop_point, loop_length, i + 2);
                // Catmull-Rom 四点插值
                let c1 = 0.5 * (p2 - p0);
                let c2 = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
                let c3 = 0.5 * (p3 - p0) + 1.5 * (p1 - p2);
                ((c3 * f + c2) * f + c1) * f + p1
            }
            InterpolatingMethods::Lanczos3 => {
                // 6 taps: i-2 .. i+3, 核宽度 a=3
                let mut acc = 0.0;
                for k in -2i64..=3 {
                    let x = (k as f32) - f; // 采样点相对 pos 的偏移
                    let w = lanczos_weight(x);
                    acc += Self::sample_at(pcm, loop_point, loop_length, i + k) * w;
                }
                acc
            }
        }
    }
}

/// Lanczos 核: sinc(x) * sinc(x/a), |x| < a, 其余 0
/// sin 用 4096 项 SINE_TABLE 查表加速:
///   sin(πx) → idx = x × 2048 (因 4096/(2π) × π = 2048)
#[inline]
fn lanczos_weight(x: f32) -> f32 {
    const A: f32 = 3.0;
    let ax = x.abs();
    if ax >= A {
        return 0.0;
    }
    if ax < 1e-6 {
        return 1.0; // 避免 0/0
    }
    let idx = (x * 2048.0) as i32;
    let idx_a = idx / 3; // sin(πx/3) 的索引
    let pix = std::f32::consts::PI * x;
    let pix_a = std::f32::consts::PI * x / A;
    let s1 = SINE_TABLE[idx.rem_euclid(4096) as usize] / pix;
    let s2 = SINE_TABLE[idx_a.rem_euclid(4096) as usize] / pix_a;
    s1 * s2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_interp() {
        // 常数信号插值应保持
        let pcm = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let v = InterpolatingMethods::Linear.interpolate(&pcm, 0, 0, 2.5);
        assert!((v - 1.0).abs() < 1e-6);

        // 斜坡 0,1,2,3...: pos=2.5 → 2.5
        let pcm = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let v = InterpolatingMethods::Linear.interpolate(&pcm, 0, 0, 2.5);
        assert!((v - 2.5).abs() < 1e-6);
    }

    #[test]
    fn hermite_interp_exact() {
        // Hermite 在整数点上应精确还原
        let pcm = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let v = InterpolatingMethods::Hermite.interpolate(&pcm, 0, 0, 3.0);
        assert!((v - 4.0).abs() < 1e-6);

        // 常数信号
        let pcm = [2.0; 16];
        let v = InterpolatingMethods::Hermite.interpolate(&pcm, 0, 0, 7.25);
        assert!((v - 2.0).abs() < 1e-4);
    }

    #[test]
    fn lanczos_interp_exact() {
        // Lanczos 在整数点应精确还原（核在整数处 = 0，除 x=0）
        let pcm = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let v = InterpolatingMethods::Lanczos3.interpolate(&pcm, 0, 0, 4.0);
        assert!((v - 5.0).abs() < 1e-6);
    }

    #[test]
    fn loop_wrap() {
        // 循环采样: loop_point=2, loop_length=4 (样本 2..5), pos=6.5 → 折回 2.5
        let pcm = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        // sample_at 直接验证 wrap
        let s = InterpolatingMethods::sample_at(&pcm, 2, 4, 6);
        assert_eq!(s, 2.0); // idx 6 → 2 + (6-2)%4 = 2+0 = 2
        let s = InterpolatingMethods::sample_at(&pcm, 2, 4, 9);
        assert_eq!(s, 5.0); // idx 9 → 2 + (9-2)%4 = 2+3 = 5
    }

    #[test]
    fn one_shot_out_of_range() {
        let pcm = [1.0, 2.0, 3.0];
        // 一次性采样 (loop_length=0): idx 超出 → 0
        let s = InterpolatingMethods::sample_at(&pcm, 0, 0, 3);
        assert_eq!(s, 0.0);
        let s = InterpolatingMethods::sample_at(&pcm, 0, 0, 10);
        assert_eq!(s, 0.0);
    }
}
