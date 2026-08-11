/// AEG (Amplitude Envelope Generator)
///
/// State machine: Attack → Decay → Sustain → Release → Finished
/// Output: level ∈ [0, 1] used directly as volume gain
///
/// Alignment notes:
/// - Part 08 pp 1A-1C (EG Attack/Decay/Release Time) applies to AEG/FEG/PEG alike
///   (XG sound source model), stage times taken from Part parameters (note-on snapshot)
/// - Sustain level: no independent field in Part 08 pp 18? → fixed 0.7 (approximation),
///   to be aligned exactly once the 2006LE data file is supported
use std::time::Duration;

use crate::audio::interface::Audio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AEGStage {
    Attack,
    Decay,
    Sustain,
    /// Damper-hold stage: entered from Sustain while the sustain pedal is
    /// held with a damper policy active (S-YXG50 acoustic-piano programs or
    /// 2006LE sustain_mode=2). The level slowly decays to zero — the pedal
    /// holds the note but the string energy fades (real piano behavior).
    Damp,
    Release,
    Finished,
}

#[derive(Debug)]
pub struct AEG {
    pub state: AEGStage,
    /// Current level [0, 1]
    pub level: f32,

    /// EG enable (element[71] eg_enable / [41] eg_amp_en; false → always 1.0)
    pub enabled: bool,

    pub attack_time: Duration,
    pub decay_time: Duration,
    pub sustain_level: f32,
    pub release_time: Duration,

    /// Damper-hold stage time (approximation; real piano ~2-5 s). To be
    /// aligned once the 2006LE data file is supported.
    pub damp_time: Duration,
    /// Damper flag: when set, Sustain decays through the Damp stage instead
    /// of freezing at sustain_level.
    damper: bool,
    /// NoteOff arrived during the protected Attack/Decay stages: the envelope
    /// finishes Attack → Decay → Sustain first, then releases (XG short-note
    /// behavior — a very short key press still sounds the full attack+decay,
    /// e.g. musicbox hammer strike + bell body before the release tail).
    pending_release: bool,
    /// Level at the start of the Damp stage (sustain_level when entered)
    damp_start_level: f32,
    /// Level at the start of the Release stage (note_off snapshot)
    release_start_level: f32,

    /// 1/time (per second) — precomputed so the per-block tick only multiplies
    /// (f32 division per tick was slow on this machine)
    inv_attack: f32,
    inv_decay: f32,
    inv_release: f32,
    inv_damp: f32,

    stage_started: Duration,
    elapsed_total: Duration,
}

impl AEG {
    pub fn new() -> Self {
        Self {
            state: AEGStage::Attack,
            level: 0.0,
            enabled: true,
            attack_time: Duration::from_millis(5),
            decay_time: Duration::from_millis(100),
            sustain_level: 0.7,
            release_time: Duration::from_millis(100),
            damp_time: Duration::from_secs(3),
            damper: false,
            pending_release: false,
            damp_start_level: 0.0,
            release_start_level: 0.0,
            inv_attack: 1.0 / 0.005,
            inv_decay: 1.0 / 0.1,
            inv_release: 1.0 / 0.1,
            inv_damp: 1.0 / 3.0,
            stage_started: Duration::ZERO,
            elapsed_total: Duration::ZERO,
        }
    }

    /// Initialize from Part parameters (snapshot at note-on)
    /// `eg_attack/decay/release`: 08 pp 1A-1C, 64=center.
    /// Attack center = 1ms (XG neutral attack must stay sharp enough to pass
    /// percussion transients — musicbox hammer strike lives in the first ~1ms
    /// of the element sample; a 5ms center faded it to ~18%).
    pub fn setup(&mut self, eg_attack: u8, eg_decay: u8, eg_release: u8) {
        self.state = AEGStage::Attack;
        self.level = 0.0;
        self.stage_started = Duration::ZERO;
        self.elapsed_total = Duration::ZERO;
        self.attack_time = Duration::from_millis(param_to_ms(eg_attack, 1.0) as u64);
        self.decay_time = Duration::from_millis(param_to_ms(eg_decay, 100.0) as u64);
        self.release_time = Duration::from_millis(param_to_ms(eg_release, 100.0) as u64);
        self.inv_attack = inv_secs(self.attack_time);
        self.inv_decay = inv_secs(self.decay_time);
        self.inv_release = inv_secs(self.release_time);
        self.sustain_level = 0.7;
        self.damper = false;
        self.pending_release = false;
        self.damp_start_level = 0.0;
        self.release_start_level = 0.0;
    }

    /// Enable/disable the damper-hold stage. While enabled, the envelope
    /// decays through `Damp` instead of freezing at sustain_level.
    pub fn set_damper(&mut self, on: bool) {
        self.damper = on;
    }

    pub fn note_off(&mut self) {
        if matches!(self.state, AEGStage::Finished | AEGStage::Release) {
            return;
        }
        if matches!(self.state, AEGStage::Attack | AEGStage::Decay) {
            // Protected stages: finish Attack → Decay → Sustain first, then
            // release (XG short-note behavior). Without this, a very short
            // key press cuts the attack/decay and the note barely sounds.
            self.pending_release = true;
            return;
        }
        self.release_start_level = self.level;
        self.state = AEGStage::Release;
        self.stage_started = self.elapsed_total;
    }

