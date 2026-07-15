use crate::engine::voice::{drum_setup::{DEFAULT_DRUM_SETUP, DrumSetupEntry}, samples::SampleMeta};

// for some drum kits, not all keys has sounds.
pub type Program = [Option<&'static SampleMeta>; 128];

pub fn to_drum_setup_entry(pg: Program) -> [DrumSetupEntry; 79] {
    let mut _data = [DEFAULT_DRUM_SETUP; 79];

    for (i, item) in pg.get(0x0C..0x5B).unwrap().iter().enumerate() {
        if let Some(sm) = item {
            if let Some(ds) = sm.drum_setup {
                _data[i] = ds;
            }
        }
    }

    _data
}
