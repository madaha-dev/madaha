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
