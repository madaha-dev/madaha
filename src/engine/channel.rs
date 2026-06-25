use crate::engine::{
    consts::{DEFAULT_COARSE_TUNING, DEFAULT_FINE_TUNING},
    controller::Controller,
};

use super::consts::PITCH_BEND_MIDDLE;

#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub _channel: usize,

    pub bank_msb: u8,
    pub bank_lsb: u8,
    pub program: u8,
    pub pitchbend: u16,
    pub pitchbend_sensitivity: u8,
    pub pitchbend_cents: u8,
    pub coarse: u8,
    pub fine_msb: u8,
    pub fine_lsb: u8,
    pub tuning_bank_select: u8,
    pub tuning_prog_select: u8,

    pub controller: Controller,

    pub data_entry_select: DataEntrySelect,

    pub drum_setup: u8,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            _channel: 0xFF,

            bank_msb: 0,
            bank_lsb: 0,
            program: 0,
            pitchbend: PITCH_BEND_MIDDLE,
            pitchbend_sensitivity: 2,
            pitchbend_cents: 0,
            fine_msb: DEFAULT_FINE_TUNING,
            fine_lsb: 0,
            coarse: DEFAULT_COARSE_TUNING,
            tuning_bank_select: 0,
            tuning_prog_select: 0,

            controller: Controller::new(),

            data_entry_select: DataEntrySelect::None,

            drum_setup: 0,
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
