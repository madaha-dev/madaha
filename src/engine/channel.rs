use crate::engine::{
    consts::{DEFAULT_COARSE_TUNING, DEFAULT_FINE_TUNING},
    controller::Controller,
};

use super::consts::PITCH_BEND_MIDDLE;

#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub bank_msb: u8,
    pub bank_lsb: u8,
    pub program: u8,
    pub pitchbend: u16,
    pub pitchbend_sensitivity: u8,
    pub coarse: u16,
    pub fine: u16,

    pub controller: Controller,

    pub data_entry_select: DataEntrySelect,
}

impl Channel {
    pub fn new(channel: u8) -> Self {
        Self {
            bank_msb: 0,
            bank_lsb: 0,
            program: 0,
            pitchbend: PITCH_BEND_MIDDLE,
            pitchbend_sensitivity: 2,
            fine: DEFAULT_FINE_TUNING,
            coarse: DEFAULT_COARSE_TUNING,

            controller: Controller::new(channel),

            data_entry_select: DataEntrySelect::None,
        }
    }

    // on sysex
    pub fn reset(&mut self) {}
}

#[derive(Debug, Clone, Copy)]
pub enum DataEntrySelect {
    None,
    RPN,
    NRPN,
}
