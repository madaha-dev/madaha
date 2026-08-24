//! cpal output sink (portable backend)
//!
//! Model: the render thread pushes blocks into a lock-free ring (same as the
//! old PipeWire sink); cpal's device callback thread pulls frames from the
//! ring at the hardware clock rate (underrun padded with silence). The actual
//! negotiated rate is reported through `AudioSink::rate()` so the render
//! virtual clock follows it (pitch correctness depends on this).

use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

use wd_log::{log_debug_ln, log_info_ln, log_warn_ln};

use crate::config::AudioConfig;

use super::ringbuf::SpscRing;
use crate::audio::sink::AudioSink;

pub struct CpalSink {
    _stream: cpal::Stream,
    _ring: Arc<SpscRing>,
    /// Accumulated interleaved f32 frames (one render block)
    buffer: Vec<f32>,
    /// Sample rate actually used for the stream (requested or device default)
    actual_rate: u32,
    debug_mode: bool,
    dump_file: Option<File>,
}

impl CpalSink {
    /// Build + play a stream for `config`; the callback drains `ring`.
    fn build_stream(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        ring: &Arc<SpscRing>,
        channels: usize,
    ) -> Result<cpal::Stream, String> {
        use cpal::traits::{DeviceTrait, StreamTrait};

        let ring_cb = ring.clone();
        let data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let frames = data.len() / channels;
            if frames == 0 {
                return;
            }
            let read = ring_cb.read(&mut data[..frames * channels]);
            if read < frames {
                // Underrun (startup or render slower than realtime): pad silence
                for v in &mut data[read * channels..frames * channels] {
                    *v = 0.0;
                }
            }
        };
        let err_fn = |e: cpal::Error| {
            eprintln!("cpal stream error: {e}");
        };
        let stream = device
            .build_output_stream::<f32, _, _>(*config, data_fn, err_fn, None)
            .map_err(|e| format!("cpal build output stream: {e}"))?;
        stream
            .play()
            .map_err(|e| format!("cpal stream play: {e}"))?;
        Ok(stream)
    }

    pub fn open(cfg: &AudioConfig) -> Result<Self, String> {
        use cpal::traits::{DeviceTrait, HostTrait};

        let host = cpal::default_host();
        let device = match &cfg.device {
            Some(name) => host
                .output_devices()
                .map_err(|e| format!("cpal list output devices: {e}"))?
                .find(|d| d.to_string() == *name)
                .ok_or_else(|| format!("cpal output device not found: {name}"))?,
            None => host
                .default_output_device()
                .ok_or_else(|| "cpal: no default output device".to_string())?,
        };
        log_info_ln!("cpal output device: {device}");

        // The synth bus is stereo; pin the stream to 2 channels.
        let channels = 2usize;
        if cfg.channels != 2 {
            log_warn_ln!("cpal sink is stereo; ignoring channels={}", cfg.channels);
        }

        // Small ring: the write-side backpressure paces the render thread to
        // the device clock. A large ring (e.g. 65536 frames = 1.36s) fills up
        // with the pre-note silence and delays every NoteOn by that amount.
        // With ~4 blocks the latency stays in the single-digit ms range while
        // still absorbing scheduler jitter.
        let ring = Arc::new(SpscRing::new((cfg.buffer_size as usize * 4).max(64)));

        let requested = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: cfg.sample_rate,
            buffer_size: cpal::BufferSize::Fixed(cfg.buffer_size),
        };
        let (stream, actual_rate) = match Self::build_stream(&device, &requested, &ring, channels) {
            Ok(stream) => (stream, requested.sample_rate),
            Err(e) => {
                log_warn_ln!(
                    "cpal build at {}Hz/{:?} failed ({e}); falling back to device default",
                    cfg.sample_rate,
                    requested.buffer_size
                );
                let def = device
                    .default_output_config()
                    .map_err(|e| format!("cpal default output config: {e}"))?;
                let fallback = cpal::StreamConfig {
                    channels: channels as u16,
                    sample_rate: def.sample_rate(),
                    buffer_size: cpal::BufferSize::Default,
                };
                log_warn_ln!("cpal using device default: {}Hz", fallback.sample_rate);
                let stream = Self::build_stream(&device, &fallback, &ring, channels)?;
                (stream, fallback.sample_rate)
            }
        };

        Ok(Self {
            _stream: stream,
            _ring: ring,
            buffer: Vec::with_capacity(cfg.buffer_size as usize * 2),
            actual_rate,
            debug_mode: false,
            dump_file: None,
        })
    }
}

impl AudioSink for CpalSink {
    fn rate(&self) -> u32 {
        self.actual_rate
    }

    fn push_frame(&mut self, left: f32, right: f32) {
        self.buffer.push(left);
        self.buffer.push(right);
    }

    fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        self._ring.write(&self.buffer);
        // Dump the rendered block (f32 LE, interleaved) for offline analysis;
        // same format as the old ALSA sink so existing tooling keeps working.
        if let Some(f) = self.dump_file.as_mut() {
            let data: Vec<u8> = self.buffer.iter().flat_map(|i| i.to_le_bytes()).collect();
            let _ = f.write_all(data.as_slice()).and_then(|_| f.flush());
        }
        self.buffer.clear();
    }

    fn frame_count(&self) -> usize {
        self.buffer.len() / 2
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn set_debug(&mut self, debug_mode: bool) {
        self.debug_mode = debug_mode;
        self.dump_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("/tmp/madaha.dmp")
            .ok();
        log_debug_ln!("cpal debug dump activated")
    }
}
