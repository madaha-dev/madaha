use std::time::Duration;

use super::super::interface::ToneGeneratorInterface;
use crate::audio::interface::Audio;

/// PEG state machine
///   Hold → 1st → 2nd → 3rd → Sustain → Release → Finished
///   Each stage has two substates: Level interpolation and Time hold
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PEGState {
    Hold,
    Stage1Level,
    Stage1Time,
    Stage2Level,
    Stage2Time,
    Stage3Level,
    Stage3Time,
    Sustain,
    Release,
    Finished,
}

#[derive(Debug)]
pub struct PEG {
    pub state: PEGState,

    /// EG enable (element[71] eg_enable / [43] eg_pitch_en; false → always 0.0)
    pub enabled: bool,

    pub current_level: f32,

    // ── Stage parameters ──
    pub stage1_level: f32,
    pub stage2_level: f32,
    pub stage3_level: f32,
    pub release_level: f32,

    pub stage1_rate: f32, // cent/sample
    pub stage2_rate: f32,
    pub stage3_rate: f32,
    pub release_rate: f32,

    pub hold_duration: Duration,
    pub stage1_duration: Duration,
    pub stage2_duration: Duration,
    pub stage3_duration: Duration,
}

impl ToneGeneratorInterface for PEG {
    fn reset(&mut self) {
        *self = Self::new();
    }

    fn kill(&mut self) {
        self.state = PEGState::Release;
    }

    fn release(&mut self) {
        self.note_off();
    }
}

impl PEG {
    pub fn new() -> Self {
        Self {
            state: PEGState::Hold,
            enabled: true,
            current_level: 0.0,
            stage1_level: 0.0,
            stage2_level: 0.0,
            stage3_level: 0.0,
            release_level: 0.0,
            stage1_rate: 0.0,
            stage2_rate: 0.0,
            stage3_rate: 0.0,
            release_rate: 0.0,
            hold_duration: Duration::ZERO,
            stage1_duration: Duration::ZERO,
            stage2_duration: Duration::ZERO,
            stage3_duration: Duration::ZERO,
        }
    }

    fn advance(&mut self, next: PEGState) {
        self.state = next;
    }

    pub fn note_off(&mut self) {
        if matches!(self.state, PEGState::Finished | PEGState::Release) {
            return;
        }
        self.advance(PEGState::Release);
    }

    /// Initialize PEG from SampleMeta (S-YXG50 element [22..30]) + velocity + key position.
    ///
    /// Alignment notes (S-YXG50 peg region vs SC-88/qxgedit style Pitch EG):
    /// - `peg_rate0-4`: Attack / Decay1 / Decay2 / (Sustain) / Release stage rates
    ///   → stage1/stage2/stage3/release_rate; `peg_rate3` (Sustain stage) is a flat hold, not consumed
    /// - `peg_vel_sense_level` (63=neutral): velocity → peak level scaling
    /// - `peg_vel_sense_rate` (63=neutral): velocity → rate scaling
    /// - `peg_rate_scaling` (63=neutral) + `peg_center_note`: key position → rate scaling
    /// - Stage levels follow the SC-88 default curve: +100 → +50 → 0 (cents), release → 0
    /// - Rates use an exponential approximation (full 100-cent sweep 30s..1ms, to be verified against the 2006LE rate table)
    /// - element[11..12] (pitch_eg_attack/decay, FM style 0-3) belongs to the vtable[0x51c]
    ///   DSP initialization chain, to be wired in during the DSP stage
    pub fn setup(
        &mut self,
        sample: &'static crate::voice_manager::SampleMeta,
        note: u8,
        vel: u8,
        sample_rate: f32,
    ) {
        // Stage levels: SC-88 default curve (cents)
        self.stage1_level = 100.0;
        self.stage2_level = 50.0;
        self.stage3_level = 0.0;
        self.release_level = 0.0;

        // Key-position rate scaling (peg_rate_scaling + peg_center_note, 63=neutral)
        let mut rate_scale = 1.0f32;
        let scaling = (sample.peg_rate_scaling as f32 - 63.0) / 64.0;
        if scaling.abs() > 1e-4 {
            let semis = note as f32 - sample.peg_center_note as f32;
            rate_scale *= 2f32.powf(semis / 12.0 * scaling * 2.0);
        }

        // Velocity rate scaling (peg_vel_sense_rate, 63=neutral)
        let vel_rate = (sample.peg_vel_sense_rate as f32 - 63.0) / 64.0;
        if vel_rate.abs() > 1e-4 {
            rate_scale *= 1.0 + (vel as f32 - 64.0) / 64.0 * vel_rate;
        }
        let rate_scale = rate_scale.clamp(0.25, 4.0);

        // Velocity level scaling (peg_vel_sense_level, 63=neutral): higher velocity → higher peak
        let vel_level = (sample.peg_vel_sense_level as f32 - 63.0) / 64.0;
        let level_scale =
            (1.0 + (vel as f32 - 64.0) / 64.0 * vel_level * 0.5).clamp(0.5, 1.5);
        self.stage1_level *= level_scale;
        self.stage2_level *= level_scale;

        // Stage rates (cent/sample)
        self.stage1_rate = rate_to_cent_per_sample(sample.peg_rate0, rate_scale, sample_rate);
        self.stage2_rate = rate_to_cent_per_sample(sample.peg_rate1, rate_scale, sample_rate);
        self.stage3_rate = rate_to_cent_per_sample(sample.peg_rate2, rate_scale, sample_rate);
        self.release_rate = rate_to_cent_per_sample(sample.peg_rate4, rate_scale, sample_rate);

        self.state = PEGState::Hold;
        self.current_level = 0.0;
    }

