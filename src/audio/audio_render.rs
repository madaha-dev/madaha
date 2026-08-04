use std::sync::Arc;

use super::tone_generator::ToneGeneratorStatus::{Idle, Running};
use super::tone_generator::interface::ToneGeneratorInterface;
use super::AudioRender;
use super::AudioRenderActions;

use crate::midi::Part;

impl AudioRender {
    pub fn audio_render(&mut self) {
        // Drain all events in channel.
        loop {
            if !self.drain_event() {
                break;
            }
        }
    }

    fn drain_event(&mut self) -> bool {
        use AudioRenderActions::*;
        if let Ok(ev) = self.rx.try_recv() {
            match ev {
                Play { note, vel, part } => {
                    self.note_handler(note, vel, part);
                }
                Release { note, part } => {
                    self.release_handler(note, part);
                }
                ReleaseAll { part } => {
                    self.release_all_handler(part);
                }
                KillAll { part } => {
                    self.kill_all_handler(part);
                }
            }
            true
        } else {
            false
        }
    }

    fn note_handler(&mut self, note: crate::midi::note::Note, vel: u8, part: Arc<std::sync::RwLock<Part>>) {
        // Find a free voice; if none, steal the lowest-scoring one.
        let index = match self
            .tone_generators
            .iter()
            .position(|t| t.status == Idle)
        {
            Some(i) => i,
            None => {
                // Steal: highest score gets killed.
                // scoring 权重: 低分 = 保护 (新音×0.1 / 延音×0.1 / 鼓×0.05),
                // 高分 = 优先被杀 (Releasing×1.5, 老音 time_weight 累加)
                let (i, _) = self
                    .tone_generators
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, t)| t.scoring())
                    .map(|(i, t)| (i, t.scoring()))
                    .unwrap();
                self.tone_generators[i].kill();
                i
            }
        };

        self.tone_generators[index].play(note, vel, part);
    }

    fn release_handler(
        &mut self,
        note: crate::midi::note::Note,
        part: Arc<std::sync::RwLock<Part>>,
    ) {
        self.tone_generators
            .iter_mut()
            .filter(|t| t.bonded_to_part(&part) && t.get_note() == Some(note))
            .for_each(|t| t.release());
    }

    fn release_all_handler(&mut self, part: Arc<std::sync::RwLock<Part>>) {
        self.tone_generators
            .iter_mut()
            .filter(|t| t.bonded_to_part(&part) && t.status == Running)
            .for_each(|t| t.release());
    }

    fn kill_all_handler(&mut self, part: Arc<std::sync::RwLock<Part>>) {
        self.tone_generators
            .iter_mut()
            .filter(|t| t.bonded_to_part(&part) && t.status == Running)
            .for_each(|t| t.kill());
    }
}
