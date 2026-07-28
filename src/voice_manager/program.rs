use std::ops::{Index, IndexMut};

use super::drum_setup::DEFAULT_DRUM_SETUP;
use super::drum_setup::DrumSetupEntry;
use super::keys::Key;

// for some drum kits, not all keys has sounds.
#[derive(Debug, Clone, Copy)]
pub struct Program([Option<Key>; 128]);

impl From<[Option<Key>; 128]> for Program {
    fn from(value: [Option<Key>; 128]) -> Self {
        Self(value)
    }
}

impl Program {
    pub fn to_drum_setup_entry(&self) -> [DrumSetupEntry; 79] {
        let mut _data = [DEFAULT_DRUM_SETUP; 79];

        for (i, item) in self.0.get(0x0C..0x5B).unwrap().iter().enumerate() {
            if let Some(sm) = item {
                if let Some(ds) = sm.drum_setup {
                    _data[i] = ds;
                }
            }
        }

        _data
    }
}

impl Index<usize> for Program {
    type Output = Option<Key>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Program {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
