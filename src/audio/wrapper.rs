use std::sync::mpsc::Receiver;

use crate::config::ScoringConfig;

use super::tone_generator::ToneGenerator;
use super::tone_generator::ToneGeneratorStatus::Running;
use super::AudioRenderActions;

#[derive(Debug)]
pub struct AudioRender {
    pub tone_generators: Box<[ToneGenerator]>,
    pub rx: Receiver<AudioRenderActions>,
    pub max_polyphony: u16,
}

impl AudioRender {
    pub fn new(
        count: usize,
        max_polyphony: u16,
        source_sample_rate: f32,
        target_sample_rate: f32,
        scoring: ScoringConfig,
        rx: Receiver<AudioRenderActions>,
    ) -> Self {
        Self {
            tone_generators: (0..count)
                .map(|_| {
                    ToneGenerator::new(source_sample_rate, target_sample_rate, scoring.clone())
                })
                .collect(),
            rx,
            max_polyphony,
        }
    }

    pub fn get_current_polyphony(&self) -> usize {
        self.tone_generators
            .iter()
            .filter(|&t| t.status == Running)
            .count()
    }

    pub fn find_all_tone_generators_by_channel(
        &mut self,
        channel: u8,
    ) -> Box<[&mut ToneGenerator]> {
        self.tone_generators
            .iter_mut()
            .filter(|t| t.bonded_to_channel(channel))
            .collect()
    }
}
