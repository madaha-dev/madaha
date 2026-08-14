use std::time::Duration;

use libmadaha::yxg50::pre_voice::{
    key_follow, peg_level, peg_rate_word, peg_vel_sense_level, peg_vel_sense_rate,
};

use super::super::interface::ToneGeneratorInterface;
use crate::audio::interface::Audio;
use crate::voice_manager::SampleMeta;

/// S-YXG50 PEG 每段推进一次对应的采样数（渲染块 = 128 样本）。
/// 引擎速率字（表 0x10048134）为「内部电平单位 / 块」，累加到 voice[0x38]；
/// 内部电平 >> 2 = cents（voice[0x36]），加到主音高（FUN_10016360）。
const PEG_BLOCK_SAMPLES: f32 = 128.0;

/// 速率字（内部单位/块）→ cent/样本：÷128（块→样本）× ÷4（内部→cent）= ÷512。
const RATE_DIVISOR: f32 = PEG_BLOCK_SAMPLES * 4.0;

/// 4 电平包络状态机（FUN_10015b10/FUN_10015ac0）：
/// elem[26] → elem[27] → elem[28] → elem[29] → 保持（sustain）。
/// elem[27]==elem[28] 跳 attack；elem[28]==elem[29] 跳 decay。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PEGState {
    /// stage 0：初始 → elem[27]（KeyOn 设置）
    Stage0,
    /// stage 1：attack（elem[27] → elem[28]）
    Attack,
    /// stage 2：decay（elem[28] → elem[29]）
    Decay,
    /// stage 3：sustain（保持）
    Sustain,
    Release,
    Finished,
}

#[derive(Debug)]
pub struct PEG {
    pub state: PEGState,

    /// EG enable（element[71] eg_enable / [43] eg_pitch_en；false → 始终 0）
    pub enabled: bool,

    /// 当前电平（cent）
    pub current_level: f32,
    /// 当前段目标电平（cent）
    pub target_level: f32,
    /// 当前段每样本速率（cent/样本，有符号；下降段为负）
    pub rate: f32,
    /// 当前段是否即时（速率字 == 0x8000 → 直接跳目标）
    pub instant: bool,

    // ── 预计算的段数据（setup 时填好，均为 cent） ──
    stage0_target: f32,
    attack_target: f32,
    decay_target: f32,
    stage0_rate_word: u16,
    attack_rate_word: u16,
    decay_rate_word: u16,
    skip_attack: bool,
    skip_decay: bool,

    // ── Release 段 ──
    pub release_level: f32,
    pub release_rate: f32,

