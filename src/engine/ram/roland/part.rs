use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Part {
    // 0x02
    pub rcv_switch: u8,
    // 0x03
    pub rcv_program_change: u8,
    // 0x14
    pub assign_mode: u8,
    // 0x15
    pub rhythm_map: u8,
    // 0x16
    pub pitch_coarse: u8,
    // 0x17-0x18: 14-bit fine tuning
    pub pitch_fine_msb: u8,
    pub pitch_fine_lsb: u8,
    // 0x19
    pub volume: u8,
    // 0x1A
    pub pan: u8,
    // 0x1B
    pub expression: u8,
    // 0x1C
    pub reverb_send: u8,
    // 0x1D
    pub chorus_send: u8,
    // 0x1E
    pub variation_send: u8,
    // 0x20
    pub portamento_time: u8,
    // 0x21
    pub portamento_switch: u8,
    // 0x24
    pub pitch_bend_range: u8,
    // 0x30
    pub filter_cutoff: u8,
    // 0x31
    pub resonance: u8,
    // 0x32
    pub eg_attack: u8,
    // 0x33
    pub eg_decay: u8,
    // 0x34
    pub eg_release: u8,
    // 0x40-0x4B: scale tuning for 12 semitones
    pub scale_tuning: [u8; 12],
}

impl Part {
    pub const fn new(part: u8) -> Self {
        Self {
            rcv_switch: 0x00,
            rcv_program_change: 0x00,
            assign_mode: if part == 0 { 0 } else { 1 },
            rhythm_map: 0x00,
            pitch_coarse: 0x40,
            pitch_fine_msb: 0x40,
            pitch_fine_lsb: 0x00,
            volume: 0x64,
            pan: 0x40,
            expression: 0x7F,
            reverb_send: 0x40,
            chorus_send: 0x40,
            variation_send: 0x00,
            portamento_time: 0x00,
            portamento_switch: 0x00,
            pitch_bend_range: 0x02,
            filter_cutoff: 0x40,
            resonance: 0x40,
            eg_attack: 0x40,
            eg_decay: 0x40,
            eg_release: 0x40,
            scale_tuning: [0x40; 12],
        }
    }

    pub fn get_pitch_fine(&self) -> u16 {
        (self.pitch_fine_msb as u16) << 7 | self.pitch_fine_lsb as u16
    }

    pub fn set_pitch_fine(&mut self, value: u16) {
        let lsb = value & 0x7F;
        let msb = (value >> 7) & 0x7F;
        self.pitch_fine_msb = msb as u8;
        self.pitch_fine_lsb = lsb as u8;
    }
}

impl Index<usize> for Part {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0x02 => &self.rcv_switch,
            0x03 => &self.rcv_program_change,
            0x14 => &self.assign_mode,
            0x15 => &self.rhythm_map,
            0x16 => &self.pitch_coarse,
            0x17 => &self.pitch_fine_msb,
            0x18 => &self.pitch_fine_lsb,
            0x19 => &self.volume,
            0x1A => &self.pan,
            0x1B => &self.expression,
            0x1C => &self.reverb_send,
            0x1D => &self.chorus_send,
            0x1E => &self.variation_send,
            0x20 => &self.portamento_time,
            0x21 => &self.portamento_switch,
            0x24 => &self.pitch_bend_range,
            0x30 => &self.filter_cutoff,
            0x31 => &self.resonance,
            0x32 => &self.eg_attack,
            0x33 => &self.eg_decay,
            0x34 => &self.eg_release,
            0x40..=0x4B => &self.scale_tuning[index - 0x40],
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for Part {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x02 => &mut self.rcv_switch,
            0x03 => &mut self.rcv_program_change,
            0x14 => &mut self.assign_mode,
            0x15 => &mut self.rhythm_map,
            0x16 => &mut self.pitch_coarse,
            0x17 => &mut self.pitch_fine_msb,
            0x18 => &mut self.pitch_fine_lsb,
            0x19 => &mut self.volume,
            0x1A => &mut self.pan,
            0x1B => &mut self.expression,
            0x1C => &mut self.reverb_send,
            0x1D => &mut self.chorus_send,
            0x1E => &mut self.variation_send,
            0x20 => &mut self.portamento_time,
            0x21 => &mut self.portamento_switch,
            0x24 => &mut self.pitch_bend_range,
            0x30 => &mut self.filter_cutoff,
            0x31 => &mut self.resonance,
            0x32 => &mut self.eg_attack,
            0x33 => &mut self.eg_decay,
            0x34 => &mut self.eg_release,
            0x40..=0x4B => &mut self.scale_tuning[index - 0x40],
            _ => panic!("Part: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for Part {
    fn reset(&mut self) {
        self.rcv_switch = 0x00;
        self.rcv_program_change = 0x00;
        self.assign_mode = 0x01;
        self.rhythm_map = 0x00;
        self.pitch_coarse = 0x40;
        self.pitch_fine_msb = 0x40;
        self.pitch_fine_lsb = 0x00;
        self.volume = 0x64;
        self.pan = 0x40;
        self.expression = 0x7F;
        self.reverb_send = 0x40;
        self.chorus_send = 0x40;
        self.variation_send = 0x00;
        self.portamento_time = 0x00;
        self.portamento_switch = 0x00;
        self.pitch_bend_range = 0x02;
        self.filter_cutoff = 0x40;
        self.resonance = 0x40;
        self.eg_attack = 0x40;
        self.eg_decay = 0x40;
        self.eg_release = 0x40;
        self.scale_tuning = [0x40; 12];
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x02..=0x03 | 0x14..=0x1E | 0x20..=0x21 | 0x24 | 0x30..=0x34 | 0x40..=0x4B)
        {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x02..=0x03 | 0x14..=0x1E | 0x20..=0x21 | 0x24 | 0x30..=0x34 | 0x40..=0x4B)
        {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }
}
