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
        current_part_mode: u8,
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
        drum_set_id: u8,
    },
    SetPartModeToMelodic {
        part_id: usize,
    },
    SetPartModeToDrums {
        part_id: usize,
    },

    ChangeResetMode {
        mode: MidiResetMode,
    },
    SetDrumSetup {
        bank_msb: u8,
        program: u8,
    },
    ResetDrumSetup {
        setup_id: u8,
    },
    ResetAllParameter,
    InsertionEffectON {
        for_part: u8,
        eff_id: u8
    },
    InsertionEffectOFF {
        for_part: u8,
        eff_id: u8
    }
}

pub trait RAMCallEffectsFunc {
    fn no_effect(&self) {}
    fn set_drum_setup(&self, bank_msb: u8, program: u8);
    
}
