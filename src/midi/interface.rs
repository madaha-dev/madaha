use super::MIDICallbackEffects;
use super::note::Note;
use super::sysex::ManufacturerId;

pub trait EventParser {
    fn on_sysex(&mut self, _mfid: ManufacturerId, _data: Box<[u8]>) -> Vec<MIDICallbackEffects> {
        vec![]
    }
    fn on_controller(&mut self, channel: u8, cc: u8, value: u8) -> Vec<MIDICallbackEffects>;
    fn on_program_change(&mut self, channel: u8, program: u8) -> Vec<MIDICallbackEffects>;
    fn on_pitchbend(&mut self, channel: u8, value: u16) -> Vec<MIDICallbackEffects>;
    fn on_rpn(&mut self, channel: u8, param: u16, value: u16) -> Vec<MIDICallbackEffects>;
    fn on_nrpn(&mut self, channel: u8, param: u16, value: u16) -> Vec<MIDICallbackEffects>;
    #[allow(dead_code)] // trait default no-op; implementers may override (NoteOn handled in engine)
    fn on_note(&mut self, _channel: u8, _note: Note, _velocity: u8) -> Vec<MIDICallbackEffects> {
        vec![]
    }
    fn on_cat(&mut self, channel: u8, pressure: u8) -> Vec<MIDICallbackEffects>;
    fn on_pat(&mut self, channel: u8, note: Note, pressure: u8) -> Vec<MIDICallbackEffects>;
}

// Called in realtime
pub trait PitchGetter {
    fn get_coarse(&self) -> i8 {
        panic!("should not called here")
    }
    fn get_delta_pitch(&self, _note: Note) -> f32 {
        panic!("should not called here")
    }
}
