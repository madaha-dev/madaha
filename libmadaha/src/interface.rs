use crate::LoadedModule;

use super::errors::LoadError;

pub trait SoundModule {
    fn load_data(paramfile: String, wavefile: String) -> Result<LoadedModule, LoadError>;
    fn check_header(header: &[u8]) -> Result<(), LoadError>;
}
