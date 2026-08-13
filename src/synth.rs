use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;
use std::time::Duration;

use crate::args::Args;
use crate::audio::AudioRender;
use crate::audio::AudioRenderActions;
use crate::config::Config;
use crate::midi::engine::Engine;

use wd_log::{log_debug_ln, log_info_ln, log_panic, log_warn_ln};

#[derive(Debug)]
pub struct Synth {}

impl Synth {
    pub fn new() -> Self {
        Self {}
    }

    fn run_audio(&self, cfg: &Config, arg: &Args, rx: Receiver<AudioRenderActions>) {
        let source_sample_rate = cfg.sound_module.module_type.get_sample_rate();
        let target_sample_rate = cfg.audio.sample_rate as f32;
        let max_polyphony = cfg.midi.max_polyphony;
        let count = (max_polyphony as u32 * cfg.midi.poly_replicant as u32 / 100) as usize;
        let scoring = cfg.midi.scoring.clone();
        let debug_mode = arg.debug;

        let cfg = cfg.clone();
        thread::spawn(move || {
            log_info_ln!("audio thread running...");
            let mut audio_render = AudioRender::new(
                count,
                max_polyphony,
                source_sample_rate,
                target_sample_rate,
                scoring,
                debug_mode,
                rx,
            );
            audio_render.dc_enabled = cfg.audio.dc_blocker;
            // Watchdog: sleep after `sleep_delay_ms` of total silence.
            audio_render.set_sleep_delay(Duration::from_millis(cfg.audio.sleep_delay_ms));
            // 按配置选择实时输出后端 (ALSA/PipeWire)
            match crate::audio::backend::create_sink(&cfg.audio) {
                Ok(mut sink) => {
                    if audio_render.debug_mode {
                        sink.set_debug(true);
                    }
                    // Render clock must follow the sink's actual negotiated
                    // rate (ALSA may return 22050 instead of 48000, which
                    // otherwise drops the pitch by ~1 octave).
                    audio_render.set_output_rate(sink.rate() as f32);
                    audio_render.set_sink(sink);
                }
                Err(e) => {
                    log_warn_ln!("audio backend open failed ({e}); using internal buffer");
                }
            }
            let block = cfg.audio.buffer_size.max(1) as usize;
            // Render loop with a watchdog: while any tone generator is
            // sounding (or the idle grace window hasn't elapsed) render
            // blocks like before; once the watchdog decides everything is
            // silent, sleep until a MIDI/audio event arrives (event-driven
            // wakeup — the device callback pads silence meanwhile).
            let sleep_poll = Duration::from_millis(200);
            loop {
                if audio_render.needs_render() {
                    for _ in 0..block {
                        audio_render.audio_render();
                    }
                    audio_render.flush();
                    // Yield between blocks: a pure busy loop pegs one core at
                    // 100% and starves the pipewire thread's scheduling on
                    // loaded systems. The sink's blocking writei still
                    // provides the audio clock; yielding just hands the core
                    // back without the imprecision of a sleep.
                    thread::yield_now();
                } else {
                    // All silent: block until an audio action arrives (or the
                    // poll interval elapses so state changes get re-checked).
                    audio_render.sleep_idle(sleep_poll);
                }
            }
        });
    }

    pub fn run(&mut self, cfg: &Config, arg: &Args) {
        log_debug_ln!("get ready for run synth");
        // main event loop
        let (tx, rx) = sync_channel(cfg.midi.channel_size);
        self.run_audio(cfg, arg, rx);

        let mut engine = Engine::new(cfg, arg, tx);
        engine.send_audio_init();
        log_debug_ln!("engine ready");

        // MIDI 输入：ALSA Seq 后端
        let mut source = match crate::midi::source::create_midi_source(cfg.midi.input_engine) {
            Ok(src) => src,
            Err(e) => {
                log_warn_ln!("midi input open failed ({e})");
                log_panic!("midi input unavailable: {e}");
            }
        };
        log_info_ln!("madaha running...");
        loop {
            if let Some(event) = source.next_event() {
                engine.on_event(event);
            }
        }
    }
}
