//! ALSA output sink (blocking writes provide the audio clock)
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

use wd_log::log_debug_ln;

use crate::config::{AudioConfig, AudioDepth};

use super::encode_frame;
use super::sample_bytes;
use crate::audio::sink::AudioSink;

pub struct AlsaSink {
    pcm: alsa::PCM,
    /// Sample rate after hw_params negotiation (may differ from cfg)
    pub actual_rate: u32,
    /// Format after hw_params negotiation (may differ from cfg.depth)
    pub actual_format: AudioDepth,
    /// Accumulated interleaved f32 frames (block size)
    buffer: Vec<f32>,
    /// Frames to accumulate before a blocking write. Small writes (e.g. 64
    /// frames) get garbled by pipewire's ALSA plugin (data misalignment with
    /// its quantum); buffering to a full quantum-sized chunk keeps it clean.
    flush_threshold: usize,
    channels: u32,
    /// Raw byte output for the current block (encoded once at flush)
    byte_buf: Vec<u8>,

    debug_mode: bool,

    dump_file: Option<File>,
}

impl AlsaSink {
    pub fn open(cfg: &AudioConfig) -> Result<Self, String> {
        use alsa::pcm::{Access, Format, HwParams};
        use alsa::{Direction, ValueOr};

        let device = cfg.device.clone().unwrap_or_else(|| "default".to_string());
        // Fail-fast probe with a non-blocking open: an unavailable/busy device
        // (e.g. hardware already claimed by pipewire) errors out immediately so
        // the upstream fallback (internal buffer sink) kicks in.
        let probe = alsa::PCM::new(&device, Direction::Playback, true)
            .map_err(|e| format!("alsa open {}: {e}", device))?;
        drop(probe);
        // Reopen in BLOCKING mode: the audio thread runs a blocking write loop,
        // so `writei` completes the whole chunk (a non-blocking handle returns
        // partial writes/EAGAIN, which garbled the stream and dropped data).
        let pcm = alsa::PCM::new(&device, Direction::Playback, false)
            .map_err(|e| format!("alsa open {}: {e}", device))?;
        {
            let hwp = HwParams::any(&pcm).map_err(|e| format!("alsa hwparams: {e}"))?;
            hwp.set_channels(cfg.channels)
                .map_err(|e| format!("alsa channels: {e}"))?;
            hwp.set_rate(cfg.sample_rate as u32, ValueOr::Nearest)
                .map_err(|e| format!("alsa rate: {e}"))?;
            let format = match cfg.depth.clone() {
                AudioDepth::U8bit => Format::U8,
                AudioDepth::S16bit => Format::S16LE,
                AudioDepth::S24bit => Format::S24LE,
                AudioDepth::F32bit => Format::FloatLE,
            };
            hwp.set_format(format)
                .map_err(|e| format!("alsa format: {e}"))?;
            hwp.set_access(Access::RWInterleaved)
                .map_err(|e| format!("alsa access: {e}"))?;
            let buf_frames = cfg.alsa_buffer_frames.unwrap_or(cfg.buffer_size as u32 * 4) as i64;
            hwp.set_buffer_size(buf_frames)
                .map_err(|e| format!("alsa buffer: {e}"))?;
            pcm.hw_params(&hwp)
                .map_err(|e| format!("alsa hw_params: {e}"))?;
        }
        // Read back the negotiated rate/format (pipewire's ALSA plugin can
        // return a different format than requested, which would garble output)
        let actual_rate = pcm
            .hw_params_current()
            .and_then(|h| h.get_rate())
            .unwrap_or(cfg.sample_rate as u32);
        let actual_format = match pcm
            .hw_params_current()
            .and_then(|h| h.get_format())
            .unwrap_or(Format::S16LE)
        {
            Format::U8 => AudioDepth::U8bit,
            Format::S16LE => AudioDepth::S16bit,
            Format::S24LE => AudioDepth::S24bit,
            Format::FloatLE => AudioDepth::F32bit,
            _ => cfg.depth.clone(),
        };

        Ok(Self {
            pcm,
            actual_rate,
            actual_format,
            buffer: Vec::with_capacity(cfg.buffer_size as usize * 2),
            // Write every render block (the blocking-mode writei + retry loop
            // handles partial writes cleanly; the old "small write garbling"
            // was caused by the non-blocking handle dropping data). Combined
            // with the real-time render throttle in the audio thread, small
            // continuous writes keep the pipewire buffer stable instead of
            // bursting 1024-frame chunks every 21ms.
            flush_threshold: cfg.buffer_size.max(1024) as usize,
            channels: cfg.channels,
            byte_buf: Vec::with_capacity(cfg.buffer_size as usize * 2 * 4),
            debug_mode: false,
            dump_file: None,
        })
    }
}

impl AlsaSink {
    /// Encode the accumulated frames with the NEGOTIATED format and write them
    /// with a blocking write loop.
    ///
    /// The PCM was opened non-blocking (fail-fast when the device is busy), so
    /// `writei` may return a partial write or EAGAIN instead of blocking. The
    /// loop with a short sleep emulates a blocking write; abort + prepare only
    /// on real errors (XRUN etc). Previously a partial write was treated as
    /// success and the device reset, dropping data → garbled/noisy output.
    fn write_pending(&mut self) {
        if self.buffer.is_empty() {
            return;
        }
        // Encode using the NEGOTIATED format (pipewire's ALSA plugin may return
        // a different format than the configured depth, and `writei` interprets
        // the bytes according to the negotiated format)
        let fmt = self.actual_format;
        self.byte_buf.clear();
        for chunk in self.buffer.chunks_exact(2) {
            encode_frame(fmt, chunk[0], chunk[1], &mut self.byte_buf);
        }
        let bytes_per_frame = self.channels as usize * sample_bytes(fmt);
        let io = self.pcm.io_bytes();
        let mut off = 0usize;
        while off < self.byte_buf.len() {
            match io.writei(&self.byte_buf[off..]) {
                Ok(n) => off += n * bytes_per_frame,
                Err(e) if e.errno() == libc::EAGAIN => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Err(_) => {
                    // XRUN or other error: recover by resetting the device
                    let _ = self.pcm.prepare();
                    break;
                }
            }
        }
        // when in debug mode, dump my output for offline analyze.
        if let Some(f) = self.dump_file.as_mut() {
            let data: Vec<u8> = self
                .buffer
                .iter()
                .map(|i| i.to_le_bytes())
                .flatten()
                .collect();
            let _ = f.write_all(data.as_slice()).and_then(|_| f.flush());
        }
        self.buffer.clear();
        self.byte_buf.clear();
    }
}

impl AudioSink for AlsaSink {
    fn push_frame(&mut self, left: f32, right: f32) {
        self.buffer.push(left);
        self.buffer.push(right);
    }

    fn flush(&mut self) {
        // Accumulate until the write threshold; small writes to pipewire's
        // ALSA plugin come out garbled.
        if self.buffer.len() / 2 < self.flush_threshold {
            return;
        }
        self.write_pending();
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
        log_debug_ln!("alsa debug dump activated")
    }
}

impl Drop for AlsaSink {
    fn drop(&mut self) {
        // Flush any remaining frames so a clean shutdown doesn't cut the tail
        // (the audio thread renders exact blocks, but keep this robust)
        self.write_pending();
        let _ = self.pcm.drain();
    }
}
