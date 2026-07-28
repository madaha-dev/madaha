use crate::engine::{
    consts::{DEFAULT_COARSE_TUNING, DEFAULT_FINE_TUNING, DRUM_CHANNEL_ID},
    controller::Controller,
    data_entry::DataEntrySelect,
    voice::{program::Program, voice_manager::VoiceManager},
};

use super::consts::PITCH_BEND_MIDDLE;

#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub _channel: usize,

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
    pub program_entry: Program,

    pub drum_setup: u8,

    pub prev_bank_msb: u8,
    pub prev_bank_lsb: u8,
    pub prev_program: u8,
}

impl Channel {
    pub fn new(channel: usize, vm: &VoiceManager) -> Self {
        Self {
            _channel: channel,

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
            program_entry: if channel == DRUM_CHANNEL_ID {
                vm.get_program(0x7F, 0, 0).unwrap()
            } else {
                vm.get_program(0, 0, 0).unwrap()
            },
            drum_setup: 0,

            prev_bank_lsb: 0,
            prev_bank_msb: 0,
            prev_program: 0,
        }
    }

    // on sysex
    pub fn reset(&mut self) {}
}
