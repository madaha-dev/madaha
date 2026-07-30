pub mod ac;
pub mod aftertouch;
pub mod bend;
pub mod mw;
pub mod rcv_switches;

use crate::engine::consts::DRUM_CHANNEL_ID;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use crate::engine::ram::xg::multi_part::ac::AC;
use crate::engine::ram::xg::multi_part::aftertouch::AfterTouch;
use crate::engine::ram::xg::multi_part::bend::Bend;
use crate::engine::ram::xg::multi_part::rcv_switches::RcvSwitches;
use crate::engine::{errors::MidiError, ram::xg::multi_part::mw::MW};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiPart {
    /// Element reserve count (0-15, 2=normal, 0=drum)
    pub element_reserve: u8,
    /// Bank Select MSB (CC#0, 0x00=normal, 0x7F=drum)
    pub bank_select_msb: u8,
    /// Bank Select LSB (CC#32)
    pub bank_select_lsb: u8,
    /// Program number (0-127)
    pub program_number: u8,
    /// MIDI receive channel (0-15)
    pub rcv_channel: u8,
    // poly/mono
    /// Mono/Poly mode (0=mono, 1=poly)
    pub mode: u8,
    // single/multi/inst
    /// Key-on assign mode (0=single, 1=multi, 2=inst/drum)
    pub key_assign: u8,
    // normal/drum/drums1-4
    /// Part mode (0=normal, 1=drum, 2-5=drum kit variants)
    pub part_mode: u8,
    /// Note shift in semitones (centered at 0x40=0, range 32-96 = -32~+32)
    pub note_shift: u8,
    /// Detune MSB (14-bit resolution, combined with detune_lsb)
    pub detune_msb: u8,
    /// Detune LSB (lower 4 bits)
    pub detune_lsb: u8,
    /// Part volume (CC#7, 0-127)
    pub volume: u8,
    /// Velocity sensitivity depth (0x40=center)
    pub velocity_sense_depth: u8,
    /// Velocity sense offset (0x40=center)
    pub velocity_sense_offset: u8,
    /// Panpot (CC#10, 0=left, 64=center, 127=right)
    pub pan: u8,
    /// Note limit low (lowest playable note)
    pub note_limit_low: u8,
    /// Note limit high (highest playable note)
    pub note_limit_high: u8,
    /// Dry output level (0-127)
    pub dry_level: u8,
    /// Chorus send level (CC#93)
    pub chorus_send: u8,
    /// Reverb send level (CC#91)
    pub reverb_send: u8,
    /// Variation send level (CC#94)
    pub variation_send: u8,
    /// Vibrato rate
    pub vibrato_rate: u8,
    /// Vibrato depth
    pub vibrato_depth: u8,
    /// Vibrato delay
    pub vibrato_delay: u8,
    /// Filter cutoff frequency (CC#74/Brightness, 0x40=center)
    pub filter_cutoff_freq: u8,
    /// Filter resonance (CC#71/Harmonic Content, 0x40=center)
    pub filter_resonance: u8,
    /// EG attack time (CC#73)
    pub eg_attack_time: u8,
    /// EG decay time
    pub eg_decay_time: u8,
    /// EG release time (CC#72)
    pub eg_release_time: u8,
    /// Modulation wheel control parameters
    pub mw: MW,
    /// Pitch bend control parameters
    pub bend: Bend,
    /// Receive switch flags for various MIDI messages
    pub rcv_switches: RcvSwitches,
    /// Scale tuning for 12 notes (C-B, centered at 0x40=0 cents)
    pub scale_tuning: [u8; 12],
    /// Channel Aftertouch control parameters
    pub cat: AfterTouch,
    /// Polyphonic Aftertouch control parameters
    pub pat: AfterTouch,
    /// Assignable Controller 1/2 parameters
    pub ac: [AC; 2],
    /// Portamento switch (CC#65, 0=off, 1=on)
    pub portamento_switch: u8,
    /// Portamento time (CC#5)
    pub portamento_time: u8,
    /// Pitch EG initial level
    pub pitch_eg_init_level: u8,
    /// Pitch EG attack time
    pub pitch_eg_attack_time: u8,
    /// Pitch EG release level
    pub pitch_eg_release_level: u8,
    /// Pitch EG release time
    pub pitch_eg_release_time: u8,
    /// Velocity limit low (lowest playable velocity)
    pub velocity_limit_low: u8,
    /// Velocity limit high (highest playable velocity)
    pub velocity_limit_high: u8,

    // XG Spec 2.0
    /// Bend pitch control (low range, for negative bend)
    pub bend_pitch_low_control: u8,
    /// Filter EG depth
    pub filter_eg_depth: u8,
    /// EQ bass gain
    pub eq_bass: u8,
    /// EQ treble gain
    pub eq_treble: u8,
    /// EQ mid-bass gain (not used)
    pub eq_mid_bass: u8,
    /// EQ mid-treble gain (not used)
    pub eq_mid_treble: u8,
    /// EQ bass frequency
    pub eq_bass_freq: u8,
    /// EQ treble frequency
    pub eq_treble_freq: u8,
    /// EQ mid-bass frequency (not used)
    pub eq_mid_bass_freq: u8,
    /// EQ mid-treble frequency (not used)
    pub eq_mid_treble_freq: u8,
    /// EQ bass Q (not used)
    pub eq_bass_q: u8,
    /// EQ treble Q (not used)
    pub eq_treble_q: u8,
    /// EQ mid-bass Q (not used)
    pub eq_mid_bass_q: u8,
    /// EQ mid-treble Q (not used)
    pub eq_mid_treble_q: u8,
    /// EQ bass shape (not used)
    pub eq_bass_shape: u8,
    /// EQ treble shape (not used)
    pub eq_treble_shape: u8,
}

