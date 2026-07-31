use super::MemoryAddr;

#[derive(Debug)]
pub enum RAMCallbackEffects {
    NoEffect,
    ChangeProgram {
        part_id: u8,
        program: u8,
    },
    BackupBankSet {
        pard_id: u8,
        bank_msb: u8,
        bank_lsb: u8,
        program: u8,
    },
    SetRAM {
        addr: MemoryAddr,
        value: u8,
    },
}
