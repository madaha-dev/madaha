use crate::{config::Config, engine::MidiResetMode, voice_manager::{errors::LoadError, voice::voice::Voice}};

pub trait SoundModule<T> {
    fn load_data(cfg: &Config) -> Result<(T, Box<[u8]>), LoadError>;
    fn check_header(header: &[u8]) -> Result<(), LoadError>;
    fn get_voice(opts: &GetVoiceOpts) -> Option<Voice>;
}

pub struct GetVoiceOpts {
    pub reset_mode: MidiResetMode,
    pub is_drumkit: bool,
    pub msb: u8,
    pub lsb: u8,
    pub prog: u8,
    pub note: u8,
    pub vel: u8,
}