    /// XG Part Pitch EG (0A pp 34-37) overrides, only non-default (0x40) values take effect:
    /// - `init_level`: initial pitch offset (±12 semitones)
    /// - `attack_time`: overrides the attack (stage1) rate
    /// - `release_level`: release target (±12 semitones)
    /// - `release_time`: overrides the release rate
    pub fn apply_xg_eg(
        &mut self,
        init_level: u8,
        attack_time: u8,
        release_level: u8,
        release_time: u8,
        sample_rate: f32,
    ) {
        if init_level != 0x40 {
            self.current_level = (init_level as f32 - 64.0) / 64.0 * 1200.0;
        }
        if attack_time != 0x40 {
            let t = 0.002 * 2f32.powf((127 - attack_time.min(127)) as f32 / 10.0);
            self.stage1_rate = 100.0 / t / sample_rate;
        }
        if release_level != 0x40 {
            self.release_level = (release_level as f32 - 64.0) / 64.0 * 1200.0;
        }
        if release_time != 0x40 {
            let t = 0.002 * 2f32.powf((127 - release_time.min(127)) as f32 / 10.0);
            self.release_rate = 100.0 / t / sample_rate;
        }
    }
}

/// rate (0-127) → cent/sample
/// Exponential approximation: rate 0 → full 100 cents ≈ 30s, rate 127 → ≈ 1ms
fn rate_to_cent_per_sample(rate: u8, scale: f32, sample_rate: f32) -> f32 {
    let t = 30.0 * 2f32.powf(-(rate as f32) / 6.4);
    100.0 / t * scale / sample_rate
}

impl Audio for PEG {
    fn tick(&mut self, elapsed: Duration) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        use PEGState::*;

        match self.state {
            Hold => {
                if elapsed >= self.hold_duration {
                    if self.current_level != self.stage1_level {
                        self.advance(Stage1Level);
                    } else if self.current_level != self.stage2_level {
                        self.advance(Stage2Level);
                    } else {
                        self.advance(Sustain);
                    }
                }
            }

            Stage1Level => {
                let diff = self.stage1_level - self.current_level;
                if diff.abs() <= self.stage1_rate.abs() {
                    self.current_level = self.stage1_level;
                    self.advance(Stage1Time);
                } else {
                    self.current_level += self.stage1_rate * diff.signum();
                }
            }
            Stage1Time => {
                if elapsed >= self.stage1_duration + self.hold_duration {
                    if self.stage1_level != self.stage2_level {
                        self.advance(Stage2Level);
                    } else {
                        self.advance(Sustain);
                    }
                }
            }

            Stage2Level => {
                let diff = self.stage2_level - self.current_level;
                if diff.abs() <= self.stage2_rate.abs() {
                    self.current_level = self.stage2_level;
                    self.advance(Stage2Time);
                } else {
                    self.current_level += self.stage2_rate * diff.signum();
                }
            }
            Stage2Time => {
                if elapsed >= self.stage2_duration + self.stage1_duration + self.hold_duration {
                    if self.stage2_level != self.stage3_level {
                        self.advance(Stage3Level);
                    } else {
                        self.advance(Sustain);
                    }
                }
            }

            Stage3Level => {
                let diff = self.stage3_level - self.current_level;
                if diff.abs() <= self.stage3_rate.abs() {
                    self.current_level = self.stage3_level;
                    self.advance(Stage3Time);
                } else {
                    self.current_level += self.stage3_rate * diff.signum();
                }
            }
            Stage3Time => {
                if elapsed
                    >= self.stage3_duration
                        + self.stage2_duration
                        + self.stage1_duration
                        + self.hold_duration
                {
                    self.advance(Sustain);
                }
            }

            Sustain => {}

            Release => {
                let diff = self.release_level - self.current_level;
                if diff.abs() <= self.release_rate.abs() || self.current_level == 0.0 {
                    self.current_level = self.release_level;
                    self.advance(Finished);
                } else {
                    self.current_level += self.release_rate * diff.signum();
                }
            }

            Finished => self.current_level = 0.0,
        }

        self.current_level
    }
}
