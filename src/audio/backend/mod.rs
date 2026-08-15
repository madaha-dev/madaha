//! Real-time audio backend: cpal
//!
//! The cpal device callback thread pulls frames from a ring buffer (provides
//! the audio clock); the render thread pushes blocks in `flush()`. Sample
//! conversion is handled inside cpal; the sink always exchanges f32
//! interleaved frames.

use crate::audio::sink::AudioSink;
use crate::config::AudioConfig;

pub(crate) mod cpal;
pub(crate) mod ringbuf;

/// Create the cpal output sink
pub fn create_sink(cfg: &AudioConfig) -> Result<Box<dyn AudioSink>, String> {
    use crate::audio::sink::GainSink;
    let raw: Box<dyn AudioSink> = cpal::CpalSink::open(cfg)
        .map(|s| Box::new(s) as Box<dyn AudioSink>)?;
    Ok(Box::new(GainSink::new(raw, cfg.master_volume, cfg.soft_clip)))
}
