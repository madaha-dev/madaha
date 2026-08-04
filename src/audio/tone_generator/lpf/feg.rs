/// FEG (Filter Envelope Generator, 滤波器包络)
///
/// 状态机: Attack → Decay → Sustain → Release → Finished
/// 输出: level ∈ [0, 1], 经 `filter_eg_depth` 调制 cutoff 参数
///
/// 对齐说明:
/// - Part 08 pp 1A-1C (EG Attack/Decay/Release Time) 同时作用于 AEG/FEG/PEG
///   (XG 音源模型), 故 FEG 段时间取自 Part 参数 (note-on 快照)
/// - Part 08 pp 71 (Filter EG Depth, 64=中心) → 调制深度, 默认 0 = 无影响
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
    /// 当前电平 [0, 1]
    pub level: f32,

    /// EG 使能 (element[71] eg_enable / [40] eg_filt_en; false → 恒 0.0)
    pub enabled: bool,

    // ── 段参数 ──
    pub attack_time: Duration,
    pub decay_time: Duration,
    pub sustain_level: f32,
    pub release_time: Duration,

    // 段起始时间戳
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

    /// 从 Part 参数初始化 (note-on 时快照)
    ///
    /// `eg_attack/decay/release`: 08 pp 1A-1C, 64=中心, 相对 VCE 的时间偏移
    /// `feg_depth`: 08 pp 71 Filter EG Depth (64=0)
    pub fn setup(&mut self, eg_attack: u8, eg_decay: u8, eg_release: u8, _feg_depth: u8) {
        self.state = FEGStage::Attack;
        self.level = 0.0;
        self.stage_started = Duration::ZERO;
        self.elapsed_total = Duration::ZERO;

        // 时间参数 (相对): 64=中心 → 基准时间, 简单映射到 ms
        // (2006LE 用速率表, 这里用近似; 待数据文件支持后精确对接)
        self.attack_time = Duration::from_millis(param_to_ms(eg_attack, 5.0) as u64);
        self.decay_time = Duration::from_millis(param_to_ms(eg_decay, 100.0) as u64);
        self.release_time = Duration::from_millis(param_to_ms(eg_release, 100.0) as u64);
        // VCE 无独立 sustain level 字段 → 默认 0.5 (中性)
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

    /// 推进包络 (每 block 调用一次), 返回当前 level [0,1]
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

/// 08 pp 1A-1C 相对时间参数 → 毫秒
/// 64=中心(基准), 0=最快, 127=最慢
fn param_to_ms(param: u8, base_ms: f32) -> f32 {
    let off = param as f32 - 64.0;
    // 每单位约 ×1.2 (对数), 范围 ±~200x
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
        feg.setup(0x40, 0x40, 0x40, 0x40); // 64 中心: attack 5ms, decay 100ms
        // attack 到 1.0
        let mut level = step_ms(&mut feg, 10);
        assert!((level - 1.0).abs() < 1e-4, "attack end = {level}");
        assert_eq!(feg.state, FEGStage::Decay);
        // decay 到 sustain (0.5)
        step_ms(&mut feg, 500);
        assert!((feg.level - 0.5).abs() < 1e-4, "sustain = {}", feg.level);
        assert_eq!(feg.state, FEGStage::Sustain);
    }

    #[test]
    fn feg_release() {
        let mut feg = FEG::new();
        feg.setup(0x40, 0x40, 0x40, 0x40);
        step_ms(&mut feg, 1000); // 到 sustain
        feg.note_off();
        let mut level = step_ms(&mut feg, 500);
        assert!((level - 0.0).abs() < 1e-4, "release end = {level}");
        assert_eq!(feg.state, FEGStage::Finished);
    }

    #[test]
    fn cutoff_compute() {
        let co = CutOff::new();
        // base 64, 无调制 → 64 参数的 Hz
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
