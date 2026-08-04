use std::sync::{Arc, RwLock};

use crate::midi::{Part, note::Note};

#[derive(Debug)]
pub enum AudioRenderActions {
    KillAll {
        part: Arc<RwLock<Part>>,
    },
    ReleaseAll {
        part: Arc<RwLock<Part>>,
    },

    Play {
        note: Note,
        vel: u8,
        part: Arc<RwLock<Part>>,
    },
    Release {
        note: Note,
        part: Arc<RwLock<Part>>,
    },
}
