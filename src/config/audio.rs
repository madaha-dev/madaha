use crate::config::audio_errors::AudioConfigError;
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

fn default_max_polyphony() -> u16 {
    512
}

fn default_sample_rate() -> u32 {
    44100
}

fn default_audio_depth() -> AudioDepth {
    AudioDepth::S16bit
}

fn default_master_tune() -> f64 {
    440.0
}

#[derive(Debug, Deserialize)]
pub struct AudioConfig {
    /// audio engine
    #[serde(default = "default_audio_engine")]
    pub engine: AudioEngine,

    /// max polyphony
    #[serde(default = "default_max_polyphony")]
    pub max_polyphony: u16, // never over 512, or boom.

    /// sample rate
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    #[serde(default = "default_audio_depth")]
    pub depth: AudioDepth,

    #[serde(default = "default_master_tune")]
    pub master_tune: f64,
    // TODO: more params.
}

impl AudioConfig {
    /// check all config
    pub fn check(&self) -> Result<(), AudioConfigError> {
        self.check_max_polyphony()?;
        self.check_sample_rate()?;

        Ok(())
    }

    /// check max polyphony
    fn check_max_polyphony(&self) -> Result<(), AudioConfigError> {
        const LIMIT: u16 = 1024;
        if self.max_polyphony >= LIMIT {
            Err(AudioConfigError::TooHighPolyphony {
                max: self.max_polyphony,
                limit: LIMIT,
            })
        } else {
            Ok(())
        }
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
}
