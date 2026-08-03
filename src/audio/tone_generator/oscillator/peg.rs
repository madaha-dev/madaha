use std::time::Duration;

use super::super::interface::ToneGeneratorInterface;

/// PEG 状态机
///   Hold → 1st → 2nd → 3rd → Sustain → Release → Finished
///   每段有 Level 插值和 Time 保持两个子状态
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

    pub current_level: f32,

    // ── 段参数 ──
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

    /// 每 block 调用, now 由音频线程传入 (每 block 取一次)
    fn step(&mut self, elapsed: Duration) -> f32 {
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

impl PEG {
    pub fn new() -> Self {
        Self {
            state: PEGState::Hold,
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
}
