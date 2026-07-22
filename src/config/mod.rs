mod interface;

mod audio;
mod audio_errors;
mod midi;
mod midi_errors;
mod sound_module;
mod sound_module_errors;

use std::{error::Error, fs};

pub use audio::AudioConfig;
pub use midi::ScoringConfig;
pub use sound_module::{SoundModuleConfig, SoundModuleType};

use serde::Deserialize;
use wd_log::{Level, set_level};

use crate::config::audio_errors::AudioConfigError;
use crate::config::interface::ConfigObject;
use crate::config::midi::MidiConfig;
use crate::config::midi_errors::MidiConfigError;
use crate::config::sound_module_errors::SoundModuleError;

#[derive(Debug)]
pub enum ConfigError {
    Audio(AudioConfigError),
    Midi(MidiConfigError),
    SoundModule(SoundModuleError),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audio(e) => write!(f, "audio: {}", e),
            Self::Midi(e) => write!(f, "midi: {}", e),
            Self::SoundModule(e) => write!(f, "sound_module: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

fn default_log_level() -> String {
    "info".into()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// log level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// load tbl files
    pub sound_module: SoundModuleConfig,

    /// audio configs
    pub audio: AudioConfig,

    /// midi configs
    pub midi: MidiConfig,
}

impl Config {
    pub fn log_level(&self) {
        let level = match self.log_level.as_str() {
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "warning" => Level::WARN,
            "err" => Level::ERROR,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };
        set_level(level);
    }

    pub fn from_file(path: String) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;

        config.log_level();
        Ok(config)
    }
}

impl ConfigObject<ConfigError> for Config {
    fn check(&self) -> Result<(), ConfigError> {
        self.audio.check().map_err(ConfigError::Audio)?;
        self.midi.check().map_err(ConfigError::Midi)?;
        self.sound_module
            .check()
            .map_err(ConfigError::SoundModule)?;
        Ok(())
    }
}
