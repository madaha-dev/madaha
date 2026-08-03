use crate::midi::consts::{DEFAULT_COARSE_TUNING, DEFAULT_FINE_TUNING};

#[derive(Debug, Copy, Clone)]
pub struct RPN {
    /// Pitch bend sensitivity in semitones (RPN#0)
    pub pitchbend_sensitivity: u8,
    /// Pitch bend sensitivity in cents (RPN#0 LSB, for fine tuning)
    pub pitchbend_cents: u8,
    /// Coarse tuning (RPN#2, -64~+63 semitones, 0x40=center)
    pub coarse: u8,
    /// Fine tuning MSB (RPN#1, combined with fine_lsb for 14-bit)
    pub fine_msb: u8,
    /// Fine tuning LSB (lower 7 bits of fine tuning)
    pub fine_lsb: u8,
    /// Microtuning bank select (RPN#3)
    pub tuning_bank_select: u8,
    /// Microtuning program select (RPN#4)
    pub tuning_prog_select: u8,
}

impl RPN {
    pub fn new() -> Self {
        Self {
            pitchbend_sensitivity: 2,
            pitchbend_cents: 0,
            fine_msb: DEFAULT_FINE_TUNING,
            fine_lsb: 0,
            coarse: DEFAULT_COARSE_TUNING,
            tuning_bank_select: 0,
            tuning_prog_select: 0,
        }
    }

    // in cents
    pub fn get_pitch_bend_sensitivity(&self) -> f32 {
        (self.pitchbend_sensitivity as f32 + self.pitchbend_cents as f32 / 128.0) * 100.0
    }

    pub fn get(&self, param: u16) -> u16 {
        match param {
            0x0000 => (self.pitchbend_sensitivity as u16) << 7 | self.pitchbend_cents as u16,
            0x0001 => (self.fine_msb as u16) << 7 | self.fine_lsb as u16,
            0x0002 => (self.coarse as u16) << 7,
            0x0003 => (self.tuning_bank_select as u16) << 7,
            0x0004 => (self.tuning_prog_select as u16) << 7,

            _ => 0xFFFF,
        }
    }
}
