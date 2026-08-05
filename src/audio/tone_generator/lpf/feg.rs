/// FEG (Filter Envelope Generator)
///
/// State machine: Attack → Decay → Sustain → Release → Finished
/// Output: level ∈ [0, 1], modulates the cutoff parameter via `filter_eg_depth`
///
/// Alignment notes:
/// - Part 08 pp 1A-1C (EG Attack/Decay/Release Time) applies to AEG/FEG/PEG alike
///   (XG sound source model), so FEG stage times come from the Part parameters (note-on snapshot)
/// - Part 08 pp 71 (Filter EG Depth, 64=center) → modulation depth, default 0 = no effect
use std::time::Duration;

use crate::audio::interface::Audio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FEGStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

#[derive(Debug)]
pub struct FEG {
    pub state: FEGStage,
    /// Current level [0, 1]
    pub level: f32,

    /// EG enable (element[71] eg_enable / [40] eg_filt_en; false → always 0.0)
    pub enabled: bool,

    // ── Stage parameters ──
    pub attack_time: Duration,
    pub decay_time: Duration,
    pub sustain_level: f32,
    pub release_time: Duration,

    // Stage start timestamp
    stage_started: Duration,
    elapsed_total: Duration,
}

impl FEG {
    pub fn new() -> Self {
        Self {
            state: FEGStage::Attack,
            level: 0.0,
            enabled: true,
            attack_time: Duration::from_millis(5),
            decay_time: Duration::from_millis(100),
            sustain_level: 0.5,
            release_time: Duration::from_millis(100),
            stage_started: Duration::ZERO,
            elapsed_total: Duration::ZERO,
        }
    }

    /// Initialize from Part parameters (snapshot at note-on)
    ///
    /// `eg_attack/decay/release`: 08 pp 1A-1C, 64=center, relative time offset from VCE
    /// `feg_depth`: 08 pp 71 Filter EG Depth (64=0)
    pub fn setup(&mut self, eg_attack: u8, eg_decay: u8, eg_release: u8, _feg_depth: u8) {
        self.state = FEGStage::Attack;
        self.level = 0.0;
        self.stage_started = Duration::ZERO;
        self.elapsed_total = Duration::ZERO;

        // Time parameters (relative): 64=center → base time, simply mapped to ms
        // (2006LE uses a rate table; approximate here; to be aligned exactly once the data file is supported)
        self.attack_time = Duration::from_millis(param_to_ms(eg_attack, 5.0) as u64);
        self.decay_time = Duration::from_millis(param_to_ms(eg_decay, 100.0) as u64);
        self.release_time = Duration::from_millis(param_to_ms(eg_release, 100.0) as u64);
        // VCE has no independent sustain level field → default 0.5 (neutral)
        self.sustain_level = 0.5;
    }

    pub fn note_off(&mut self) {
        if matches!(self.state, FEGStage::Finished | FEGStage::Release) {
            return;
        }
        self.state = FEGStage::Release;
        self.stage_started = self.elapsed_total;
    }

    pub fn kill(&mut self) {
        self.state = FEGStage::Finished;
        self.level = 0.0;
    }

    /// Advance the envelope (called once per block), return current level [0,1]
    pub fn tick(&mut self, elapsed: Duration) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        self.elapsed_total += elapsed;
        let t = self.elapsed_total - self.stage_started;

        match self.state {
            FEGStage::Attack => {
                if t >= self.attack_time {
                    self.level = 1.0;
                    self.advance(FEGStage::Decay);
                } else if self.attack_time.is_zero() {
                    self.level = 1.0;
                    self.advance(FEGStage::Decay);
                } else {
                    self.level = (t.as_secs_f32() / self.attack_time.as_secs_f32()).min(1.0);
                }
            }
            FEGStage::Decay => {
                if t >= self.decay_time {
                    self.level = self.sustain_level;
                    self.advance(FEGStage::Sustain);
                } else if self.decay_time.is_zero() {
                    self.level = self.sustain_level;
                    self.advance(FEGStage::Sustain);
                } else {
                    let p = t.as_secs_f32() / self.decay_time.as_secs_f32();
                    self.level = 1.0 - (1.0 - self.sustain_level) * p;
                }
            }
            FEGStage::Sustain => {
                self.level = self.sustain_level;
            }
            FEGStage::Release => {
                if t >= self.release_time || self.release_time.is_zero() {
                    self.level = 0.0;
                    self.advance(FEGStage::Finished);
                } else {
                    let p = t.as_secs_f32() / self.release_time.as_secs_f32();
                    self.level = self.sustain_level * (1.0 - p);
                }
            }
            FEGStage::Finished => {
                self.level = 0.0;
            }
        }

        self.level
    }

    fn advance(&mut self, next: FEGStage) {
        self.state = next;
        self.stage_started = self.elapsed_total;
    }
}

impl Audio for FEG {
    fn tick(&mut self, elapsed: Duration) -> f32 {
        self.tick(elapsed)
    }
}

/// 08 pp 1A-1C relative time parameter → milliseconds
/// 64=center (base), 0=fastest, 127=slowest
fn param_to_ms(param: u8, base_ms: f32) -> f32 {
    let off = param as f32 - 64.0;
    // About ×1.2 per unit (logarithmic), range ±~200x
    (base_ms * 1.2f32.powf(off)).clamp(0.1, 20000.0)
}

#[cfg(test)]
mod tests {
    use super::super::{CutOff, LPF};
    use super::*;

    fn step_ms(feg: &mut FEG, ms: u64) -> f32 {
        feg.tick(Duration::from_millis(ms))
    }

    #[test]
    fn feg_attack_decay_sustain() {
        let mut feg = FEG::new();
        feg.setup(0x40, 0x40, 0x40, 0x40); // 64 center: attack 5ms, decay 100ms
        // attack reaches 1.0
        let level = step_ms(&mut feg, 10);
        assert!((level - 1.0).abs() < 1e-4, "attack end = {level}");
        assert_eq!(feg.state, FEGStage::Decay);
        // decay to sustain (0.5)
        step_ms(&mut feg, 500);
        assert!((feg.level - 0.5).abs() < 1e-4, "sustain = {}", feg.level);
        assert_eq!(feg.state, FEGStage::Sustain);
    }

    #[test]
    fn feg_release() {
        let mut feg = FEG::new();
        feg.setup(0x40, 0x40, 0x40, 0x40);
        step_ms(&mut feg, 1000); // to sustain
        feg.note_off();
        let level = step_ms(&mut feg, 500);
        assert!((level - 0.0).abs() < 1e-4, "release end = {level}");
        assert_eq!(feg.state, FEGStage::Finished);
    }

    #[test]
    fn cutoff_compute() {
        let co = CutOff::new();
        // base 64, no modulation → Hz of parameter 64
        let hz0 = co.compute_hz(0.0, 0.0);
        assert!((hz0 - LPF::cutoff_param_to_hz(64)).abs() < 0.1);
        // FEG level 1 × depth 64 → param 128 → clamp 127
        let co2 = CutOff {
            base: 64.0,
            feg_depth: 64.0,
            ..CutOff::new()
        };
        let hz1 = co2.compute_hz(1.0, 0.0);
        assert!((hz1 - LPF::cutoff_param_to_hz(127)).abs() < 0.1);
    }
}
