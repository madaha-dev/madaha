use crate::{
    audio::tone_generator::oscillator::InterpolatingMethods,
    config::{audio_errors::AudioConfigError, interface::ConfigObject},
};
use serde::Deserialize;
use strum_macros::EnumString;

#[derive(Debug, Deserialize, EnumString, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AudioEngine {
    Alsa,
    Pipewire,
    #[serde(alias = "pa")]
    PulseAudio,
    Jack,
}

#[derive(Debug, Deserialize, EnumString, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AudioDepth {
    #[serde(alias = "u8")]
    U8bit, // Unsigned 8 bit
    #[serde(alias = "s16")]
    S16bit, // Signed 16 bit
    #[serde(alias = "s24")]
    S24bit, // Signed 24 bit
    #[serde(alias = "f32")]
    F32bit, // Float 32 bit
}

fn default_audio_engine() -> AudioEngine {
    AudioEngine::Alsa
}

fn default_sample_rate() -> u32 {
    44100
}

fn default_audio_depth() -> AudioDepth {
    AudioDepth::S16bit
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

fn default_jack_client_name() -> String {
    "madaha".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioConfig {
    /// audio engine
    #[serde(default = "default_audio_engine")]
    pub engine: AudioEngine,

    /// sample rate
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    #[serde(default = "default_audio_depth")]
    pub depth: AudioDepth,

    #[serde(default = "default_buffer_size")]
    pub buffer_size: u32,
    // TODO: more params.
    #[serde(default = "default_interpolating")]
    pub interpolating: InterpolatingMethods,

    /// Output device name (None = system default; ALSA: "default", "hw:0,0" etc.)
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

    /// ALSA buffer frames (None = driver default; buffer_size is the block size)
    #[serde(default)]
    pub alsa_buffer_frames: Option<u32>,

    /// Jack client name (default "madaha")
    #[serde(default = "default_jack_client_name")]
    pub jack_client_name: String,
}

impl ConfigObject<AudioConfigError> for AudioConfig {
    fn check(&self) -> Result<(), AudioConfigError> {
        self.check_sample_rate()?;
        self.check_buffer_size()?;
        self.check_master_volume()?;
        self.check_alsa_buffer()?;
        Ok(())
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

    /// check ALSA buffer frames (must be a positive power of two if set)
    fn check_alsa_buffer(&self) -> Result<(), AudioConfigError> {
        if let Some(frames) = self.alsa_buffer_frames {
            if frames == 0 || !frames.is_power_of_two() {
                return Err(AudioConfigError::BadBufferSize {
                    buffer_size: frames,
                });
            }
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
