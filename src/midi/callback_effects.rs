use crate::midi::engine::MidiResetMode;
use crate::midi::ram::MemoryAddr;

#[derive(Debug)]
pub enum MIDICallbackEffects {
    NoEffect,
    // SysEx: F0 43 1n 27 30 00 00 mm ll cc F7
    ChangeMasterTuning {
        tuning: u16,
    },
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
        set: u8,
        bank_msb: u8,
        program: u8,
    },
    ResetDrumSetup {
        setup_id: u8,
    },
    ResetAllParameter,
    InsertionEffectON {
        for_part: u8,
        eff_id: u8,
    },
    InsertionEffectOFF {
        for_part: u8,
        eff_id: u8,
    },
    ChannelResetAllController {
        part_id: usize,
    },
    AllSoundOFF {
        part_id: usize,
    },
    AllNotesOFF {
        part_id: usize,
    },
}
