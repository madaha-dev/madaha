mod audio;
mod audio_errors;
mod tbl;

use std::{error::Error, fs};

pub use audio::AudioConfig;
pub use tbl::TBLConfig;

use serde::Deserialize;
use wd_log::{Level, log_debug_ln, set_level};

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// log level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// load tbl files
    pub tbl: TBLConfig,

    /// audio configs
    pub audio: AudioConfig,
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
