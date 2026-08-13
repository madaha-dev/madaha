use std::sync::mpsc::Receiver;
use std::time::Duration;

use crate::config::ScoringConfig;

use super::dsp::{
    EffectProcessor, MultiEqDsp, build_chorus, build_reverb, build_variation,
};
use std::collections::HashMap;

use crate::midi::effect_params::variation_type::XGVariationType;
use crate::midi::ram::xg::multi_eq::EQBand;

use super::AudioShared;
use super::sink::{AudioSink, VecBufferSink};
use super::tone_generator::ToneGenerator;
use super::tone_generator::ToneGeneratorStatus;
use super::AudioRenderActions;

pub struct AudioRender {
    pub tone_generators: Box<[ToneGenerator]>,
    pub rx: Receiver<AudioRenderActions>,
    pub max_polyphony: u16,
    /// Shared effect/system parameters (None until the Init event arrives)
    pub shared: Option<AudioShared>,
    /// Audio output target
    pub sink: Box<dyn AudioSink>,
    /// Target sample rate (the tone_generator chain already operates at this rate)
    pub sample_rate: f32,
    /// DC offset blockers on the master bus (XG Spec: serial chains introduce DC)
    pub dc_l: super::dsp::core::dc_blocker::DcBlocker,
    pub dc_r: super::dsp::core::dc_blocker::DcBlocker,
    /// Master-bus DC blocking enabled (config: audio.dc_blocker, default true)
    pub dc_enabled: bool,


    // ── System effect instances (stage 3) ──
    pub reverb: Box<dyn EffectProcessor>,
    pub chorus: Box<dyn EffectProcessor>,
    pub variation: Box<dyn EffectProcessor>,
    pub multi_eq: MultiEqDsp,
    /// Parameter cache (type + parameters, rebuilt only on change)
    pub reverb_key: (u8, u8, [u16; 16]),
    pub chorus_key: (u8, u8, [u16; 16]),
    pub variation_key: (u8, u8, [u16; 16]),
    pub multi_eq_key: (u8, EQBand, EQBand, EQBand, EQBand, EQBand),

    // ── Insertion effect instance cache (03 nn → processor) ──
    pub insertion_instances: HashMap<u8, Box<dyn EffectProcessor>>,
    pub insertion_key: HashMap<u8, (u8, u8, [u16; 16])>,

    /// NoteOffs suspended while CC#64 sustain is held (keyed by part id).
    /// Each suspended entry corresponds to one NoteOff; on pedal release (or
    /// CC#123 All Notes Off / CC#120 All Sound Off) the entries are released.
    pub sustain_held: HashMap<usize, Vec<crate::midi::note::Note>>,
    pub dbg_frames: u64,
    /// Monotonic NoteOn counter: every NoteOn assigns a fresh id shared by all
    /// of its element voices (dual-element group release on NoteOff).
    pub note_on_counter: u64,
    /// CC#66 sostenuto: notes that were already sounding when the pedal was
    /// pressed (only these are held); keyed by part id.
    pub sostenuto_active: HashMap<usize, Vec<crate::midi::note::Note>>,
    /// Sostenuto-suspended NoteOffs (released on pedal release / CC#123/120).
    pub sostenuto_held: HashMap<usize, Vec<crate::midi::note::Note>>,

    /// Idle-voice buffer: a just-released voice is skipped while it has been
    /// idle for less than this window (gives it breathing room before reuse;
    /// falls back to any idle voice when the pool is exhausted).
    idle_buffer: Duration,

    /// Watchdog idle timer: accumulates rendered time while every tone
    /// generator is Idle. Once it reaches `sleep_delay` the render thread may
    /// sleep (event-driven wakeup) instead of spinning on silence.
    pub idle_elapsed: Duration,
    /// Watchdog threshold: total silence before sleeping (config
    /// audio.sleep_delay_ms; zero disables sleeping).
    pub sleep_delay: Duration,

    pub debug_mode: bool,
}

