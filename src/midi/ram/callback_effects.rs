use crate::midi::engine::MidiResetMode;

use super::MemoryAddr;

#[derive(Debug)]
pub enum RAMCallbackEffects {
    NoEffect,
    ChangeProgram {
        part_id: usize,
        program: u8,
        bank_msb: u8,
        bank_lsb: u8,
    },
    BackupBankSet {
        part_id: usize,
        bank_msb: u8,
        bank_lsb: u8,
        program: u8,
    },
    SetRAM {
        addr: MemoryAddr,
        value: u8,
    },
    SetPartModeToRhythm {
        part_id: usize,
        drum_set_id: usize,
    },
    SetPartModeToMelodic {
        part_id: usize,
    },
    SetPartBankMSB {
        part_id: usize,
        bank_msb: u8,
    },
    CallPartModeChange {
        part_id: usize,
        set: u8,
    },
    ChangeResetMode {
        mode: MidiResetMode,
    },
}
