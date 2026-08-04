/// Amp (放大器)
///
/// 信号链: input × AEG.level × velocity × expression × part_volume × (1 + LFO AM)
///
/// 对齐说明:
/// - velocity: `MultiPart.get_velocity` (力度下限/上限 + sense depth/offset)
/// - expression: Part.controller.expression (CC#11, 每块更新)
/// - part_volume: 08 pp 0B (CC#7), note-on 快照
/// - LFO AM: LFO.amp 输出调制 (MW LFO AMOD 08 pp 22 深度, 默认 0 = 无影响)
pub mod aeg;

pub use aeg::{AEG, AEGStage};

use std::time::Duration;

use crate::midi::ram::xg::multi_part::MultiPart;

#[derive(Debug)]
pub struct Amp {
    pub aeg: AEG,
    /// 有效力度 [0, 1] (note-on 快照)
    pub velocity: f32,
    /// Expression [0, 1] (CC#11, 每块更新)
    pub expression: f32,
    /// Part Volume [0, 1] (08 pp 0B, note-on 快照)
    pub volume: f32,
    /// LFO AM 调制深度 (0-1, MW LFO AMOD, 实时 ×MW)
    pub lfo_depth: f32,
    /// 外部调制 (MW/Bend/CAT/PAT amplitude control), dB, 每块更新
    pub mod_gain_db: f32,
}

impl Amp {
    pub fn new() -> Self {
        Self {
            aeg: AEG::new(),
            velocity: 1.0,
            expression: 1.0,
            volume: 1.0,
            lfo_depth: 0.0,
            mod_gain_db: 0.0,
        }
    }

    /// note-on 初始化
    pub fn setup(
        &mut self,
        vel: u8,
        ram: &MultiPart,
        eg_attack: u8,
        eg_decay: u8,
        eg_release: u8,
    ) {
        self.velocity = ram.get_velocity(vel) as f32 / 127.0;
        self.volume = ram.volume as f32 / 127.0;
        self.aeg.setup(eg_attack, eg_decay, eg_release);
    }

    /// 每 block 更新实时参数 (expression 等)
    pub fn update(&mut self, expression: u8) {
        self.expression = expression as f32 / 127.0;
    }

    /// 处理一个采样: 推进 AEG (每 block 一次, 由调用方控制频率) 并应用增益
    pub fn tick(&mut self, input: f32, block_elapsed: Duration, lfo_amp: f32) -> f32 {
        let eg = self.aeg.tick(block_elapsed);
        let am = 1.0 + lfo_amp * self.lfo_depth;
        let mod_gain = 10f32.powf(self.mod_gain_db / 20.0);
        input * eg * self.velocity * self.expression * self.volume * am * mod_gain
    }

    pub fn note_off(&mut self) {
        self.aeg.note_off();
    }

    pub fn kill(&mut self) {
        self.aeg.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amp_gain_chain() {
        let mut amp = Amp::new();
        amp.velocity = 0.5;
        amp.expression = 0.5;
        amp.volume = 0.5;
        amp.aeg.setup(0x40, 0x40, 0x40);
        // attack 完成后 eg=1: out = 1 × 1 × 0.5 × 0.5 × 0.5 = 0.125
        let out = amp.tick(1.0, Duration::from_millis(10), 0.0);
        assert!((out - 0.125).abs() < 1e-4, "out={out}");
    }

    #[test]
    fn lfo_am_modulates() {
        let mut amp = Amp::new();
        amp.velocity = 1.0;
        amp.expression = 1.0;
        amp.volume = 1.0;
        amp.lfo_depth = 0.5;
        amp.aeg.setup(0x40, 0x40, 0x40);
        amp.aeg.tick(Duration::from_millis(10)); // 到 attack 结束
        let out = amp.tick(1.0, Duration::from_millis(0), 0.5); // lfo_amp=+0.5
        assert!((out - 1.25).abs() < 1e-4, "out={out}");
    }

    #[test]
    fn aeg_release_to_zero() {
        let mut amp = Amp::new();
        amp.aeg.setup(0x40, 0x40, 0x40);
        amp.aeg.tick(Duration::from_millis(1000)); // 到 sustain
        amp.aeg.note_off();
        let level = amp.aeg.tick(Duration::from_millis(500));
        assert!(level.abs() < 1e-4, "level={level}");
        assert_eq!(amp.aeg.state, AEGStage::Finished);
    }
}
