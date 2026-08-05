/// XG effect parameter conversion
///
/// Unified entry: effect param array `[u16; 16]` + param const indices from effect_obj.rs
/// → physical values (Hz / seconds / gain / 0-1, etc.)
use crate::midi::effect_params::parameter_table::{
    XG_DELAY_TIME_200MS_TABLE, XG_DRY_WET, XG_FEEDBACK_LEVEL_REVERB_DELAY_FLANGER, XG_LEVEL,
    XG_LFO_FREQ_TABLE, XG_REVERB_TIME_TABLE,
};

/// Get one entry from the 16-entry param array (out of range → 0)
#[inline]
pub fn p16(params: &[u16; 16], id: usize) -> u16 {
    params.get(id).copied().unwrap_or(0) & 0x7F
}

/// dB → linear gain
#[inline]
pub fn db_to_lin(db: f32) -> f32 {
    if db.is_infinite() {
        if db.is_sign_negative() {
            0.0
        } else {
            1.0
        }
    } else {
        10f32.powf(db / 20.0)
    }
}

/// XG_LEVEL (0-127, dB table) → linear
#[inline]
pub fn level_gain(v: u16) -> f32 {
    db_to_lin(XG_LEVEL[(v.min(127)) as usize])
}

/// XG feedback level table (0-127, dB) → linear (0=no feedback, 127=max, clamp ≤0.99 prevents divergence)
#[inline]
pub fn feedback_gain(v: u16) -> f32 {
    if v == 0 {
        return 0.0;
    }
    db_to_lin(XG_FEEDBACK_LEVEL_REVERB_DELAY_FLANGER[(v.min(127)) as usize]).clamp(0.0, 0.99)
}

/// LFO frequency (0-127) → Hz (XG Spec Table #1)
#[inline]
pub fn lfo_freq(v: u16) -> f32 {
    XG_LFO_FREQ_TABLE[v.min(127) as usize]
}

/// DRY/WET (0-127) → (dry, wet) linear
/// v=1 full wet, v=127 full dry; 64 = MIN_POSITIVE sentinel (param unused → passthrough)
#[inline]
pub fn dry_wet(v: u16) -> (f32, f32) {
    let (d, w) = XG_DRY_WET[v.min(127) as usize];
    (db_to_lin_sentinel(d), db_to_lin_sentinel(w))
}

/// db conversion (MIN_POSITIVE sentinel → 0dB passthrough)
#[inline]
fn db_to_lin_sentinel(db: f32) -> f32 {
    if db > 0.0 && db < 1e-30 {
        1.0
    } else {
        db_to_lin(db)
    }
}

/// Reverb time param (0-69) → seconds (XG Spec Table #4)
#[inline]
pub fn reverb_time_sec(v: u16) -> f32 {
    XG_REVERB_TIME_TABLE[(v.min(69)) as usize]
}

/// Delay time (200ms mode table, 0-127) → seconds
#[inline]
pub fn delay_time_sec(v: u16) -> f32 {
    XG_DELAY_TIME_200MS_TABLE[v.min(127) as usize] / 1000.0
}

/// Delay time param → sample count
#[inline]
pub fn delay_time_samples(v: u16, sample_rate: f32) -> usize {
    (delay_time_sec(v) * sample_rate).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_and_feedback() {
        assert!((level_gain(127) - 1.0).abs() < 1e-5);
        assert_eq!(level_gain(0), 0.0);
        assert_eq!(feedback_gain(0), 0.0);
        let f127 = feedback_gain(127);
        assert!((f127 - 0.99).abs() < 1e-6, "f127={f127}"); // clamp max
        let f64 = feedback_gain(64);
        assert!(f64 > 0.5 && f64 < 1.0, "f64={f64}");
    }

    #[test]
    fn dry_wet_endpoints() {
        // 127 → full dry (XG table: higher value = drier)
        let (d, w) = dry_wet(127);
        assert!((d - 1.0).abs() < 1e-4, "dry={d}");
        assert!(w < 0.1, "wet={w}");
        // 1 → full wet
        let (d, w) = dry_wet(1);
        assert!(d < 0.1, "dry={d}");
        assert!((w - 1.0).abs() < 1e-4, "wet={w}");
        // 64 (sentinel) → passthrough
        let (d, w) = dry_wet(64);
        assert!((d - 1.0).abs() < 1e-4, "dry={d}");
        assert!((w - 1.0).abs() < 1e-4, "wet={w}");
    }

    #[test]
    fn time_tables() {
        assert!((reverb_time_sec(0) - 0.3).abs() < 1e-5);
        assert!((reverb_time_sec(69) - 30.0).abs() < 1e-5);
        assert!(delay_time_sec(0) > 0.0);
    }
}
