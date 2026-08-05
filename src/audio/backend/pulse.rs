//! PulseAudio output sink
//!
//! Implemented via the ALSA PulseAudio plugin (`device = "pulse"`), which is
//! the standard Linux approach: PulseAudio ships an ALSA compatibility layer
//! that routes to the default Pulse server.
use crate::config::AudioConfig;

use super::alsa::AlsaSink;
use crate::audio::sink::AudioSink;

pub struct PulseSink {
    inner: AlsaSink,
}

impl PulseSink {
    pub fn open(cfg: &AudioConfig) -> Result<Self, String> {
        let mut pulse_cfg = cfg.clone();
        if pulse_cfg.device.is_none() {
            pulse_cfg.device = Some("pulse".to_string());
        }
        Ok(Self {
            inner: AlsaSink::open(&pulse_cfg)?,
        })
    }
}

impl AudioSink for PulseSink {
    fn push_frame(&mut self, left: f32, right: f32) {
        self.inner.push_frame(left, right);
    }

    fn flush(&mut self) {
        self.inner.flush();
    }

    fn frame_count(&self) -> usize {
        self.inner.frame_count()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
