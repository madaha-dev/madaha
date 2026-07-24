use crate::{LoadError, LoadedModule, SoundModule, SoundModuleType, yxg50};
use std::fs;

pub fn load(
    smtype: SoundModuleType,
    binfile: String,
    wavefile: String,
) -> Result<LoadedModule, LoadError> {
    let mut smtype = smtype;
    if smtype == SoundModuleType::Auto {
        let data = fs::read(&binfile).map_err(|e| LoadError::LoadBinTBLFailed {
            reason: e.to_string(),
        })?;
        let header = &data[0..8];
        smtype = check_header(header);
    }

    match smtype {
        SoundModuleType::Syxg50 => yxg50::BinTbl::load_data(binfile, wavefile),

        _ => Err(LoadError::InvalidBinFile),
    }
}

fn check_header(header: &[u8]) -> SoundModuleType {
    // TODO check header magic here.
    if yxg50::check_header(&header[0..4]) {
        return SoundModuleType::Syxg50;
    }

    //if yxg2006le::check_header(header) {
    //    return SoundModuleType::Syxg2006le
    //}

    SoundModuleType::NotSupported
}
