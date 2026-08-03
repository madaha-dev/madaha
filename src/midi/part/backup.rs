#[derive(Debug, Clone, Copy)]
pub struct BackupSets {
    pub msb: u8,
    pub lsb: u8,
    pub prog: u8,
}

impl BackupSets {
    pub fn new() -> Self {
        Self {
            msb: 0xFF,
            lsb: 0xFF,
            prog: 0xFF,
        }
    }

    pub fn set(&mut self, msb: u8, lsb: u8, prog: u8) {
        self.msb = msb;
        self.lsb = lsb;
        self.prog = prog;
    }

    pub fn reset(&mut self) {
        *self = Self::new()
    }
}
