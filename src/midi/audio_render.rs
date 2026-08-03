// Audio render

use std::sync::{Arc, RwLock, RwLockReadGuard, mpsc::Receiver};

use crate::engine::{
    Engine, channel, event::MidiEvent, note::Note, tone_generator::ToneGeneratorStatus::Running
};

impl Engine {
    pub fn audio_render(engine: &Arc<RwLock<Self>>, rx: &Receiver<MidiEvent>) {
        let e = engine.read().unwrap();


    }
}


