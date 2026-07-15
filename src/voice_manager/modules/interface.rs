use crate::{config::Config, voice_manager::errors::LoadError};

pub trait SoundModule<T> {
    fn load_data(cfg: &Config) -> Result<(T, Box<[u8]>), LoadError>;
    fn check_header(header: &[u8]) -> Result<(), LoadError>;
    fn get_program(msb:u8, lsb:u8, prog: u8);
}