    pub fn kill(&mut self) {
        self.state = AEGStage::Finished;
        self.level = 0.0;
    }

    /// Advance the envelope (called once per block), return current level [0,1]
    pub fn tick(&mut self, elapsed: Duration) -> f32 {
        if !self.enabled {
            return 1.0;
        }
        self.elapsed_total += elapsed;
        let t = self.elapsed_total - self.stage_started;

        match self.state {
            AEGStage::Attack => {
                if self.attack_time.is_zero() || t >= self.attack_time {
                    self.level = 1.0;
                    self.advance(AEGStage::Decay);
                } else {
                    self.level = (t.as_secs_f32() * self.inv_attack).min(1.0);
                }
            }
            AEGStage::Decay => {
                if self.decay_time.is_zero() || t >= self.decay_time {
                    self.level = self.sustain_level;
                    self.advance(AEGStage::Sustain);
                } else {
                    let p = t.as_secs_f32() * self.inv_decay;
                    self.level = 1.0 - (1.0 - self.sustain_level) * p;
                }
            }
            AEGStage::Sustain => {
                if self.pending_release {
                    // Attack+Decay completed after a short key press: release now.
                    self.pending_release = false;
                    self.release_start_level = self.level;
                    self.advance(AEGStage::Release);
                } else if self.damper {
                    self.damp_start_level = self.level;
                    self.advance(AEGStage::Damp);
                } else {
                    self.level = self.sustain_level;
                }
            }
            AEGStage::Damp => {
                if self.damp_time.is_zero() || t >= self.damp_time {
                    self.level = 0.0;
                    self.advance(AEGStage::Finished);
                } else {
                    let p = t.as_secs_f32() * self.inv_damp;
                    self.level = (self.damp_start_level * (1.0 - p)).max(0.0);
                }
            }
            AEGStage::Release => {
                if self.release_time.is_zero() || t >= self.release_time {
                    self.level = 0.0;
                    self.advance(AEGStage::Finished);
                } else {
                    let p = t.as_secs_f32() * self.inv_release;
                    self.level = (self.release_start_level * (1.0 - p)).max(0.0);
                }
            }
            AEGStage::Finished => {
                self.level = 0.0;
            }
        }

        self.level
    }

    fn advance(&mut self, next: AEGStage) {
        self.state = next;
        self.stage_started = self.elapsed_total;
    }
}

impl Audio for AEG {
    fn tick(&mut self, elapsed: Duration) -> f32 {
        self.tick(elapsed)
    }
}

/// 08 pp 1A-1C relative time parameter → milliseconds (64=center)
fn param_to_ms(param: u8, base_ms: f32) -> f32 {
    let off = param as f32 - 64.0;
    (base_ms * 1.2f32.powf(off)).clamp(0.1, 20000.0)
}

/// 1 / duration_secs (zero-safe: zero duration → 0, the tick's is_zero branch
/// short-circuits before the multiply)
#[inline]
fn inv_secs(d: Duration) -> f32 {
    let s = d.as_secs_f32();
    if s > 0.0 { 1.0 / s } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settled(aeg: &mut AEG, steps: usize, step: Duration) {
        for _ in 0..steps {
            aeg.tick(step);
        }
    }

    #[test]
    fn damper_decays_from_sustain_to_release() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64); // attack 5ms, decay 100ms, release 100ms
        settled(&mut aeg, 200, Duration::from_millis(1));
        assert_eq!(aeg.state, AEGStage::Sustain);
        assert_eq!(aeg.level, aeg.sustain_level);

        aeg.set_damper(true);
        aeg.tick(Duration::from_millis(1));
        assert_eq!(aeg.state, AEGStage::Damp, "damper must enter Damp from Sustain");
        let l0 = aeg.level;
        aeg.tick(Duration::from_millis(100));
        assert!(aeg.level < l0, "Damp stage must decay the level");

        aeg.note_off();
        assert_eq!(aeg.state, AEGStage::Release, "pedal release → Release stage");
        let l1 = aeg.level;
        settled(&mut aeg, 50, Duration::from_millis(2));
        assert_eq!(aeg.state, AEGStage::Finished, "release finishes quickly");
        assert!(aeg.level < l1, "release decays from the current level");
    }

    #[test]
    fn damper_disabled_freezes_at_sustain() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64);
        settled(&mut aeg, 200, Duration::from_millis(1));
        aeg.set_damper(false);
        settled(&mut aeg, 50, Duration::from_millis(10));
        assert_eq!(aeg.state, AEGStage::Sustain);
        assert_eq!(aeg.level, aeg.sustain_level, "no damper → frozen at sustain level");
    }

    #[test]
    fn release_from_decay_uses_current_level() {
        let mut aeg = AEG::new();
        aeg.setup(64, 64, 64);
        settled(&mut aeg, 30, Duration::from_millis(1)); // still inside Decay
        assert_eq!(aeg.state, AEGStage::Decay);
        let l0 = aeg.level;
        assert!(l0 < 1.0 && l0 > 0.0);
        aeg.note_off();
        aeg.tick(Duration::from_millis(1));
        assert!(aeg.level <= l0, "release continues from the current level");
    }
}
