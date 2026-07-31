use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use std::ops::{Index, IndexMut};

/// XG Spec 2.0 Virtual Lead (VL) parameters (hi addr 0x09).
///
/// Contains per-part VL parameters for monophonic lead voice control,
/// including pressure, embouchure, tonguing, and other VL-specific
/// controller assignments and depths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiPartVL {
    // XG Spec 2.0
    // hi addr 0x09
    // mid addr for channel (0x00-0x0F)

    // lo addr 0x00
    /// Note assign mode (1=mono, other values=poly)
    pub note_assign: u8,

    // lo addr 0x02
    /// Note filter value
    pub note_filter: u8,
    /// Pressure → VL control number
    pub pressure_control_number: u8,
    /// Pressure → VL control depth
    pub pressure_control_depth: u8,
    /// Embouchure → VL control number
    pub embouchure_control_number: u8,
    /// Embouchure → VL control depth
    pub embouchure_control_depth: u8,
    /// Tonguing → VL control number
    pub tonguing_control_number: u8,
    /// Tonguing → VL control depth
    pub tonguing_control_depth: u8,
    /// Scream → VL control number
    pub scream_control_number: u8,
    /// Scream → VL control depth
    pub scream_control_depth: u8,
    /// Breath → VL control number
    pub breath_control_number: u8,
    /// Breath → VL control depth
    pub breath_control_depth: u8,
    /// Growl → VL control number
    pub growl_control_number: u8,
    /// Growl → VL control depth
    pub growl_control_depth: u8,
    /// Throat formant → VL control number
    pub throat_formant_control_number: u8,
    /// Throat formant → VL control depth
    pub throat_formant_control_depth: u8,
    /// Harmonic enhancer → VL control number
    pub harmonic_enhancer_control_number: u8,
    /// Harmonic enhancer → VL control depth
    pub harmonic_enhancer_control_depth: u8,
    /// Damping → VL control number
    pub damping_control_number: u8,
    /// Damping → VL control depth
    pub damping_control_depth: u8,
    /// Absorption → VL control number
    pub absorption_control_number: u8,
    /// Absorption → VL control depth
    pub absorption_control_depth: u8,
}

impl MultiPartVL {
    pub const fn new() -> Self {
        Self {
            note_assign: 1,
            note_filter: 0x7F,
            pressure_control_number: 0x00,
            pressure_control_depth: 0x40,
            embouchure_control_number: 0x00,
            embouchure_control_depth: 0x40,
            tonguing_control_number: 0x00,
            tonguing_control_depth: 0x40,
            scream_control_number: 0x00,
            scream_control_depth: 0x40,
            breath_control_number: 0x00,
            breath_control_depth: 0x40,
            growl_control_number: 0x00,
            growl_control_depth: 0x40,
            throat_formant_control_number: 0x00,
            throat_formant_control_depth: 0x40,
            harmonic_enhancer_control_number: 0x00,
            harmonic_enhancer_control_depth: 0x40,
            damping_control_number: 0x00,
            damping_control_depth: 0x40,
            absorption_control_number: 0x00,
            absorption_control_depth: 0x40,
        }
    }
}

impl Index<usize> for MultiPartVL {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0x00 => &self.note_assign,
            // 0x01: reserved
            0x02 => &self.note_filter,
            0x03 => &self.pressure_control_number,
            0x04 => &self.pressure_control_depth,
            0x05 => &self.embouchure_control_number,
            0x06 => &self.embouchure_control_depth,
            0x07 => &self.tonguing_control_number,
            0x08 => &self.tonguing_control_depth,
            0x09 => &self.scream_control_number,
            0x0A => &self.scream_control_depth,
            0x0B => &self.breath_control_number,
            0x0C => &self.breath_control_depth,
            0x0D => &self.growl_control_number,
            0x0E => &self.growl_control_depth,
            0x0F => &self.throat_formant_control_number,
            0x10 => &self.throat_formant_control_depth,
            0x11 => &self.harmonic_enhancer_control_number,
            0x12 => &self.harmonic_enhancer_control_depth,
            0x13 => &self.damping_control_number,
            0x14 => &self.damping_control_depth,
            0x15 => &self.absorption_control_number,
            0x16 => &self.absorption_control_depth,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for MultiPartVL {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x00 => &mut self.note_assign,
            0x02 => &mut self.note_filter,
            0x03 => &mut self.pressure_control_number,
            0x04 => &mut self.pressure_control_depth,
            0x05 => &mut self.embouchure_control_number,
            0x06 => &mut self.embouchure_control_depth,
            0x07 => &mut self.tonguing_control_number,
            0x08 => &mut self.tonguing_control_depth,
            0x09 => &mut self.scream_control_number,
            0x0A => &mut self.scream_control_depth,
            0x0B => &mut self.breath_control_number,
            0x0C => &mut self.breath_control_depth,
            0x0D => &mut self.growl_control_number,
            0x0E => &mut self.growl_control_depth,
            0x0F => &mut self.throat_formant_control_number,
            0x10 => &mut self.throat_formant_control_depth,
            0x11 => &mut self.harmonic_enhancer_control_number,
            0x12 => &mut self.harmonic_enhancer_control_depth,
            0x13 => &mut self.damping_control_number,
            0x14 => &mut self.damping_control_depth,
            0x15 => &mut self.absorption_control_number,
            0x16 => &mut self.absorption_control_depth,
            _ => panic!("MultiPartVL: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for MultiPartVL {
    fn reset(&mut self) {
        *self = MultiPartVL::new();
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00 | 0x02..=0x16) {
            return Err(err);
        }
        Ok(self[addr])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00 | 0x02..=0x16) {
            return Err(err);
        }
        self[addr] = value;
        Ok(vec![RAMCallbackEffects::NoEffect])
    }
}