impl AudioRender {
    pub fn new(
        count: usize,
        max_polyphony: u16,
        source_sample_rate: f32,
        target_sample_rate: f32,
        scoring: ScoringConfig,
        debug_mode: bool,
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
            shared: None,
            dc_l: super::dsp::core::dc_blocker::DcBlocker::new(),
            dc_r: super::dsp::core::dc_blocker::DcBlocker::new(),
            dc_enabled: true,

            sink: Box::new(VecBufferSink::new()),
            sample_rate: target_sample_rate,
            reverb: build_reverb(target_sample_rate, &[0; 16]),
            chorus: build_chorus(target_sample_rate, &[0; 16]),
            variation: build_variation(
                XGVariationType::NoEffect,
                &[0; 16],
                target_sample_rate,
            ),
            multi_eq: MultiEqDsp::new(),
            reverb_key: (0, 0, [0; 16]),
            chorus_key: (0, 0, [0; 16]),
            variation_key: (0, 0, [0; 16]),
            multi_eq_key: (0, EQBand::default(), EQBand::default(), EQBand::default(), EQBand::default(), EQBand::default()),
            insertion_instances: HashMap::new(),
            insertion_key: HashMap::new(),
            sustain_held: HashMap::new(),
            dbg_frames: 0,
            note_on_counter: 0,
            sostenuto_active: HashMap::new(),
            sostenuto_held: HashMap::new(),
            idle_elapsed: Duration::ZERO,
            sleep_delay: Duration::from_secs(2),
            idle_buffer: Duration::from_millis(50),

            debug_mode,
        }
    }

    /// 替换输出后端 (启动时按 cfg.audio.engine 选择)
    pub fn set_sink(&mut self, sink: Box<dyn AudioSink>) {
        self.sink = sink;
    }

    /// 输出一帧块积累的样本到后端
    pub fn flush(&mut self) {
        self.sink.flush();
    }

    pub fn get_current_polyphony(&self) -> usize {
        self.tone_generators
            .iter()
            .filter(|&t| t.status == ToneGeneratorStatus::Running)
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

    /// Set the watchdog sleep delay (config audio.sleep_delay_ms; zero = never sleep).
    pub fn set_sleep_delay(&mut self, delay: Duration) {
        self.sleep_delay = delay;
    }

    /// Allocate an idle voice for a new note-on.
    ///
    /// Sequential scan (no random start): pass 1 prefers idle voices whose
    /// release finished long enough ago (`idle_buffer` — the just-released
    /// voice gets breathing room); pass 2 falls back to any idle voice.
    /// Returns None when every voice is busy (caller then steals).
    pub(crate) fn find_idle_voice(&mut self) -> Option<usize> {
        let n = self.tone_generators.len();
        if n == 0 {
            return None;
        }
        let idle = |t: &ToneGenerator| t.status == ToneGeneratorStatus::Idle;
        // Pass 1: buffered (long-idle) voices only
        for (idx, t) in self.tone_generators.iter().enumerate() {
            if idle(t) && t.idle_since.elapsed() >= self.idle_buffer {
                return Some(idx);
            }
        }
        // Pass 2: any idle voice (just-released ones accepted under pressure)
        for (idx, t) in self.tone_generators.iter().enumerate() {
            if idle(t) {
                return Some(idx);
            }
        }
        None
    }

    /// Allocate the companion voice of a dual-element note right after the
    /// first element's slot (associative placement: the two element voices
    /// stay adjacent — cache-friendly and keeps element order predictable).
    /// Falls back to the regular idle search when the adjacent slot is busy.
    pub(crate) fn find_adjacent_idle(&mut self, prev: usize) -> Option<usize> {
        let n = self.tone_generators.len();
        if n == 0 {
            return None;
        }
        let next = (prev + 1) % n;
        if self.tone_generators[next].status == ToneGeneratorStatus::Idle
            && self.tone_generators[next].idle_since.elapsed() >= self.idle_buffer
        {
            return Some(next);
        }
        if self.tone_generators[next].status == ToneGeneratorStatus::Idle {
            return Some(next);
        }
        self.find_idle_voice()
    }

    /// Watchdog check: does the render loop still need to run?
    ///
    /// True while any tone generator is active (Running/Releasing — including
    /// sustain-held notes and release tails), or while the idle grace window
    /// hasn't elapsed yet (lets reverb/chorus tails fade before sleeping).
    pub fn needs_render(&self) -> bool {
        if self.sleep_delay.is_zero() {
            return true;
        }
        self.tone_generators
            .iter()
            .any(|t| t.status != ToneGeneratorStatus::Idle)
            || self.idle_elapsed < self.sleep_delay
    }

    /// Advance the watchdog idle timer; called once per rendered frame.
    /// `active` = any tone generator is still sounding.
    pub(crate) fn tick_idle_watchdog(&mut self, active: bool) {
        if active {
            self.idle_elapsed = Duration::ZERO;
        } else if !self.sleep_delay.is_zero() {
            self.idle_elapsed += Duration::from_secs_f32(1.0 / self.sample_rate);
        }
    }

    /// Sleep until an audio action arrives (event-driven wakeup). Incoming
    /// actions are processed immediately (same handler as the render loop).
    /// Returns true if an action was processed (render loop should resume).
    pub fn sleep_idle(&mut self, timeout: Duration) -> bool {
        match self.rx.recv_timeout(timeout) {
            Ok(ev) => {
                self.handle_action(ev);
                true
            }
            Err(_) => false,
        }
    }
}
