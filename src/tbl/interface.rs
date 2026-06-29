use std::io::Error;

pub trait TBLFile {
    // TODO: Map to engine::VoiceManager
    fn load_file(bin_path: String, wave_path: String) -> Result<Self::Output, Error>;
}