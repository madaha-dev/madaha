use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use crate::engine::{errors::MidiError, };
use crate::voice_manager::{DrumSetupEntry};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrumSetup {
    pub pitch_coarse: u8,
    pub pitch_fine: u8,
    pub level: u8,
    pub alternate_group: u8,
    pub pan: u8,
    pub reverb_send: u8,
    pub chorus_send: u8,
    pub variation_send: u8,
    pub key_assign: u8,
    pub rcv_note_off: u8,
    pub rcv_note_on: u8,
    pub filter_cutoff_freq: u8,
    pub filter_resonance: u8,
    pub eg_attack_rate: u8,
    pub eg_decay1_rate: u8,
    pub eg_decay2_rate: u8,

    // XG Spec 2.0
    // lo addr 0x20
    pub eq_bass: u8,
    pub eq_treble: u8,
    pub eq_mid_bass: u8,   // not used.
    pub eq_mid_treble: u8, // not used.
    pub eq_bass_freq: u8,
    pub eq_treble_freq: u8,
    pub eq_mid_bass_freq: u8,   // not used.
    pub eq_mid_treble_freq: u8, // not used.
    pub eq_bass_q: u8,          // not used.
    pub eq_treble_q: u8,        // not used.
    pub eq_mid_bass_q: u8,      // not used.
    pub eq_mid_treble_q: u8,    // not used.
    pub eq_bass_shape: u8,      // not used.
    pub eq_treble_shape: u8,    // not used.

    // lo addr 0x40
    pub output_select: u8,

    // lo addr 0x50
    pub hpf_cutoff_freq: u8,
    pub hpf_resonance: u8,

    // lo addr 0x60
    pub velocity_pitch_sense: u8,
    pub velocity_lpf_cutoff_sense: u8,

    // lo addr 0x70
    pub source_drum_kit_bank_msb: u8,
    pub source_drum_kit_bank_lsb: u8,
    pub source_drum_kit_program: u8,
    pub source_drum_kit_note: u8,

    pub _init_data: Option<&'static Box<[u8]>>,
}

impl DrumSetup {
    pub const fn new(data: &'static Box<[u8]>) -> Self {
        let mut _data = DEFAULT_DRUM_SETUP;
        _data.level = data[2];
        _data.alternate_group = data[3];
        _data.pan = data[4];
        _data.reverb_send = data[5];
        _data.chorus_send = data[6];
        _data.rcv_note_off = data[9];
        _data.velocity_pitch_sense = data[22];
        _data.velocity_lpf_cutoff_sense = data[23];
        _data._init_data = Some(data);
        _data
    }
}

impl Index<usize> for DrumSetup {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            // lo addr 0x00
            0x00 => &self.pitch_coarse,
            0x01 => &self.pitch_fine,
            0x02 => &self.level,
            0x03 => &self.alternate_group,
            0x04 => &self.pan,
            0x05 => &self.reverb_send,
            0x06 => &self.chorus_send,
            0x07 => &self.variation_send,
            0x08 => &self.key_assign,
            0x09 => &self.rcv_note_off,
            0x0A => &self.rcv_note_on,
            0x0B => &self.filter_cutoff_freq,
            0x0C => &self.filter_resonance,
            0x0D => &self.eg_attack_rate,
            0x0E => &self.eg_decay1_rate,
            0x0F => &self.eg_decay2_rate,
            // lo addr 0x20: EQ
            0x20 => &self.eq_bass,
            0x21 => &self.eq_treble,
            0x22 => &self.eq_mid_bass,
            0x23 => &self.eq_mid_treble,
            0x24 => &self.eq_bass_freq,
            0x25 => &self.eq_treble_freq,
            0x26 => &self.eq_mid_bass_freq,
            0x27 => &self.eq_mid_treble_freq,
            0x28 => &self.eq_bass_q,
            0x29 => &self.eq_treble_q,
            0x2A => &self.eq_mid_bass_q,
            0x2B => &self.eq_mid_treble_q,
            0x2C => &self.eq_bass_shape,
            0x2D => &self.eq_treble_shape,
            // lo addr 0x40
            0x40 => &self.output_select,
            // lo addr 0x50
            0x50 => &self.hpf_cutoff_freq,
            0x51 => &self.hpf_resonance,
            // lo addr 0x60
            0x60 => &self.velocity_pitch_sense,
            0x61 => &self.velocity_lpf_cutoff_sense,
            // lo addr 0x70
            0x70 => &self.source_drum_kit_bank_msb,
            0x71 => &self.source_drum_kit_bank_lsb,
            0x72 => &self.source_drum_kit_program,
            0x73 => &self.source_drum_kit_note,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for DrumSetup {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x00 => &mut self.pitch_coarse,
            0x01 => &mut self.pitch_fine,
            0x02 => &mut self.level,
            0x03 => &mut self.alternate_group,
            0x04 => &mut self.pan,
            0x05 => &mut self.reverb_send,
            0x06 => &mut self.chorus_send,
            0x07 => &mut self.variation_send,
            0x08 => &mut self.key_assign,
            0x09 => &mut self.rcv_note_off,
            0x0A => &mut self.rcv_note_on,
            0x0B => &mut self.filter_cutoff_freq,
            0x0C => &mut self.filter_resonance,
            0x0D => &mut self.eg_attack_rate,
            0x0E => &mut self.eg_decay1_rate,
            0x0F => &mut self.eg_decay2_rate,
            0x20 => &mut self.eq_bass,
            0x21 => &mut self.eq_treble,
            0x22 => &mut self.eq_mid_bass,
            0x23 => &mut self.eq_mid_treble,
            0x24 => &mut self.eq_bass_freq,
            0x25 => &mut self.eq_treble_freq,
            0x26 => &mut self.eq_mid_bass_freq,
            0x27 => &mut self.eq_mid_treble_freq,
            0x28 => &mut self.eq_bass_q,
            0x29 => &mut self.eq_treble_q,
            0x2A => &mut self.eq_mid_bass_q,
            0x2B => &mut self.eq_mid_treble_q,
            0x2C => &mut self.eq_bass_shape,
            0x2D => &mut self.eq_treble_shape,
            0x40 => &mut self.output_select,
            0x50 => &mut self.hpf_cutoff_freq,
            0x51 => &mut self.hpf_resonance,
            0x60 => &mut self.velocity_pitch_sense,
            0x61 => &mut self.velocity_lpf_cutoff_sense,
            0x70 => &mut self.source_drum_kit_bank_msb,
            0x71 => &mut self.source_drum_kit_bank_lsb,
            0x72 => &mut self.source_drum_kit_program,
            0x73 => &mut self.source_drum_kit_note,
            _ => panic!("DrumSetup: index {:#X} out of bounds", index),
        }
    }
}

