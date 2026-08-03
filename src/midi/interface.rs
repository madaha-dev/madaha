use super::note::Note;
use super::ram::RAMCallbackEffects;
use super::sysex::ManufacturerId;

pub trait EventParser {
    fn on_sysex(&mut self, _mfid: ManufacturerId, _data: Box<[u8]>) -> Vec<RAMCallbackEffects> {
        vec![]
    }
    fn on_controller(&mut self, channel: u8, cc: u8, value: u8) -> Vec<RAMCallbackEffects>;
    fn on_program_change(&mut self, channel: u8, program: u8) -> Vec<RAMCallbackEffects>;
    fn on_pitchbend(&mut self, channel: u8, value: u16) -> Vec<RAMCallbackEffects>;
    fn on_rpn(&mut self, channel: u8, param: u16, value: u16) -> Vec<RAMCallbackEffects>;
    fn on_nrpn(&mut self, channel: u8, param: u16, value: u16) -> Vec<RAMCallbackEffects>;
    fn on_note(&mut self, _channel: u8, _note: Note, _velocity: u8) -> Vec<RAMCallbackEffects> {
        vec![]
    }
    fn on_cat(&mut self, channel: u8, pressure: u8) -> Vec<RAMCallbackEffects>;
    fn on_pat(&mut self, channel: u8, note: Note, pressure: u8) -> Vec<RAMCallbackEffects>;
}
