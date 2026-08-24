use crate::midi::MIDICallbackEffects;

use super::super::engine::Engine;

pub trait Event {
    fn parse(e: &mut Engine, data: &[u8]) -> Vec<MIDICallbackEffects>;
}
