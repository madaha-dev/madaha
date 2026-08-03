use std::time::Duration;

use super::{
    super::interface::ToneGeneratorInterface, delay::Delay, peg::PEG, pitch::Pitch,
    portamento::Portamento,
};

use crate::{
    midi::{channel::Channel, ram::xg::multi_part::MultiPart},
    voice_manager::SampleMeta,
};

#[derive(Debug)]
pub struct Oscillator {
    pub peg: PEG,
    pub delay: Delay,
    pub portamento: Portamento,
    pub pitch: Pitch,
    pub velocity: u8,

    pub pcm_position_abs: f32,
    pub pcm_position_rel: f32,

    pub sample: Option<&'static SampleMeta>,

    // source_sample_rate / target_sample_rate
    pub play_speed_base: f32,
}

impl ToneGeneratorInterface for Oscillator {
    fn reset(&mut self) {}

    fn kill(&mut self) {
        self.peg.kill();
    }

    // every loop for a sample
    fn step(&mut self, elapsed: Duration) -> f32 {
        if let Some(sample) = self.sample {
            let coarse_in_cent = (sample.pitch_coarse as i8 - 64) as f32 * 100.0;

            let note_in_cent = self.delay.step(elapsed)
                + self.peg.step(elapsed)
                + self.portamento.step(elapsed)
                + self.pitch.step(elapsed)
                + coarse_in_cent;

            todo!()
        }
        0.0
    }
}

impl Oscillator {
    pub fn new(source_sample_rate: f32, target_sample_rate: f32) -> Self {
        Self {
            pitch: Pitch::new(),
            peg: PEG::new(),
            delay: Delay::new(),
            portamento: Portamento::new(),
            pcm_position_abs: 0.0,
            pcm_position_rel: 0.0,
            velocity: 0,

            play_speed_base: source_sample_rate / target_sample_rate,

            sample: None,
        }
    }

    /// 不能直接传入note，需要查表拿到音分，音分表由GM可选标准计算得出
    /// Madaha 实现了这些标准
    pub fn set_note(
        &mut self,
        note: u8,
        cent_table: [f32; 128],
        ram: &'static MultiPart,
        ch: &'static Channel,
    ) {
        let note = cent_table[note as usize];
        self.pitch.set_note(note, ram, ch);
        self.portamento.target_note = note;
    }

    pub fn set_lfo(&mut self, lfo_input: f32) {
        self.delay.lfo_input = lfo_input;
    }

    fn position_to_rel(&mut self) {
        if let Some(sample) = self.sample {
            let loop_point = sample.loop_point as f32;
            self.pcm_position_rel = if self.pcm_position_abs >= loop_point {
                (self.pcm_position_abs - loop_point) % sample.loop_length as f32 + loop_point
            } else {
                self.pcm_position_abs
            };
        }
    }

    pub fn is_drum(&self) -> bool {
        if let Some(r) = self.pitch.ram {
            r.part_mode != 0
        } else {
            false
        }
    }

    pub fn is_looping(&self) -> bool {
        if let Some(sm) = self.sample {
            sm.loop_length != 0
        } else {
            true
        }
    }
}
