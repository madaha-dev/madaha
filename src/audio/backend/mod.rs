//! Real-time audio backends: ALSA, PulseAudio, Jack, PipeWire
//!
//! - ALSA/PulseAudio: block writes in `flush()` (provides the audio clock)
//! - Jack/PipeWire: push into a ringbuffer; the callback thread reads it
//!
//! Sample format follows `AudioDepth` (all backends support f32; ALSA/Pulse
//! additionally convert to u8/s16/s24 when configured).

use crate::config::{AudioConfig, AudioDepth, AudioEngine};

pub mod alsa;
pub mod jack;
pub mod pipewire;
pub mod pulse;
mod ringbuf;

/// Create the sink selected by `cfg.audio.engine`
pub fn create_sink(cfg: &AudioConfig) -> Result<Box<dyn crate::audio::sink::AudioSink>, String> {
    use crate::audio::sink::GainSink;
    let raw: Box<dyn crate::audio::sink::AudioSink> = match cfg.engine {
        AudioEngine::Alsa => alsa::AlsaSink::open(cfg).map(|s| Box::new(s) as Box<dyn crate::audio::sink::AudioSink>),
        AudioEngine::PulseAudio => {
            pulse::PulseSink::open(cfg).map(|s| Box::new(s) as Box<dyn crate::audio::sink::AudioSink>)
        }
        AudioEngine::Jack => jack::JackSink::open(cfg).map(|s| Box::new(s) as Box<dyn crate::audio::sink::AudioSink>),
        AudioEngine::Pipewire => pipewire::PipewireSink::open(cfg)
            .map(|s| Box::new(s) as Box<dyn crate::audio::sink::AudioSink>),
    }?;
    Ok(Box::new(GainSink::new(raw, cfg.master_volume, cfg.soft_clip)))
}

/// Convert one stereo f32 frame into the target depth bytes (little endian)
pub fn encode_frame(depth: AudioDepth, l: f32, r: f32, out: &mut Vec<u8>) {
    match depth {
        AudioDepth::U8bit => {
            out.push(((l.clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0) as u8);
            out.push(((r.clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0) as u8);
        }
        AudioDepth::S16bit => {
            for v in [l, r] {
                out.extend_from_slice(&((v.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
            }
        }
        AudioDepth::S24bit => {
            for v in [l, r] {
                let s = (v.clamp(-1.0, 1.0) * 8388607.0) as i32;
                out.extend_from_slice(&[(s & 0xFF) as u8, ((s >> 8) & 0xFF) as u8, ((s >> 16) & 0xFF) as u8]);
            }
        }
        AudioDepth::F32bit => {
            for v in [l, r] {
                out.extend_from_slice(&v.clamp(-1.0, 1.0).to_le_bytes());
            }
        }
    }
}

/// Bytes per sample (per channel) for the depth
pub fn sample_bytes(depth: AudioDepth) -> usize {
    match depth {
        AudioDepth::U8bit => 1,
        AudioDepth::S16bit => 2,
        AudioDepth::S24bit => 3,
        AudioDepth::F32bit => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_s16_zero_is_zero() {
        let mut out = Vec::new();
        encode_frame(AudioDepth::S16bit, 0.0, 0.0, &mut out);
        assert_eq!(out, vec![0, 0, 0, 0]);
    }

    #[test]
    fn encode_f32_roundtrip() {
        let mut out = Vec::new();
        encode_frame(AudioDepth::F32bit, 0.5, -0.5, &mut out);
        assert_eq!(out.len(), 8);
        let l = f32::from_le_bytes([out[0], out[1], out[2], out[3]]);
        assert!((l - 0.5).abs() < 1e-6);
    }
}
