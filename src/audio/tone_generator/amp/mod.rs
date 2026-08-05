/// Amp (amplifier)
///
/// Signal chain: input × AEG.level × velocity × expression × part_volume × (1 + LFO AM)
///
/// Alignment notes:
/// - velocity: `MultiPart.get_velocity` (velocity range limits + sense depth/offset)
/// - expression: Part.controller.expression (CC#11, updated each block)
/// - part_volume: 08 pp 0B (CC#7), note-on snapshot
/// - LFO AM: LFO.amp output modulation (MW LFO AMOD 08 pp 22 depth, default 0 = no effect)
pub mod aeg;

pub use aeg::AEG;

use std::time::Duration;

use crate::midi::ram::xg::multi_part::MultiPart;

#[derive(Debug)]
pub struct Amp {
    pub aeg: AEG,
    /// Effective velocity [0, 1] (note-on snapshot)
    pub velocity: f32,
    /// Expression [0, 1] (CC#11, updated each block)
    pub expression: f32,
    /// Part Volume [0, 1] (08 pp 0B, note-on snapshot)
    pub volume: f32,
    /// LFO AM modulation depth (0-1, MW LFO AMOD, real-time ×MW)
    pub lfo_depth: f32,
    /// External modulation (MW/Bend/CAT/PAT amplitude control), dB, updated each block
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

    /// note-on initialization
    pub fn setup(&mut self, vel: u8, ram: &MultiPart, eg_attack: u8, eg_decay: u8, eg_release: u8) {
        self.velocity = ram.get_velocity(vel) as f32 / 127.0;
        self.volume = ram.volume as f32 / 127.0;
        self.aeg.setup(eg_attack, eg_decay, eg_release);
    }

    /// Update real-time parameters each block (expression, etc.)
    pub fn update(&mut self, expression: u8) {
        self.expression = expression as f32 / 127.0;
    }

    /// Process one sample: advance AEG (once per block, frequency controlled by caller) and apply gain
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
    use super::aeg::AEGStage;

    #[test]
    fn amp_gain_chain() {
        let mut amp = Amp::new();
        amp.velocity = 0.5;
        amp.expression = 0.5;
        amp.volume = 0.5;
        amp.aeg.setup(0x40, 0x40, 0x40);
        // after attack completes eg=1: out = 1 × 1 × 0.5 × 0.5 × 0.5 = 0.125
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
        amp.aeg.tick(Duration::from_millis(10)); // reach end of attack
        let out = amp.tick(1.0, Duration::from_millis(0), 0.5); // lfo_amp=+0.5
        assert!((out - 1.25).abs() < 1e-4, "out={out}");
    }

    #[test]
    fn aeg_release_to_zero() {
        let mut amp = Amp::new();
        amp.aeg.setup(0x40, 0x40, 0x40);
        amp.aeg.tick(Duration::from_millis(1000)); // reach sustain
        amp.aeg.note_off();
        let level = amp.aeg.tick(Duration::from_millis(500));
        assert!(level.abs() < 1e-4, "level={level}");
        assert_eq!(amp.aeg.state, AEGStage::Finished);
    }
}
