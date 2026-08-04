use std::sync::LazyLock;

pub const SINE_TABLE: LazyLock<[f32; 4096]> = LazyLock::new(|| {
    let mut table = [0.0; 4096];
    let mut i = 0;
    while i < 4096 {
        table[i] = ((i as f64) * 2.0 * std::f64::consts::PI / 4096.0).sin() as f32;
        i += 1;
    }
    table
});

/// 查表 sin: 任意角度 (弧度), 内部自动回卷
/// 4096 项全周期表, 索引 = θ/2π × 4096 & 0xFFF
#[inline]
pub fn fast_sin(theta: f32) -> f32 {
    let idx = (theta / (2.0 * std::f32::consts::PI) * 4096.0) as usize;
    SINE_TABLE[idx & 0xFFF]
}

/// 查表 cos: cos(θ) = sin(θ + π/2)
#[inline]
pub fn fast_cos(theta: f32) -> f32 {
    let idx =
        ((theta + std::f32::consts::FRAC_PI_2) / (2.0 * std::f32::consts::PI) * 4096.0) as usize;
    SINE_TABLE[idx & 0xFFF]
}