    // Part 音高 EG 覆盖用（apply_xg_eg）：默认 attack 速率（OLD 近似，用 elem[27]）
    default_attack_rate: u8,
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
            state: PEGState::Finished,
            enabled: true,
            current_level: 0.0,
            target_level: 0.0,
            rate: 0.0,
            instant: false,
            stage0_target: 0.0,
            attack_target: 0.0,
            decay_target: 0.0,
            stage0_rate_word: 0,
            attack_rate_word: 0,
            decay_rate_word: 0,
            skip_attack: false,
            skip_decay: false,
            release_level: 0.0,
            release_rate: 0.0,
            default_attack_rate: 0x40,
        }
    }

    pub fn note_off(&mut self) {
        if matches!(self.state, PEGState::Finished | PEGState::Release) {
            return;
        }
        self.state = PEGState::Release;
        // 下降段速率取负（release_level 通常 ≤ 当前电平）
        self.rate = -self.release_rate.abs();
        self.target_level = self.release_level;
    }

    /// 初始化 PEG（S-YXG50 元素 [17..24]/[26..29] + velocity + key，FUN_10015c90 区语义）。
    ///
    /// 4 电平包络：`level(elem[26]) → level(elem[27]) → level(elem[28]) → level(elem[29])`；
    /// 速率字由表 0x10048134 查表（`peg_rate_word`），键跟 C（elem[20]/[21]）与 elem[19]
    /// 力度缩放汇入速率索引。
    pub fn setup(
        &mut self,
        sample: &'static SampleMeta,
        note: u8,
        vel: u8,
        _sample_rate: f32,
    ) {
        self.default_attack_rate = sample.peg_rate1; // elem[27]（OLD 近似速率，供 Part EG 默认用）
        // 缩放助手（voice[0x4f]/[0x50]/[0x51]）
        let vel_sense = peg_vel_sense_level(sample.note_shift, vel) as u8; // elem[18] → voice[0x51]
        let detune = (peg_vel_sense_rate(sample.detune, vel) as i8) as i32; // elem[19] → voice[0x50]（signed）
        let key_follow_c = key_follow(note, sample.peg_center_high, sample.peg_center_low); // elem[21]/[20]
        let mode = sample.voice_type; // elem[17]

        // 4 电平（内部单位 → cent）
        let level26 = peg_level(sample.peg_rate0 as i32 - 0x40, vel_sense, mode) as f32 / 4.0;
        let level27 = peg_level(sample.peg_rate1 as i32 - 0x40, vel_sense, mode) as f32 / 4.0;
        let level28 = peg_level(sample.peg_rate2 as i32 - 0x40, vel_sense, mode) as f32 / 4.0;
        let level29 = peg_level(sample.peg_rate3 as i32 - 0x40, vel_sense, mode) as f32 / 4.0;

        // 3 段速率字（stage0 键跟输入 = elem[22]；attack/decay = 0x40 中性，Part 音高 EG 另经 apply_xg_eg）
        self.stage0_rate_word = peg_rate_word(sample.peg_rate0, sample.peg_vel_sense_level, key_follow_c, detune);
        self.attack_rate_word = peg_rate_word(sample.peg_vel_sense_rate, 0x40, key_follow_c, detune);
        self.decay_rate_word = peg_rate_word(sample.peg_rate_scaling, 0x40, key_follow_c, detune);

        // 段目标（cent）+ 跳过标志
        self.stage0_target = level27;
        self.attack_target = level28;
        self.decay_target = level29;
        self.skip_attack = sample.peg_rate1 == sample.peg_rate2;
        self.skip_decay = sample.peg_rate2 == sample.peg_rate3;

        // 初始状态：current = level(elem[26])，target = level(elem[27])，rate = stage0 速率字
        self.current_level = level26;
        self.target_level = level27;
        self.start_rate(self.stage0_rate_word, self.stage0_target, PEGState::Stage0);
        self.release_level = level29;
        self.release_rate = 0.0;
    }

    /// 按速率字 + 目标设定当前段（含符号与即时判定）。
    fn start_rate(&mut self, rate_word: u16, target: f32, state: PEGState) {
        self.instant = rate_word == 0x8000;
        let mag = rate_word as f32 / RATE_DIVISOR;
        self.rate = if target < self.current_level {
            -mag
        } else {
            mag
        };
        self.target_level = target;
        self.state = state;
    }

    /// 推进一段（到达目标 → 进入下一段；含跳过与即时处理）。
    fn advance_stage(&mut self) {
        self.current_level = self.target_level;
        match self.state {
            PEGState::Stage0 => {
                if self.skip_attack {
                    if self.skip_decay {
                        self.state = PEGState::Sustain;
                    } else {
                        self.start_rate(self.decay_rate_word, self.decay_target, PEGState::Decay);
                    }
                } else {
                    self.start_rate(self.attack_rate_word, self.attack_target, PEGState::Attack);
                }
            }
            PEGState::Attack => {
                if self.skip_decay {
                    self.state = PEGState::Sustain;
                } else {
                    self.start_rate(self.decay_rate_word, self.decay_target, PEGState::Decay);
                }
            }
            PEGState::Decay => self.state = PEGState::Sustain,
            _ => {}
        }
    }

    /// 每样本推进一步（累加 cent/样本速率）。到达目标返回 true。
    fn step(&mut self) -> bool {
        if self.instant || self.rate == 0.0 {
            return true;
        }
        let diff = self.target_level - self.current_level;
        if diff.abs() <= self.rate.abs() {
            true
        } else {
            self.current_level += self.rate;
            false
        }
    }

    /// XG Part Pitch EG（0A pp 34-37，即引擎 part[0x62]/[0x63]）覆盖，仅非默认（0x40）生效。
    /// 覆盖元素 PEG 为简单的「init → 0 → sustain 0 → release」包络。
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
            self.target_level = 0.0;
            self.instant = false;
            self.state = PEGState::Stage0;
        }
        if attack_time != 0x40 {
            let t = 0.002 * 2f32.powf((127 - attack_time.min(127)) as f32 / 10.0);
            self.rate = 100.0 / t / sample_rate;
            if self.target_level < self.current_level {
                self.rate = -self.rate;
            }
        } else if init_level != 0x40 {
            // 默认 attack 速率（OLD 近似：rate_to_cent_per_sample(elem[27])）
            let t = 30.0 * 2f32.powf(-(self.default_attack_rate as f32) / 6.4);
            self.rate = -(100.0 / t / sample_rate); // 下滑至 0 → 负速率
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

impl Audio for PEG {
    fn tick(&mut self, _elapsed: Duration) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        use PEGState::*;

        match self.state {
            Stage0 | Attack | Decay => {
                if self.step() {
                    self.advance_stage();
                }
            }
            Sustain => {}
            Release => {
                if self.step() {
                    self.state = Finished;
                    self.current_level = self.release_level;
                }
            }
            Finished => self.current_level = 0.0,
        }

        self.current_level
    }
}
