
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use crate::audio::AudioRender;
use crate::audio::AudioRenderActions;
use crate::config::Config;
use crate::midi::engine::Engine;

use wd_log::{log_info_ln, log_panic, log_warn_ln};

#[derive(Debug)]
pub struct Synth {}

impl Synth {
    pub fn new() -> Self {
        Self {}
    }


    fn run_audio(&self, cfg: &Config, rx: Receiver<AudioRenderActions>) {
        let source_sample_rate = cfg.sound_module.module_type.get_sample_rate();
        let target_sample_rate = cfg.audio.sample_rate as f32;
        let max_polyphony = cfg.midi.max_polyphony;
        let count = (max_polyphony * cfg.midi.poly_replicant / 100) as usize;
        let scoring = cfg.midi.scoring.clone();

        let cfg = cfg.clone();
        thread::spawn(move || {
            log_info_ln!("audio thread running...");
            let mut audio_render = AudioRender::new(
                count,
                max_polyphony,
                source_sample_rate,
                target_sample_rate,
                scoring,
                rx,
            );
            audio_render.dc_enabled = cfg.audio.dc_blocker;
            // 按配置选择实时输出后端 (ALSA/PulseAudio/Jack/PipeWire)
            match crate::audio::backend::create_sink(&cfg.audio) {
                Ok(sink) => {
                    audio_render.set_sink(sink);
                }
                Err(e) => {
                    log_warn_ln!("audio backend open failed ({e}); using internal buffer");
                }
            }
            let block = cfg.audio.buffer_size.max(1) as usize;
            loop {
                for _ in 0..block {
                    audio_render.audio_render();
                }
                audio_render.flush();
            }
        });
    }

    pub fn run(&mut self, cfg: &Config) {
        log_info_ln!("madaha running...");
        // main event loop
        let (tx, rx) = sync_channel(cfg.midi.channel_size);
        self.run_audio(cfg, rx);
        
        let mut engine = Engine::new(cfg, tx);
        engine.send_audio_init();

        // MIDI 输入按配置选择后端 (ALSA Seq / Jack / PipeWire)
        let mut source = match crate::midi::source::create_midi_source(cfg.midi.input_engine) {
            Ok(src) => src,
            Err(e) => {
                log_warn_ln!("midi input open failed ({e}); falling back to ALSA");
                crate::midi::source::create_midi_source(crate::config::MidiInputEngine::Alsa)
                    .unwrap_or_else(|e2| {
                        log_panic!("midi input unavailable: {e2}");
                    })
            }
        };
        loop {
            if let Some(event) = source.next_event() {
                engine.on_event(event);
            }
        }
    }
}
