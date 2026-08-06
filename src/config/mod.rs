mod interface;

mod audio;
mod audio_errors;
mod midi;
mod midi_errors;
mod sound_module;
mod sound_module_errors;

use std::{env, error::Error, fs, path::Path};

pub use audio::{AudioConfig, AudioDepth, AudioEngine};
pub use interface::ConfigObject;
pub use midi::{MidiConfig, MidiInputEngine, ScoringConfig};
pub use sound_module::SoundModuleConfig;

use serde::{Deserialize, Serialize};
use wd_log::{Level, set_level};

use audio_errors::AudioConfigError;
use midi_errors::MidiConfigError;
use sound_module_errors::SoundModuleError;

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

#[derive(Debug, Deserialize, Clone, Serialize)]
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
        let path = Path::new(&path);

        let mut config_pathes = vec![path.to_path_buf()];
        let xdg = env::var("XDG_CONFIG_HOME")?;
        config_pathes.push(Path::new(&xdg).join("madaha").join(path));
        config_pathes.push(Path::new("/etc/madaha").join(path));

        let mut content = String::new();
        let mut last_err = None;
        for p in config_pathes {
            match fs::read_to_string(p) {
                Ok(c) => {
                    content = c;
                    break;
                }
                Err(err) => last_err = Some(err),
            }
        }
        if let Some(err) = last_err {
            return Err(Box::new(err));
        }

        let config: Config = toml::from_str(&content)?;

        config.log_level();
        Ok(config)
    }

    pub fn generate_default(path: String) -> Result<(), Box<dyn Error>> {
        let content = toml::to_string(&Self::new())?;
        fs::write(path, content)?;

        Ok(())
    }
}

impl ConfigObject<ConfigError> for Config {
    fn new() -> Self {
        Self {
            log_level: default_log_level(),
            sound_module: SoundModuleConfig::new(),
            audio: AudioConfig::new(),
            midi: MidiConfig::new(),
        }
    }

    fn check(&self) -> Result<(), ConfigError> {
        self.audio.check().map_err(ConfigError::Audio)?;
        self.midi.check().map_err(ConfigError::Midi)?;
        self.sound_module
            .check()
            .map_err(ConfigError::SoundModule)?;
        Ok(())
    }
}
