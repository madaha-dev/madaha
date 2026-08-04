use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::audio::interface::Audio;

use super::delay::Delay;
use super::interpolating::InterpolatingMethods;
use super::peg::PEG;
use super::pitch::Pitch;
use super::portamento::Portamento;
use super::super::interface::ToneGeneratorInterface;

use crate::midi::Part;
use crate::voice_manager::SampleMeta;

/// ln(2) / 1200 —— cent → 频率比
const LN2_OVER_1200: f64 = 0.000577_622_650_319_656_4;

#[derive(Debug)]
pub struct Oscillator {
    pub peg: PEG,
    pub delay: Delay,
    pub portamento: Portamento,
    pub pitch: Pitch,
    pub velocity: u8,
    /// 外部调制 (MW/Bend/CAT/PAT pitch control), 音分, 每块更新
    pub pitch_mod: f32,

    /// 绑定的采样元数据（含 PCM 数据）
    sample: Option<&'static SampleMeta>,
    /// DDS 播放位置（样本单位, f64 防漂移）
    pos: f64,
    /// 插值方式
    pub interpolating: InterpolatingMethods,
    /// LFO 波形类型 (0-12, 与 2006LE 一致)
    pub lfo_wave: u8,

    // source_sample_rate / target_sample_rate
    pub play_speed_base: f64,
}

impl Oscillator {
    pub fn new(source_sample_rate: f32, target_sample_rate: f32) -> Self {
        Self {
            pitch: Pitch::new(),
            peg: PEG::new(),
            delay: Delay::new(),
            portamento: Portamento::new(),
            velocity: 0,
            pitch_mod: 0.0,

            play_speed_base: source_sample_rate as f64 / target_sample_rate as f64,
            interpolating: InterpolatingMethods::Linear,

            sample: None,
            pos: 0.0,
            lfo_wave: 0,
        }
    }

    pub fn set_sample(&mut self, sample: &'static SampleMeta) {
        self.sample = Some(sample);
        self.pos = 0.0;
    }

    /// 从 SampleMeta (S-YXG50 element) 初始化发声参数。
    ///
    /// 对齐说明 (S-YXG50 数据 vs 2006LE 程序):
    /// - 已对齐: coarse / fine(查表) / pitch_offset / tone / loop / pcm
    /// - PEG: `peg_rate0-4` 转换表未解析 → 中性值 (见 `PEG::setup`)
    /// - LFO: `lfo_wave` 0-12 与 2006LE 一致 → 直接映射
    /// - Part 级 (08 pp: vibrato/bend/detune/note_shift) 由 2006LE 程序从
    ///   MultiPart 读取, 待 voice 实时调制接入
    pub fn setup(&mut self, sample: &'static SampleMeta, note: u8, vel: u8, sample_rate: f32) {
        self.set_sample(sample);
        self.velocity = vel;
        self.pitch.note = note;
        self.pitch.note_in_cent = note as f32 * 100.0;
        self.portamento.target_note = self.pitch.note_in_cent;
        // PEG: S-YXG50 element[22..30] + 力度 + 键位
        self.peg.setup(sample, note, vel, sample_rate);
        self.lfo_wave = sample.lfo_wave & 0x07;
    }

    /// 不能直接传入note，需要查表拿到音分，音分表由GM可选标准计算得出
    /// Madaha 实现了这些标准
    pub fn set_note(&mut self, note: u8, cent_table: [f32; 128]) {
        let note_in_cent = cent_table[note as usize];
        self.pitch.note_in_cent = note_in_cent;
        self.pitch.note = note;
        self.portamento.target_note = note_in_cent;
    }

    pub fn set_lfo(&mut self, lfo_input: f32) {
        self.delay.lfo_input = lfo_input;
    }

    pub fn is_drum(&self) -> bool {
        // TODO: 从绑定的 part 读取 part_mode
        false
    }

    pub fn is_looping(&self) -> bool {
        if let Some(sm) = self.sample {
            sm.loop_length != 0
        } else {
            true
        }
    }

    pub fn play(&mut self, _p: f32, _part: Arc<RwLock<Part>>) {
        // TODO: 绑定 part + 实时 pitch 计算
    }
}

impl ToneGeneratorInterface for Oscillator {
    fn reset(&mut self) {}

    fn kill(&mut self) {
        self.peg.kill();
    }

    fn release(&mut self) {}
}

impl Audio for Oscillator {
    /// 每 sample 调用一次, 返回当前采样值
    fn tick(&mut self, elapsed: Duration) -> f32 {
        let Some(sample) = self.sample else {
            return 0.0;
        };
        let Some(pcm) = sample.pcm else {
            return 0.0;
        };
        if pcm.is_empty() {
            return 0.0;
        }

        // 1. 实时音分: note + 调制 + element 偏移
        let note_in_cent = self.delay.tick(elapsed)
            + self.peg.tick(elapsed)
            + self.portamento.tick(elapsed)
            + self.pitch.tick(elapsed)
            + self.pitch_mod
            + sample.get_coarse_in_cent()
            + sample.get_fine_in_cent(self.velocity)
            + sample.get_pitch_offset();

        // 2. cent → 频率比: ratio = 2^(cents/1200)
        //    (key - base) × 100 + tone + element 偏移
        let ratio_cents =
            note_in_cent - sample.get_base_note_cent() + sample.get_tone();
        let ratio = (ratio_cents as f64 * LN2_OVER_1200).exp();

        // 3. DDS 推进: step = ratio × (source_sr / target_sr)
        self.pos += ratio * self.play_speed_base;

        // 4. 位置回绕
        let len = pcm.len() as f64;
        if sample.loop_length > 0 {
            let loop_len = sample.loop_length as f64;
            if self.pos >= len {
                // 超出采样末尾（循环区之后）→ 折回
                let loop_start = sample.loop_point as f64;
                self.pos = loop_start + (self.pos - loop_start) % loop_len;
            }
        } else if self.pos >= len {
            // 一次性采样播完
            self.pos = len;
            return 0.0;
        }

        // 5. 插值采样
        self.interpolating
            .interpolate(pcm, sample.loop_point, sample.loop_length, self.pos)
    }
}
