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
    /// CC#64 sustain pedal state change (on = pressed): on release, the
    /// suspended NoteOffs of the part must be released.
    SustainChange {
        part: Arc<DoubleBuffered<Part>>,
        on: bool,
    },
    /// CC#66 sostenuto pedal state change: on press, snapshot the currently
    /// sounding notes; on release, release the suspended NoteOffs.
    SostenutoChange {
        part: Arc<DoubleBuffered<Part>>,
        on: bool,
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
