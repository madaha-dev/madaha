use crate::{
    audio::tone_generator::oscillator::InterpolatingMethods,
    config::{audio_errors::AudioConfigError, interface::ConfigObject},
};
use serde::Deserialize;
use strum_macros::EnumString;

#[derive(Debug, Deserialize, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum AudioEngine {
    Alsa,
    Pipewire,
    #[serde(alias = "pa")]
    PulseAudio,
    Jack,
}

#[derive(Debug, Deserialize, EnumString)]
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

#[derive(Debug, Deserialize)]
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
}

impl ConfigObject<AudioConfigError> for AudioConfig {
    fn check(&self) -> Result<(), AudioConfigError> {
        self.check_sample_rate()?;

        Ok(())
    }
}

impl AudioConfig {
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
