use std::time::Instant;

use crate::config::ScoringConfig;

use super::sample::Sample;

#[derive(Debug)]
pub enum EGStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

#[derive(Debug)]
pub struct Voice {
    pub create_time: Instant,
    pub drumkit: bool,
    pub note: u8,
    pub velocity: u8,
    pub eg_stage: EGStage,
    pub samples: Box<[Sample]>,
    pub volume: f32, // TODO: more accratute measure.
}

impl Voice {
    // should not score here, just sample, will removed later.
    pub fn scoring(self: &Self, args: &ScoringConfig) -> u128 {
        let mut score = self.create_time.elapsed().as_millis() * args.time_weight as u128;
        score = match self.eg_stage {
            EGStage::Attack | EGStage::Decay => score * args.protect_attack as u128 / 1000,
            EGStage::Release => score * args.penalty_release as u128 / 1000,
            _ => score,
        };

        // if sustain(CC#64) hold
        score = score * args.protect_sustain_pedal as u128 / 1000;

        // note protect
        score = if self.drumkit {
            score * args.get_drum_scoring_map()[self.note as usize] as u128 / 1000
        } else {
            score * args.get_note_scoring_map()[self.note as usize] as u128 / 1000
        };

        // if non-loop sample
        score = score * args.protect_non_looping as u128 / 1000;

        // volume
        score = score * args.get_volume_weight(self.volume) as u128 / 1000;

        score
    }
}
