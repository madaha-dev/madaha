//! ALSA output sink (blocking writes provide the audio clock)
use crate::config::{AudioConfig, AudioDepth};

use crate::audio::sink::AudioSink;
use super::encode_frame;
use super::sample_bytes;

pub struct AlsaSink {
    pcm: alsa::PCM,
    /// Accumulated interleaved f32 frames (block size)
    buffer: Vec<f32>,
    block_frames: usize,
    channels: u32,
    depth: AudioDepth,
    /// Raw byte output for the current block (encoded once at flush)
    byte_buf: Vec<u8>,
}

impl AlsaSink {
    pub fn open(cfg: &AudioConfig) -> Result<Self, String> {
        use alsa::pcm::{Access, Format, HwParams};
        use alsa::{Direction, ValueOr};

        let device = cfg
            .device
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let pcm = alsa::PCM::new(&device, Direction::Playback, false)
            .map_err(|e| format!("alsa open {}: {e}", device))?;
        {
            let hwp = HwParams::any(&pcm).map_err(|e| format!("alsa hwparams: {e}"))?;
            hwp.set_channels(cfg.channels).map_err(|e| format!("alsa channels: {e}"))?;
            hwp.set_rate(cfg.sample_rate as u32, ValueOr::Nearest)
                .map_err(|e| format!("alsa rate: {e}"))?;
            let format = match cfg.depth.clone() {
                AudioDepth::U8bit => Format::U8,
                AudioDepth::S16bit => Format::S16LE,
                AudioDepth::S24bit => Format::S24LE,
                AudioDepth::F32bit => Format::FloatLE,
            };
            hwp.set_format(format).map_err(|e| format!("alsa format: {e}"))?;
            hwp.set_access(Access::RWInterleaved).map_err(|e| format!("alsa access: {e}"))?;
            let buf_frames = cfg.alsa_buffer_frames.unwrap_or(cfg.buffer_size as u32 * 4) as i64;
            hwp.set_buffer_size(buf_frames)
                .map_err(|e| format!("alsa buffer: {e}"))?;
            pcm.hw_params(&hwp).map_err(|e| format!("alsa hw_params: {e}"))?;
        }

        Ok(Self {
            pcm,
            buffer: Vec::with_capacity(cfg.buffer_size as usize * 2),
            block_frames: cfg.buffer_size as usize,
            channels: cfg.channels,
            depth: cfg.depth,
            byte_buf: Vec::with_capacity(cfg.buffer_size as usize * 2 * 4),
        })
    }
}

impl AudioSink for AlsaSink {
    fn push_frame(&mut self, left: f32, right: f32) {
        self.buffer.push(left);
        self.buffer.push(right);
    }

    fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        // Encode the whole block according to the configured depth
        self.byte_buf.clear();
        for chunk in self.buffer.chunks_exact(2) {
            encode_frame(self.depth, chunk[0], chunk[1], &mut self.byte_buf);
        }
        // Pad a partial block with silence (the loop renders exactly block_frames,
        // but keep this robust against early drain flushes)
        while self.buffer.len() / 2 < self.block_frames {
            for _ in 0..self.channels {
                self.byte_buf.extend(std::iter::repeat(0).take(sample_bytes(self.depth)));
            }
            self.buffer.push(0.0);
            self.buffer.push(0.0);
        }
        let _ = self.buffer.drain(..);

        let frames = (self.byte_buf.len() / (self.channels as usize * sample_bytes(self.depth)))
            as alsa::pcm::Frames;
        // Blocking write; handles underrun recovery on XRUN
        let io = self.pcm.io_bytes();
        let written = io.writei(&self.byte_buf).unwrap_or(frames as usize);
        if written as i64 != frames {
            // XRUN: recover by resetting the device
            let _ = self.pcm.prepare();
        }
        self.byte_buf.clear();
    }

    fn frame_count(&self) -> usize {
        self.buffer.len() / 2
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Drop for AlsaSink {
    fn drop(&mut self) {
        let _ = self.pcm.drain();
    }
}
