use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiPartExt {
    // From XG Spec 2.0
    // hi addr = 0x0A
    // mid addr means channel(0x00-0x0F)

    // lo addr = 0x10
    pub output_select: u8,

    // lo addr start 0x20
    pub hpf_cutoff_freq: u8,
    pub hpf_resonance: u8,
    pub mw_hpf_control_depth: u8,
    pub bend_hpf_control_depth: u8,
    pub cat_hpf_control_depth: u8,
    pub pat_hpf_control_depth: u8,
    pub ac1_hpf_control_depth: u8,
    pub ac2_hpf_control_depth: u8,

    // lo addr start 0x30
    pub cbc1_control_number: u8,
    pub cbc1_pitch_control: u8,
    pub cbc1_lpf_control: u8,
    pub cbc1_amplitude_control: u8,
    pub cbc1_lfo_pmod_control_depth: u8,
    pub cbc1_lfo_fmod_control_depth: u8,
    pub cbc1_lfo_amod_control_depth: u8,

    // lo addr start 0x38
    pub cbc2_control_number: u8,
    pub cbc2_pitch_control: u8,
    pub cbc2_lpf_control: u8,
    pub cbc2_amplitude_control: u8,
    pub cbc2_lfo_pmod_control_depth: u8,
    pub cbc2_lfo_fmod_control_depth: u8,
    pub cbc2_lfo_amod_control_depth: u8,

    // lo addr start 0x40
    pub mw_offset_level_control: u8,
    pub bend_offset_level_control: u8,
    pub cat_offset_level_control: u8,
    pub pat_offset_level_control: u8,
    pub ac1_offset_level_control: u8,
    pub ac2_offset_level_control: u8,
}

impl MultiPartExt {
    pub const fn new() -> Self {
        Self {
            output_select: 0,
            hpf_cutoff_freq: 0x40,
            hpf_resonance: 0x40,
            mw_hpf_control_depth: 0x40,
            bend_hpf_control_depth: 0x40,
            cat_hpf_control_depth: 0x40,
            pat_hpf_control_depth: 0x40,
            ac1_hpf_control_depth: 0x40,
            ac2_hpf_control_depth: 0x40,
            cbc1_control_number: 0x12,
            cbc1_pitch_control: 0x40,
            cbc1_lpf_control: 0x40,
            cbc1_amplitude_control: 0x40,
            cbc1_lfo_pmod_control_depth: 0,
            cbc1_lfo_fmod_control_depth: 0,
            cbc1_lfo_amod_control_depth: 0,
            cbc2_control_number: 0x13,
            cbc2_pitch_control: 0x40,
            cbc2_lpf_control: 0x40,
            cbc2_amplitude_control: 0x40,
            cbc2_lfo_pmod_control_depth: 0,
            cbc2_lfo_fmod_control_depth: 0,
            cbc2_lfo_amod_control_depth: 0,
            mw_offset_level_control: 0x40,
            bend_offset_level_control: 0x40,
            cat_offset_level_control: 0x40,
            pat_offset_level_control: 0x40,
            ac1_offset_level_control: 0x40,
            ac2_offset_level_control: 0x40,
        }
    }
}

impl Index<usize> for MultiPartExt {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            // output select
            0x10 => &self.output_select,
            // HPF
            0x20 => &self.hpf_cutoff_freq,
            0x21 => &self.hpf_resonance,
            0x22 => &self.mw_hpf_control_depth,
            0x23 => &self.bend_hpf_control_depth,
            0x24 => &self.cat_hpf_control_depth,
            0x25 => &self.pat_hpf_control_depth,
            0x26 => &self.ac1_hpf_control_depth,
            0x27 => &self.ac2_hpf_control_depth,
            // CBC1
            0x30 => &self.cbc1_control_number,
            0x31 => &self.cbc1_pitch_control,
            0x32 => &self.cbc1_lpf_control,
            0x33 => &self.cbc1_amplitude_control,
            0x34 => &self.cbc1_lfo_pmod_control_depth,
            0x35 => &self.cbc1_lfo_fmod_control_depth,
            0x36 => &self.cbc1_lfo_amod_control_depth,
            // CBC2
            0x38 => &self.cbc2_control_number,
            0x39 => &self.cbc2_pitch_control,
            0x3A => &self.cbc2_lpf_control,
            0x3B => &self.cbc2_amplitude_control,
            0x3C => &self.cbc2_lfo_pmod_control_depth,
            0x3D => &self.cbc2_lfo_fmod_control_depth,
            0x3E => &self.cbc2_lfo_amod_control_depth,
            // Offset level
            0x40 => &self.mw_offset_level_control,
            0x41 => &self.bend_offset_level_control,
            0x42 => &self.cat_offset_level_control,
            0x43 => &self.pat_offset_level_control,
            0x44 => &self.ac1_offset_level_control,
            0x45 => &self.ac2_offset_level_control,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for MultiPartExt {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x10 => &mut self.output_select,
            0x20 => &mut self.hpf_cutoff_freq,
            0x21 => &mut self.hpf_resonance,
            0x22 => &mut self.mw_hpf_control_depth,
            0x23 => &mut self.bend_hpf_control_depth,
            0x24 => &mut self.cat_hpf_control_depth,
            0x25 => &mut self.pat_hpf_control_depth,
            0x26 => &mut self.ac1_hpf_control_depth,
            0x27 => &mut self.ac2_hpf_control_depth,
            0x30 => &mut self.cbc1_control_number,
            0x31 => &mut self.cbc1_pitch_control,
            0x32 => &mut self.cbc1_lpf_control,
            0x33 => &mut self.cbc1_amplitude_control,
            0x34 => &mut self.cbc1_lfo_pmod_control_depth,
            0x35 => &mut self.cbc1_lfo_fmod_control_depth,
            0x36 => &mut self.cbc1_lfo_amod_control_depth,
            0x38 => &mut self.cbc2_control_number,
            0x39 => &mut self.cbc2_pitch_control,
            0x3A => &mut self.cbc2_lpf_control,
            0x3B => &mut self.cbc2_amplitude_control,
            0x3C => &mut self.cbc2_lfo_pmod_control_depth,
            0x3D => &mut self.cbc2_lfo_fmod_control_depth,
            0x3E => &mut self.cbc2_lfo_amod_control_depth,
            0x40 => &mut self.mw_offset_level_control,
            0x41 => &mut self.bend_offset_level_control,
            0x42 => &mut self.cat_offset_level_control,
            0x43 => &mut self.pat_offset_level_control,
            0x44 => &mut self.ac1_offset_level_control,
            0x45 => &mut self.ac2_offset_level_control,
            _ => panic!("MultiPartExt: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for MultiPartExt {
    fn reset(&mut self) {
        *self = MultiPartExt::new();
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x10 | 0x20..=0x27 | 0x30..=0x36 | 0x38..=0x3E | 0x40..=0x45) {
            return Err(err);
        }
        Ok(self[addr])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x10 | 0x20..=0x27 | 0x30..=0x36 | 0x38..=0x3E | 0x40..=0x45) {
            return Err(err);
        }
        Ok(self[addr] = value)
    }
}