impl MultiPart {
    // depends on channel
    pub const fn new(part: usize) -> Self {
        Self {
            element_reserve: if part == DRUM_CHANNEL_ID { 0 } else { 2 },
            bank_select_msb: if part == DRUM_CHANNEL_ID { 0x7F } else { 0 },
            bank_select_lsb: 0,
            program_number: 0,
            rcv_channel: (part % 16) as u8,
            mode: 1,
            key_assign: if part == DRUM_CHANNEL_ID { 2 } else { 0 },
            part_mode: if part == DRUM_CHANNEL_ID { 2 } else { 0 },
            note_shift: 0x40,
            detune_msb: 0x80,
            detune_lsb: 0x00,
            volume: 0x64,
            velocity_sense_depth: 0x40,
            velocity_sense_offset: 0x40,
            pan: 0x40,
            note_limit_low: 0,
            note_limit_high: 0x7F,
            dry_level: 0x7F,
            chorus_send: 0,
            reverb_send: 0x28,
            variation_send: 0,
            vibrato_rate: 0x40,
            vibrato_depth: 0x40,
            vibrato_delay: 0x40,
            filter_cutoff_freq: 0x40,
            filter_resonance: 0x40,
            eg_attack_time: 0x40,
            eg_decay_time: 0x40,
            eg_release_time: 0x40,
            mw: MW::new(),
            bend: Bend::new(),
            rcv_switches: RcvSwitches::new(),
            scale_tuning: [0x40; 12],
            cat: AfterTouch::new(),
            pat: AfterTouch::new(),
            ac: [AC::new(); 2],
            portamento_switch: 0,
            portamento_time: 0,
            pitch_eg_init_level: 0x40,
            pitch_eg_attack_time: 0x40,
            pitch_eg_release_level: 0x40,
            pitch_eg_release_time: 0x40,
            velocity_limit_low: 1,
            velocity_limit_high: 0x7F,

            bend_pitch_low_control: 0x3E,
            filter_eg_depth: 0x40,
            eq_bass: 0x40,
            eq_treble: 0x40,
            eq_mid_bass: 0x40,
            eq_mid_treble: 0x40,
            eq_bass_freq: 0x0C,
            eq_treble_freq: 0x36,
            eq_mid_bass_freq: 0x22,
            eq_mid_treble_freq: 0x2E,
            eq_bass_q: 0x07,
            eq_treble_q: 0x07,
            eq_mid_bass_q: 0x07,
            eq_mid_treble_q: 0x07,
            eq_bass_shape: 0,
            eq_treble_shape: 0,
        }
    }

