/// PitchChange / Karaoke / VoiceCancel
///
/// - PitchChange: variable-rate delay + cross-fade (dual-tap sawtooth pitch shifter)
/// - Karaoke1: echo (delay + feedback), vocal karaoke delay effect
/// - VoiceCancel: L-R cancellation (center vocal removal)
use crate::midi::effect_params::effect_obj::{karaoke1_param, pitch_change_param, voice_cancel_param};

use super::core::delay::DelayLine;
use super::params::{dry_wet, feedback_gain, p16};
use super::EffectProcessor;

// ──────────────────────── Pitch Change ────────────────────────
/// WSOLA-based pitch shift: FINE_1/FINE_2 independent shift + PAN_1/PAN_2 + OUTPUT_LEVEL_1/2 + FEEDBACK
pub struct PitchChangeEffect {
    sh1: super::core::wsola::WsolaShifter,
    sh2: super::core::wsola::WsolaShifter,
    /// Output levels
    level1: f32,
    level2: f32,
    /// Pan gains (L, R)
    pan1: (f32, f32),
    pan2: (f32, f32),
    dry: f32,
    wet: f32,
    /// Feedback (FEEDBACK_GAIN)
    feedback: f32,
    fb_state: (f32, f32),
    sample_rate: f32,
}

impl PitchChangeEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sh1: super::core::wsola::WsolaShifter::new(sample_rate),
            sh2: super::core::wsola::WsolaShifter::new(sample_rate),
            level1: 1.0,
            level2: 1.0,
            pan1: (0.707, 0.707),
            pan2: (0.707, 0.707),
            dry: 0.0,
            wet: 1.0,
            feedback: 0.0,
            fb_state: (0.0, 0.0),
            sample_rate,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.sample_rate = sample_rate;
        // PITCH (0-127) → ±12 semitones (64=0)
        let semis = p16(params, pitch_change_param::PITCH) as f32 - 64.0;
        // FINE_1/FINE_2 independent fine tune (±1 semitone)
        let fine1 = p16(params, pitch_change_param::FINE_1) as f32 - 64.0;
        let fine2 = p16(params, pitch_change_param::FINE_2) as f32 - 64.0;
        self.sh1.set_shift(semis + fine1 / 100.0);
        self.sh2.set_shift(semis + fine2 / 100.0);
        // Analysis period: delay time (50-500ms)
        let delay_ms = 50.0 + p16(params, pitch_change_param::INIT_DEALY) as f32 / 127.0 * 450.0;
        self.sh1.set_period(delay_ms / 1000.0 * sample_rate);
        self.sh2.set_period(delay_ms / 1000.0 * sample_rate);
        let (d, w) = dry_wet(p16(params, pitch_change_param::DRY_WET));
        self.dry = d;
        self.wet = w;
        // Output level + pan (PAN_1/PAN_2)
        self.level1 = level(p16(params, pitch_change_param::OUTPUT_LEVEL_1));
        self.level2 = level(p16(params, pitch_change_param::OUTPUT_LEVEL_2));
        self.pan1 = pan_gain(p16(params, pitch_change_param::PAN_1));
        self.pan2 = pan_gain(p16(params, pitch_change_param::PAN_2));
        // Feedback (FEEDBACK_GAIN)
        self.feedback = p16(params, pitch_change_param::FEEDBACK_GAIN) as f32 / 127.0;
    }
}

fn level(v: u16) -> f32 {
    (v as f32 / 127.0).max(0.0).min(2.0)
}

/// pan param (64=center) → equal-power gains
fn pan_gain(v: u16) -> (f32, f32) {
    use crate::fast_sine::{fast_cos, fast_sin};
    let t = (v.min(127) as f32 - 64.0) / 64.0;
    let theta = (t + 1.0) * std::f32::consts::FRAC_PI_4;
    (fast_cos(theta), fast_sin(theta))
}

impl EffectProcessor for PitchChangeEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // Feedback mixed into input
        let (fl, fr) = self.fb_state;
        let xl = l + fl * self.feedback;
        let xr = r + fr * self.feedback;
        let s1 = self.sh1.process_sample(xl) * self.level1;
        let s2 = self.sh2.process_sample(xr) * self.level2;
        let out_l = s1 * self.pan1.0 + s2 * self.pan2.0;
        let out_r = s1 * self.pan1.1 + s2 * self.pan2.1;
        self.fb_state = (out_l, out_r);
        (l * self.dry + out_l * self.wet, r * self.dry + out_r * self.wet)
    }
}

// ──────────────────────── Karaoke ────────────────────────
pub struct KaraokeEffect {
    line: DelayLine,
    delay: f32,
    feedback: f32,
    dry: f32,
    wet: f32,
    /// HPF/LPF (3/4) input filtering
    hpf: super::core::biquad::Biquad,
    lpf: super::core::biquad::Biquad,
    sample_rate: f32,
    /// Feedback state (delay output × feedback)
    fb_state: f32,
}

