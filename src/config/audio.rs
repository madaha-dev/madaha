use crate::{
    audio::tone_generator::oscillator::InterpolatingMethods,
    config::{audio_errors::AudioConfigError, interface::ConfigObject},
};
use serde::{Deserialize, Serialize};

fn default_sample_rate() -> u32 {
    44100
}

fn default_buffer_size() -> u32 {
    64
}

fn default_interpolating() -> InterpolatingMethods {
    InterpolatingMethods::Linear
}

fn default_channels() -> u32 {
    2
}

fn default_master_volume() -> f32 {
    1.0
}

fn default_soft_clip() -> bool {
    true
}

fn default_dc_blocker() -> bool {
    true
}

fn default_loudness_norm() -> bool {
    true
}

fn default_target_lufs() -> f32 {
    -14.0
}

/// Watchdog sleep delay: 2s of total silence before the render thread sleeps
/// (long enough for reverb/chorus tails to fade out; too short cuts tails)
fn default_sleep_delay_ms() -> u64 {
    2000
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct AudioConfig {
    /// sample rate
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    #[serde(default = "default_buffer_size")]
    pub buffer_size: u32,
    // TODO: more params.
    #[serde(default = "default_interpolating")]
    pub interpolating: InterpolatingMethods,

    /// Output device name (None = system default)
    #[serde(default)]
    pub device: Option<String>,

    /// Output channel count (stereo = 2)
    #[serde(default = "default_channels")]
    pub channels: u32,

    /// Master output gain (linear, 0.05..=4.0, default 1.0)
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,

    /// Soft-clip the final output (tanh, prevents hard clipping, default true)
    #[serde(default = "default_soft_clip")]
    pub soft_clip: bool,

    /// DC offset blocker on the master bus (XG Spec: serial chains introduce DC,
    /// default true)
    #[serde(default = "default_dc_blocker")]
    pub dc_blocker: bool,

    /// Output-side dynamic loudness normalization (BS.1770 slow AGC toward a
    /// target LUFS). Keeps average loudness near the target while preserving
    /// note transients; the peak limiter still protects the loudest peaks.
    #[serde(default = "default_loudness_norm")]
    pub loudness_norm: bool,

    /// Target loudness for `loudness_norm` (LUFS, EBU R128 short-term).
    #[serde(default = "default_target_lufs")]
    pub target_lufs: f32,

    /// Watchdog sleep delay (ms): after every tone generator goes idle for
    /// this long, the render thread sleeps until a MIDI/audio event arrives
    /// (effect tails still get a grace window to fade). 0 disables sleeping.
    #[serde(default = "default_sleep_delay_ms")]
    pub sleep_delay_ms: u64,
}

impl ConfigObject<AudioConfigError> for AudioConfig {
    fn check(&self) -> Result<(), AudioConfigError> {
        self.check_sample_rate()?;
        self.check_buffer_size()?;
        self.check_master_volume()?;
        Ok(())
    }

    fn new() -> Self {
        Self {
            sample_rate: default_sample_rate(),
            buffer_size: default_buffer_size(),
            interpolating: default_interpolating(),
            device: None,
            channels: default_channels(),
            master_volume: default_master_volume(),
            soft_clip: default_soft_clip(),
            dc_blocker: default_dc_blocker(),
            loudness_norm: default_loudness_norm(),
            target_lufs: default_target_lufs(),
            sleep_delay_ms: default_sleep_delay_ms(),
        }
    }
}

impl AudioConfig {
    /// check master volume (linear gain range)
    fn check_master_volume(&self) -> Result<(), AudioConfigError> {
        if !(0.05..=4.0).contains(&self.master_volume) {
            return Err(AudioConfigError::BadMasterVolume {
                master_volume: self.master_volume,
            });
        }
        Ok(())
    }

    /// check sample rate
    fn check_sample_rate(&self) -> Result<(), AudioConfigError> {
        let sample_rate = [22050u32, 44100, 48000, 96000, 192000];
        if !sample_rate.contains(&self.sample_rate) {
            Err(AudioConfigError::BadSampleRate {
                sample_rate: self.sample_rate,
            })
        } else {
            Ok(())
        }
    }

    fn check_buffer_size(&self) -> Result<(), AudioConfigError> {
        if self.buffer_size.is_power_of_two() && self.buffer_size >= 64 {
            Ok(())
        } else {
            Err(AudioConfigError::BadBufferSize {
                buffer_size: self.buffer_size,
            })
        }
    }
}
