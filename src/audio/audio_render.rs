use std::sync::{Arc, RwLock, RwLockReadGuard};

use super::tone_generator::ToneGeneratorStatus::Running;
use super::AudioRender;

use crate::midi::Engine;
use crate::midi::event::MidiEvent;
use crate::midi::note::Note;

impl AudioRender {
    pub fn audio_render(&mut self, engine: &Arc<RwLock<Engine>>) {
        let e = engine.read().unwrap();
        // Drain all midi event in channel.
        loop {
            self.drain_midi_event(&e);
        }
    }

    fn drain_midi_event(&mut self, e: &RwLockReadGuard<Engine>) {
        if let Ok(ev) = self.rx.try_recv() {
            match ev {
                MidiEvent::NoteOn {
                    channel,
                    note,
                    velocity,
                    off_velocity: _,
                    duration: _,
                } => {
                    self.note_handler(&e, channel, note, velocity);
                }
                MidiEvent::NoteOff {
                    channel,
                    note,
                    velocity: _,
                    off_velocity: _,
                    duration: _,
                } => {
                    self.note_handler(&e, channel, note, 0);
                }
                _ => {
                    // Other type no need to proceed.
                }
            }
        }
    }

    fn note_handler(&mut self, e: &RwLockReadGuard<Engine>, channel: u8, note: Note, velocity: u8) {
        // should check element count.
        let channel = channel as usize;
        let note = note.final_note(e, channel);
        if velocity == 0 {
            // Note off, but some key note recv note off.
            if let Some(tg) = self
                .tone_generators
                .iter_mut()
                .filter(|tg| {
                    tg.status == Running
                        && tg.oscillator.pitch.note == note
                        && if let Some(chan) = tg.channel {
                            chan._channel == channel
                        } else {
                            false
                        }
                })
                .min_by_key(|tg| tg.attack_time)
            {
                tg.kill();
            }
        } else {
            // get 1-2 tone_generator(s), depends on element count.
        }
    }
}