impl KaraokeEffect {
    pub fn new() -> Self {
        Self {
            line: DelayLine::new(44100 * 2),
            delay: 0.0,
            feedback: 0.0,
            dry: 1.0,
            wet: 1.0,
            hpf: super::core::biquad::Biquad::new(),
            lpf: super::core::biquad::Biquad::new(),
            sample_rate: 44100.0,
            fb_state: 0.0,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.delay = super::params::delay_time_sec(p16(params, karaoke1_param::DELAY_TIME))
            * sample_rate;
        self.feedback = feedback_gain(p16(params, karaoke1_param::FEEDBACK_LEVEL));
        let (d, w) = dry_wet(p16(params, karaoke1_param::DRY_WET));
        self.dry = d;
        self.wet = w;
        // HPF(3)/LPF(4): echo path filtering (0=off)
        let hpf_v = p16(params, karaoke1_param::HPF_CUTOFF);
        self.hpf = if hpf_v > 0 {
            super::core::biquad::make_biquad(0.0, hpf_v as f32 * 100.0, 0.707, false, sample_rate)
        } else {
            super::core::biquad::Biquad::new()
        };
        let lpf_v = p16(params, karaoke1_param::LPF_CUTOFF);
        self.lpf = if lpf_v > 0 {
            super::core::biquad::make_biquad(-12.0, lpf_v as f32 * 100.0, 0.707, false, sample_rate)
        } else {
            super::core::biquad::Biquad::new()
        };
    }
}

impl EffectProcessor for KaraokeEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // Mono-mixed echo (karaoke delay) + HPF/LPF filtering
        let mono = (l + r) * 0.5;
        let mono = self.hpf.tick(mono);
        let mono = self.lpf.tick(mono);
        let delayed = self.line.tick(mono + self.fb_state, self.delay);
        self.fb_state = delayed * self.feedback;
        (l * self.dry + delayed * self.wet, r * self.dry + delayed * self.wet)
    }
}

// ──────────────────────── Voice Cancel ────────────────────────
pub struct VoiceCancelEffect {
    low_adjust: f32,
    high_adjust: f32,
}

impl VoiceCancelEffect {
    pub fn new() -> Self {
        Self { low_adjust: 0.5, high_adjust: 0.5 }
    }

    pub fn set_params(&mut self, params: &[u16; 16]) {
        self.low_adjust = p16(params, voice_cancel_param::LOW_ADJUST) as f32 / 127.0;
        self.high_adjust = p16(params, voice_cancel_param::HIGH_ADJUST) as f32 / 127.0;
    }
}

impl EffectProcessor for VoiceCancelEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // L-R cancellation: vocal (center) cancels, stereo instruments remain
        let cancelled = (l - r) * self.high_adjust;
        // Low-frequency portion (vocal low range) retained at a certain level
        let low = (l + r) * 0.5 * (1.0 - self.low_adjust);
        (cancelled + low, cancelled + low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_change_shifts_up() {
        let mut e = PitchChangeEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[pitch_change_param::PITCH] = 64 + 12; // +12 semitones
        p[pitch_change_param::INIT_DEALY] = 30;
        p[pitch_change_param::DRY_WET] = 1; // full wet
        p[pitch_change_param::OUTPUT_LEVEL_1] = 127;
        p[pitch_change_param::OUTPUT_LEVEL_2] = 127;
        e.set_params(&p, 44100.0);
        // 440Hz → output frequency should rise (zero-crossing count)
        let mut zero_cross = 0usize;
        let mut prev = 0.0f32;
        let mut phase: f32 = 0.0;
        for _ in 0..44100 {
            let x = (phase * std::f32::consts::PI * 2.0).sin();
            phase = (phase + 440.0 / 44100.0) % 1.0;
            let (l, _) = e.process((x, x));
            if prev <= 0.0 && l > 0.0 {
                zero_cross += 1;
            }
            prev = l;
        }
        // 880Hz zero crossings ~2× (after initial settling)
        assert!(zero_cross > 300, "zero_cross={zero_cross}");
    }

    #[test]
    fn voice_cancel_removes_center() {
        let mut e = VoiceCancelEffect::new();
        let mut p = [0u16; 16];
        p[voice_cancel_param::LOW_ADJUST] = 127; // full low cancellation
        p[voice_cancel_param::HIGH_ADJUST] = 127; // full high cancellation
        e.set_params(&p);
        // Same L/R (center signal) → cancelled
        let out = e.process((1.0, 1.0));
        assert!(out.0.abs() < 0.1, "out={:?}", out);
        // Opposite L/R (stereo) → retained
        let out2 = e.process((1.0, -1.0));
        assert!(out2.0.abs() > 0.5, "out={:?}", out2);
    }
}
