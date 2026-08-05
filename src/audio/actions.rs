use std::sync::Arc;

use crate::double_buffer::DoubleBuffered;
use crate::midi::{Part, note::Note};

use super::AudioShared;

#[derive(Debug)]
pub enum AudioRenderActions {
    /// Shared effect/system parameters (double-buffered reference), sent after the engine starts
    Init {
        shared: AudioShared,
    },

    KillAll {
        part: Arc<DoubleBuffered<Part>>,
    },
    ReleaseAll {
        part: Arc<DoubleBuffered<Part>>,
    },

    Play {
        note: Note,
        vel: u8,
        part: Arc<DoubleBuffered<Part>>,
    },
    Release {
        note: Note,
        part: Arc<DoubleBuffered<Part>>,
    },
}
