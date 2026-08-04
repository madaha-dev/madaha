/// AEG (Amplitude Envelope Generator, 幅度包络)
///
/// 状态机: Attack → Decay → Sustain → Release → Finished
/// 输出: level ∈ [0, 1] 直接作为音量增益
///
/// 对齐说明:
/// - Part 08 pp 1A-1C (EG Attack/Decay/Release Time) 同时作用于 AEG/FEG/PEG
///   (XG 音源模型), 段时间取自 Part 参数 (note-on 快照)
/// - Sustain level 由 Part 08 pp 18? 无独立字段 → 固定 0.7 (近似),
///   待 2006LE 数据文件支持后精确对接
use std::time::Duration;

use crate::audio::interface::Audio;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AEGStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

#[derive(Debug)]
pub struct AEG {
    pub state: AEGStage,
    /// 当前电平 [0, 1]
    pub level: f32,

    /// EG 使能 (element[71] eg_enable / [41] eg_amp_en; false → 恒 1.0)
    pub enabled: bool,

    pub attack_time: Duration,
    pub decay_time: Duration,
    pub sustain_level: f32,
    pub release_time: Duration,

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
            stage_started: Duration::ZERO,
            elapsed_total: Duration::ZERO,
        }
    }

    /// 从 Part 参数初始化 (note-on 时快照)
    /// `eg_attack/decay/release`: 08 pp 1A-1C, 64=中心
    pub fn setup(&mut self, eg_attack: u8, eg_decay: u8, eg_release: u8) {
        self.state = AEGStage::Attack;
        self.level = 0.0;
        self.stage_started = Duration::ZERO;
        self.elapsed_total = Duration::ZERO;
        self.attack_time = Duration::from_millis(param_to_ms(eg_attack, 5.0) as u64);
        self.decay_time = Duration::from_millis(param_to_ms(eg_decay, 100.0) as u64);
        self.release_time = Duration::from_millis(param_to_ms(eg_release, 100.0) as u64);
        self.sustain_level = 0.7;
    }

    pub fn note_off(&mut self) {
        if matches!(self.state, AEGStage::Finished | AEGStage::Release) {
            return;
        }
        self.state = AEGStage::Release;
        self.stage_started = self.elapsed_total;
    }

    pub fn kill(&mut self) {
        self.state = AEGStage::Finished;
        self.level = 0.0;
    }

    /// 推进包络 (每 block 调用一次), 返回当前 level [0,1]
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
                    self.level = (t.as_secs_f32() / self.attack_time.as_secs_f32()).min(1.0);
                }
            }
            AEGStage::Decay => {
                if self.decay_time.is_zero() || t >= self.decay_time {
                    self.level = self.sustain_level;
                    self.advance(AEGStage::Sustain);
                } else {
                    let p = t.as_secs_f32() / self.decay_time.as_secs_f32();
                    self.level = 1.0 - (1.0 - self.sustain_level) * p;
                }
            }
            AEGStage::Sustain => {
                self.level = self.sustain_level;
            }
            AEGStage::Release => {
                if self.release_time.is_zero() || t >= self.release_time {
                    self.level = 0.0;
                    self.advance(AEGStage::Finished);
                } else {
                    let p = t.as_secs_f32() / self.release_time.as_secs_f32();
                    self.level = self.sustain_level * (1.0 - p);
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

/// 08 pp 1A-1C 相对时间参数 → 毫秒 (64=中心)
fn param_to_ms(param: u8, base_ms: f32) -> f32 {
    let off = param as f32 - 64.0;
    (base_ms * 1.2f32.powf(off)).clamp(0.1, 20000.0)
}
