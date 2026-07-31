use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use std::ops::{Index, IndexMut};

/// XG Spec 2.0 extended multi-part parameters (hi addr 0x0A).
///
/// Contains per-part extended parameters including HPF settings, control bank
/// assignments (CBC1/CBC2), and offset level control depths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiPartExt {
    // hi addr = 0x0A
    // mid addr means channel(0x00-0x0F)
    /// Output destination selection for the part (lo addr 0x10)
    pub output_select: u8,

    // lo addr start 0x20
    /// HPF cutoff frequency
    pub hpf_cutoff_freq: u8,
    /// HPF resonance
    pub hpf_resonance: u8,
    /// Modulation wheel → HPF cutoff control depth
    pub mw_hpf_control_depth: u8,
    /// Pitch bend → HPF cutoff control depth
    pub bend_hpf_control_depth: u8,
    /// Channel aftertouch → HPF cutoff control depth
    pub cat_hpf_control_depth: u8,
    /// Polyphonic aftertouch → HPF cutoff control depth
    pub pat_hpf_control_depth: u8,
    /// Assignable controller 1 → HPF cutoff control depth
    pub ac1_hpf_control_depth: u8,
    /// Assignable controller 2 → HPF cutoff control depth
    pub ac2_hpf_control_depth: u8,

    // lo addr start 0x30
    /// Control bank 1 — controller number
    pub cbc1_control_number: u8,
    /// Control bank 1 — pitch control depth
    pub cbc1_pitch_control: u8,
    /// Control bank 1 — LPF cutoff control depth
    pub cbc1_lpf_control: u8,
    /// Control bank 1 — amplitude control depth
    pub cbc1_amplitude_control: u8,
    /// Control bank 1 — LFO pitch modulation control depth
    pub cbc1_lfo_pmod_control_depth: u8,
    /// Control bank 1 — LFO filter modulation control depth
    pub cbc1_lfo_fmod_control_depth: u8,
    /// Control bank 1 — LFO amplitude modulation control depth
    pub cbc1_lfo_amod_control_depth: u8,

    // lo addr start 0x38
    /// Control bank 2 — controller number
    pub cbc2_control_number: u8,
    /// Control bank 2 — pitch control depth
    pub cbc2_pitch_control: u8,
    /// Control bank 2 — LPF cutoff control depth
    pub cbc2_lpf_control: u8,
    /// Control bank 2 — amplitude control depth
    pub cbc2_amplitude_control: u8,
    /// Control bank 2 — LFO pitch modulation control depth
    pub cbc2_lfo_pmod_control_depth: u8,
    /// Control bank 2 — LFO filter modulation control depth
    pub cbc2_lfo_fmod_control_depth: u8,
    /// Control bank 2 — LFO amplitude modulation control depth
    pub cbc2_lfo_amod_control_depth: u8,

    // lo addr start 0x40
    /// Modulation wheel → offset level control depth
    pub mw_offset_level_control: u8,
    /// Pitch bend → offset level control depth
    pub bend_offset_level_control: u8,
    /// Channel aftertouch → offset level control depth
    pub cat_offset_level_control: u8,
    /// Polyphonic aftertouch → offset level control depth
    pub pat_offset_level_control: u8,
    /// Assignable controller 1 → offset level control depth
    pub ac1_offset_level_control: u8,
    /// Assignable controller 2 → offset level control depth
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

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x10 | 0x20..=0x27 | 0x30..=0x36 | 0x38..=0x3E | 0x40..=0x45) {
            return Err(err);
        }
        self[addr] = value;
        Ok(vec![RAMCallbackEffects::NoEffect])
    }
}
