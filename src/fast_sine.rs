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

/// 4096 / 2π — precomputed index scale. The per-call f32 division was slow on
/// this machine; multiplication by a folded constant keeps it at one multiply.
const TABLE_SCALE: f32 = 651.898_646_9;

/// Table-lookup sin: any angle (radians), wraps internally
/// 4096-entry full-period table, index = θ × (4096/2π) & 0xFFF
#[inline]
pub fn fast_sin(theta: f32) -> f32 {
    let idx = (theta * TABLE_SCALE) as usize;
    SINE_TABLE[idx & 0xFFF]
}

/// Table-lookup cos: cos(θ) = sin(θ + π/2)
#[inline]
pub fn fast_cos(theta: f32) -> f32 {
    let idx = ((theta + std::f32::consts::FRAC_PI_2) * TABLE_SCALE) as usize;
    SINE_TABLE[idx & 0xFFF]
}
