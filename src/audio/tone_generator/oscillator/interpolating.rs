use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use crate::fast_sine::SINE_TABLE;

#[derive(Debug, Deserialize, EnumString, PartialEq, Clone, Copy, Serialize)]
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
    /// Read the idx-th sample from a sample with known (loop_point, loop_length).
    /// - Looping sample: wraps back into the loop region when past its end
    /// - One-shot sample: returns 0 past the end, clamps negative indices to 0
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

    /// Interpolate the sample at pos (f64, in samples).
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
                // Catmull-Rom four-point interpolation
                let c1 = 0.5 * (p2 - p0);
                let c2 = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
                let c3 = 0.5 * (p3 - p0) + 1.5 * (p1 - p2);
                ((c3 * f + c2) * f + c1) * f + p1
            }
            InterpolatingMethods::Lanczos3 => {
                // 6 taps: i-2 .. i+3, kernel width a=3
                let mut acc = 0.0;
                for k in -2i64..=3 {
                    let x = (k as f32) - f; // sample point offset relative to pos
                    let w = lanczos_weight(x);
                    acc += Self::sample_at(pcm, loop_point, loop_length, i + k) * w;
                }
                acc
            }
        }
    }
}

/// Lanczos kernel: sinc(x) * sinc(x/a), |x| < a, 0 otherwise
/// sin accelerated by a 4096-entry SINE_TABLE lookup:
///   sin(πx) → idx = x × 2048 (since 4096/(2π) × π = 2048)
#[inline]
fn lanczos_weight(x: f32) -> f32 {
    const A: f32 = 3.0;
    let ax = x.abs();
    if ax >= A {
        return 0.0;
    }
    if ax < 1e-6 {
        return 1.0; // avoid 0/0
    }
    let idx = (x * 2048.0) as i32;
    let idx_a = idx / 3; // index for sin(πx/3)
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
        // Constant signal interpolation should hold
        let pcm = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let v = InterpolatingMethods::Linear.interpolate(&pcm, 0, 0, 2.5);
        assert!((v - 1.0).abs() < 1e-6);

        // Ramp 0,1,2,3...: pos=2.5 → 2.5
        let pcm = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let v = InterpolatingMethods::Linear.interpolate(&pcm, 0, 0, 2.5);
        assert!((v - 2.5).abs() < 1e-6);
    }

    #[test]
    fn hermite_interp_exact() {
        // Hermite should exactly reconstruct at integer points
        let pcm = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let v = InterpolatingMethods::Hermite.interpolate(&pcm, 0, 0, 3.0);
        assert!((v - 4.0).abs() < 1e-6);

        // Constant signal
        let pcm = [2.0; 16];
        let v = InterpolatingMethods::Hermite.interpolate(&pcm, 0, 0, 7.25);
        assert!((v - 2.0).abs() < 1e-4);
    }

    #[test]
    fn lanczos_interp_exact() {
        // Lanczos should exactly reconstruct at integer points (kernel = 0 at integers except x=0)
        let pcm = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let v = InterpolatingMethods::Lanczos3.interpolate(&pcm, 0, 0, 4.0);
        assert!((v - 5.0).abs() < 1e-6);
    }

    #[test]
    fn loop_wrap() {
        // Looping sample: loop_point=2, loop_length=4 (samples 2..5), pos=6.5 → wraps to 2.5
        let pcm = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        // sample_at directly verifies wrap
        let s = InterpolatingMethods::sample_at(&pcm, 2, 4, 6);
        assert_eq!(s, 2.0); // idx 6 → 2 + (6-2)%4 = 2+0 = 2
        let s = InterpolatingMethods::sample_at(&pcm, 2, 4, 9);
        assert_eq!(s, 5.0); // idx 9 → 2 + (9-2)%4 = 2+3 = 5
    }

    #[test]
    fn one_shot_out_of_range() {
        let pcm = [1.0, 2.0, 3.0];
        // One-shot sample (loop_length=0): idx out of range → 0
        let s = InterpolatingMethods::sample_at(&pcm, 0, 0, 3);
        assert_eq!(s, 0.0);
        let s = InterpolatingMethods::sample_at(&pcm, 0, 0, 10);
        assert_eq!(s, 0.0);
    }
}
