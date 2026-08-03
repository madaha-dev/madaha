use super::note::Note;
use super::ram::RAMCallbackEffects;
use super::sysex::ManufacturerId;

pub trait EventParser<T> {
    fn on_sysex(&mut self, _mfid: ManufacturerId, _data: Box<[u8]>) -> Vec<RAMCallbackEffects> {
        vec![]
    }
    fn on_controller(&mut self, channel: T, cc: u8, value: u8) -> Vec<RAMCallbackEffects>;
    fn on_program_change(&mut self, channel: T, program: u8) -> Vec<RAMCallbackEffects>;
    fn on_pitchbend(&mut self, channel: T, value: u16) -> Vec<RAMCallbackEffects>;
    fn on_rpn(&mut self, channel: T, param: u16, value: u16) -> Vec<RAMCallbackEffects>;
    fn on_nrpn(&mut self, channel: T, param: u16, value: u16) -> Vec<RAMCallbackEffects>;
    fn on_note(&mut self, _channel: T, _note: Note, _velocity: u8) -> Vec<RAMCallbackEffects> {
        vec![]
    }
    fn on_cat(&mut self, channel: T, pressure: u8) -> Vec<RAMCallbackEffects>;
    fn on_pat(&mut self, channel: T, note: Note, pressure: u8) -> Vec<RAMCallbackEffects>;
}