    pub fn get_detune(&self) -> u8 {
        (self.detune_msb & 0xF) << 4 | self.detune_lsb & 0xF
    }

    pub fn set_detune(&mut self, value: u8) {
        self.detune_lsb = value & 0xF;
        self.detune_msb = (value >> 4) & 0xF;
    }

    pub fn get_velocity(&self, vel: u8) -> u8 {
        if vel < self.velocity_limit_low || vel > self.velocity_limit_high {
            0
        } else {
            let r: i8 = (vel as i8 - 64) * self.velocity_sense_depth as i8 / 64;
            (r + self.velocity_sense_offset as i8).clamp(1, 127) as u8
        }
    }

    // in cents
    pub fn get_delta_pitch(&self, note_in_cents: f32) -> f32 {
        let scale_in_cents = self.scale_tuning[note_in_cents as usize % 1200] as f32;

        note_in_cents + self.get_detune() as f32 * 10.0 + scale_in_cents
    }
}

impl Index<usize> for MultiPart {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0x00 => &self.element_reserve,
            0x01 => &self.bank_select_msb,
            0x02 => &self.bank_select_lsb,
            0x03 => &self.program_number,
            0x04 => &self.rcv_channel,
            0x05 => &self.mode,
            0x06 => &self.key_assign,
            0x07 => &self.part_mode,
            0x08 => &self.note_shift,
            0x09 => &self.detune_msb,
            0x0A => &self.detune_lsb,
            0x0B => &self.volume,
            0x0C => &self.velocity_sense_depth,
            0x0D => &self.velocity_sense_offset,
            0x0E => &self.pan,
            0x0F => &self.note_limit_low,
            0x10 => &self.note_limit_high,
            0x11 => &self.dry_level,
            0x12 => &self.chorus_send,
            0x13 => &self.reverb_send,
            0x14 => &self.variation_send,
            0x15 => &self.vibrato_rate,
            0x16 => &self.vibrato_depth,
            0x17 => &self.vibrato_delay,
            0x18 => &self.filter_cutoff_freq,
            0x19 => &self.filter_resonance,
            0x1A => &self.eg_attack_time,
            0x1B => &self.eg_decay_time,
            0x1C => &self.eg_release_time,
            0x1D..=0x22 => &self.mw[index],
            0x23..=0x28 => &self.bend[index],
            0x30..=0x40 => &self.rcv_switches[index],
            0x41..=0x4C => &self.scale_tuning[index - 0x41],
            0x4D..=0x52 => &self.cat[index],
            0x53..=0x58 => &self.pat[index],
            0x59..=0x5F => &self.ac[0][index],
            0x60..=0x66 => &self.ac[1][index],
            0x67 => &self.portamento_switch,
            0x68 => &self.portamento_time,
            0x69 => &self.pitch_eg_init_level,
            0x6A => &self.pitch_eg_attack_time,
            0x6B => &self.pitch_eg_release_level,
            0x6C => &self.pitch_eg_release_time,
            0x6D => &self.velocity_limit_low,
            0x6E => &self.velocity_limit_high,