impl From<DrumSetupEntry> for DrumSetup {
    fn from(value: DrumSetupEntry) -> Self {
        let mut _data = DEFAULT_DRUM_SETUP;
        _data.pitch_coarse = value.pitch_coarse;
        _data.pitch_fine = value.pitch_fine;
        _data.level = value.level;
        _data.alternate_group = value.alter_group;
        _data.pan = value.pan;
        _data.reverb_send = value.reverb_send;
        _data.chorus_send = value.chorus_send;
        _data.variation_send = value.variation_send;
        _data.key_assign = value.key_assign;
        _data.rcv_note_off = value.rcv_note_off;
        _data.rcv_note_on = value.rcv_note_on;
        _data.filter_cutoff_freq = value.filter_cutoff_freq;
        _data.filter_resonance = value.filter_resonance;
        _data.eg_attack_rate = value.eg_attack;
        _data.eg_decay1_rate = value.eg_decay1;
        _data.eg_decay2_rate = value.eg_decay2;
        _data.eq_bass = value.eq_bass;
        _data.eq_treble = value.eq_treble;
        _data.eq_bass_freq = value.eq_bass_freq;
        _data.eq_treble_freq = value.eq_treble_freq;
        _data.output_select = value.output_select;
        _data.hpf_cutoff_freq = value.hpf_cutoff_freq;
        _data.velocity_pitch_sense = value.vel_pitch_sense;
        _data.velocity_lpf_cutoff_sense = value.vel_lpf_cutoff_sense;
        _data
    }
}

impl Memory for DrumSetup {
    fn reset(&mut self) {
        *self = DrumSetup::new(self._init_data.unwrap());
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00..=0x0F | 0x20..=0x2D | 0x40 | 0x50..=0x51 | 0x60..=0x61 | 0x70..=0x73)
        {
            return Err(err);
        }
        Ok(self[addr])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00..=0x0F | 0x20..=0x2D | 0x40 | 0x50..=0x51 | 0x60..=0x61 | 0x70..=0x73)
        {
            return Err(err);
        }
        Ok(self[addr] = value)
    }
}

const DEFAULT_DRUM_SETUP: DrumSetup = DrumSetup {
    pitch_coarse: 0x40,
    pitch_fine: 0x40,
    level: 0x7F,
    alternate_group: 0x0,
    pan: 0x40,
    reverb_send: 0x7F,
    chorus_send: 0x7F,
    variation_send: 0x7F,
    key_assign: 0x0,
    rcv_note_off: 0x0,
    rcv_note_on: 0x01,
    filter_cutoff_freq: 0x40,
    filter_resonance: 0x40,
    eg_attack_rate: 0x40,
    eg_decay1_rate: 0x40,
    eg_decay2_rate: 0x40,

    eq_bass: 0x40,
    eq_treble: 0x40,
    eq_mid_bass: 0x40,
    eq_mid_treble: 0x40,
    eq_bass_freq: 0x0C,
    eq_treble_freq: 0x36,
    eq_mid_bass_freq: 0x22,
    eq_mid_treble_freq: 0x2E,
    eq_bass_q: 7,
    eq_treble_q: 7,
    eq_mid_bass_q: 7,
    eq_mid_treble_q: 7,
    eq_bass_shape: 0,
    eq_treble_shape: 0,

    output_select: 0,

    hpf_cutoff_freq: 0x40,
    hpf_resonance: 0x40,

    velocity_pitch_sense: 0x40,
    velocity_lpf_cutoff_sense: 0x40,

    source_drum_kit_bank_msb: 0x7F,
    source_drum_kit_bank_lsb: 0x00,
    source_drum_kit_program: 0xFF,
    source_drum_kit_note: 0xFF,

    _init_data: None,
};
