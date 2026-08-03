use std::time::{self, Instant};

use crate::lfo::LFO;
use crate::{config::ScoringConfig, midi::channel::Channel};

use super::interface::ToneGeneratorInterface;
use super::oscillator::Oscillator;

#[derive(Debug, PartialEq)]
pub enum ToneGeneratorStatus {
    Idle,
    Running,
    Releasing,
}

#[derive(Debug)]
pub struct ToneGenerator {
    // update when NoteOn
    pub attack_time: time::Instant,
    // update when NoteOff/NoteOn(vel=0)
    pub release_time: time::Instant,

    pub status: ToneGeneratorStatus,
    pub scoring_config: ScoringConfig,

    // Channel will exists all the runtime.
    pub channel: Option<&'static Channel>,

    pub lfo: LFO,

    pub oscillator: Oscillator,
    pub lpf: u8, // TODO
    pub hpf: u8, // TODO
    pub amp: u8, // TODO
    pub eq: u8,  // might read from xg ram
    pub pan: u8, // read from channel parameters
}

impl ToneGenerator {
    pub fn new(source_sample_rate: f32, target_sample_rate: f32, scoring: ScoringConfig) -> Self {
        Self {
            attack_time: Instant::now(),
            release_time: Instant::now(),
            status: ToneGeneratorStatus::Idle,
            channel: None,
            lfo: LFO::new(),
            oscillator: Oscillator::new(source_sample_rate, target_sample_rate),
            lpf: 0, // TODO
            hpf: 0, // TODO
            amp: 0, // TODO
            eq: 0,  // TODO
            pan: 0, // TODO
            scoring_config: scoring,
        }
    }

    pub fn kill(&mut self) {
        self.release_time = Instant::now();
        self.oscillator.kill();
        // ... lfp hpf amp ...
    }

    pub fn scoring(&self) -> u128 {
        let args = self.scoring_config;
        let mut score = self.attack_time.elapsed().as_millis() * args.time_weight as u128;
        score = match self.eg_stage {
            EGStage::Attack | EGStage::Decay => score * args.protect_attack as u128 / 1000,
            EGStage::Release => score * args.penalty_release as u128 / 1000,
            _ => score,
        };

        // if sustain(CC#64) hold
        score = score * args.protect_sustain_pedal as u128 / 1000;

        // note protect
        score = if self.oscillator.is_drum() {
            score * args.get_drum_scoring_map()[self.get_note()] as u128 / 1000
        } else {
            score * args.get_note_scoring_map()[self.get_note()] as u128 / 1000
        };

        // if non-loop sample
        if !self.oscillator.is_looping() {
            score = score * args.protect_non_looping as u128 / 1000;
        }

        // volume
        score = score * args.get_volume_weight(self.volume) as u128 / 1000;

        score
    }
}