            0x70 => &self.bend_pitch_low_control,
            0x71 => &self.filter_eg_depth,
            0x72 => &self.eq_bass,
            0x73 => &self.eq_treble,
            0x74 => &self.eq_mid_bass,
            0x75 => &self.eq_mid_treble,
            0x76 => &self.eq_bass_freq,
            0x77 => &self.eq_treble_freq,
            0x78 => &self.eq_mid_bass_freq,
            0x79 => &self.eq_mid_treble_freq,
            0x7A => &self.eq_bass_q,
            0x7B => &self.eq_treble_q,
            0x7C => &self.eq_mid_bass_q,
            0x7D => &self.eq_mid_treble_q,
            0x7E => &self.eq_bass_shape,
            0x7F => &self.eq_treble_shape,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for MultiPart {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x00 => &mut self.element_reserve,
            0x01 => &mut self.bank_select_msb,
            0x02 => &mut self.bank_select_lsb,
            0x03 => &mut self.program_number,
            0x04 => &mut self.rcv_channel,
            0x05 => &mut self.mode,
            0x06 => &mut self.key_assign,
            0x07 => &mut self.part_mode,
            0x08 => &mut self.note_shift,
            0x09 => &mut self.detune_msb,
            0x0A => &mut self.detune_lsb,
            0x0B => &mut self.volume,
            0x0C => &mut self.velocity_sense_depth,
            0x0D => &mut self.velocity_sense_offset,
            0x0E => &mut self.pan,
            0x0F => &mut self.note_limit_low,
            0x10 => &mut self.note_limit_high,
            0x11 => &mut self.dry_level,
            0x12 => &mut self.chorus_send,
            0x13 => &mut self.reverb_send,
            0x14 => &mut self.variation_send,
            0x15 => &mut self.vibrato_rate,
            0x16 => &mut self.vibrato_depth,
            0x17 => &mut self.vibrato_delay,
            0x18 => &mut self.filter_cutoff_freq,
            0x19 => &mut self.filter_resonance,
            0x1A => &mut self.eg_attack_time,
            0x1B => &mut self.eg_decay_time,
            0x1C => &mut self.eg_release_time,
            0x1D..=0x22 => &mut self.mw[index],
            0x23..=0x28 => &mut self.bend[index],
            0x30..=0x40 => &mut self.rcv_switches[index],
            0x41..=0x4C => &mut self.scale_tuning[index - 0x41],
            0x4D..=0x52 => &mut self.cat[index],
            0x53..=0x58 => &mut self.pat[index],
            0x59..=0x5F => &mut self.ac[0][index],
            0x60..=0x66 => &mut self.ac[1][index],
            0x67 => &mut self.portamento_switch,
            0x68 => &mut self.portamento_time,
            0x69 => &mut self.pitch_eg_init_level,
            0x6A => &mut self.pitch_eg_attack_time,
            0x6B => &mut self.pitch_eg_release_level,
            0x6C => &mut self.pitch_eg_release_time,
            0x6D => &mut self.velocity_limit_low,
            0x6E => &mut self.velocity_limit_high,
            0x70 => &mut self.bend_pitch_low_control,
            0x71 => &mut self.filter_eg_depth,
            0x72 => &mut self.eq_bass,
            0x73 => &mut self.eq_treble,
            0x74 => &mut self.eq_mid_bass,
            0x75 => &mut self.eq_mid_treble,
            0x76 => &mut self.eq_bass_freq,
            0x77 => &mut self.eq_treble_freq,
            0x78 => &mut self.eq_mid_bass_freq,
            0x79 => &mut self.eq_mid_treble_freq,
            0x7A => &mut self.eq_bass_q,
            0x7B => &mut self.eq_treble_q,
            0x7C => &mut self.eq_mid_bass_q,
            0x7D => &mut self.eq_mid_treble_q,
            0x7E => &mut self.eq_bass_shape,
            0x7F => &mut self.eq_treble_shape,
            _ => panic!("MultiPart: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for MultiPart {
    fn reset(&mut self) {
        let part = self.rcv_channel as usize;
        *self = MultiPart::new(part);
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00..=0x28 | 0x30..=0x6E | 0x70..=0x7F) {
            return Err(err);
        }
        Ok(self[addr])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2] as usize;
        if !matches!(addr, 0x00..=0x28 | 0x30..=0x6E | 0x70..=0x7F) {
            return Err(err);
        }
        Ok(self[addr] = value)
    }
}
