use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RcvSwitches {
    pub rcv_pitch_bend: u8,
    pub rcv_chan_aftertouch: u8,
    pub rcv_program_change: u8,
    pub rcv_control_change: u8,
    pub rcv_poly_aftertouch: u8,
    pub rcv_note_message: u8,
    pub rcv_rpn: u8,
    pub rcv_nrpn: u8,
    pub rcv_moduration: u8,
    pub rcv_volume: u8,
    pub rcv_pan: u8,
    pub rcv_expression: u8,
    pub rcv_hold1: u8,
    pub rcv_portamento: u8,
    pub rcv_sostenuto: u8,
    pub rcv_soft_pedal: u8,
    pub rcv_bank_select: u8,
}

impl RcvSwitches {
    pub const fn new() -> Self {
        Self {
            rcv_pitch_bend: 1,
            rcv_chan_aftertouch: 1,
            rcv_program_change: 1,
            rcv_control_change: 1,
            rcv_poly_aftertouch: 1,
            rcv_note_message: 1,
            rcv_rpn: 1,
            rcv_nrpn: 1,
            rcv_moduration: 1,
            rcv_volume: 1,
            rcv_pan: 1,
            rcv_expression: 1,
            rcv_hold1: 1,
            rcv_portamento: 1,
            rcv_sostenuto: 1,
            rcv_soft_pedal: 1,
            rcv_bank_select: 1,
        }
    }
}

impl Index<usize> for RcvSwitches {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 0x30 => &self.rcv_pitch_bend,
            1 | 0x31 => &self.rcv_chan_aftertouch,
            2 | 0x32 => &self.rcv_program_change,
            3 | 0x33 => &self.rcv_control_change,
            4 | 0x34 => &self.rcv_poly_aftertouch,
            5 | 0x35 => &self.rcv_note_message,
            6 | 0x36 => &self.rcv_rpn,
            7 | 0x37 => &self.rcv_nrpn,
            8 | 0x38 => &self.rcv_moduration,
            9 | 0x39 => &self.rcv_volume,
            10 | 0x3A => &self.rcv_pan,
            11 | 0x3B => &self.rcv_expression,
            12 | 0x3C => &self.rcv_hold1,
            13 | 0x3D => &self.rcv_portamento,
            14 | 0x3E => &self.rcv_sostenuto,
            15 | 0x3F => &self.rcv_soft_pedal,
            16 | 0x40 => &self.rcv_bank_select,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for RcvSwitches {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 0x30 => &mut self.rcv_pitch_bend,
            1 | 0x31 => &mut self.rcv_chan_aftertouch,
            2 | 0x32 => &mut self.rcv_program_change,
            3 | 0x33 => &mut self.rcv_control_change,
            4 | 0x34 => &mut self.rcv_poly_aftertouch,
            5 | 0x35 => &mut self.rcv_note_message,
            6 | 0x36 => &mut self.rcv_rpn,
            7 | 0x37 => &mut self.rcv_nrpn,
            8 | 0x38 => &mut self.rcv_moduration,
            9 | 0x39 => &mut self.rcv_volume,
            10 | 0x3A => &mut self.rcv_pan,
            11 | 0x3B => &mut self.rcv_expression,
            12 | 0x3C => &mut self.rcv_hold1,
            13 | 0x3D => &mut self.rcv_portamento,
            14 | 0x3E => &mut self.rcv_sostenuto,
            15 | 0x3F => &mut self.rcv_soft_pedal,
            16 | 0x40 => &mut self.rcv_bank_select,
            _ => panic!("RcvSwitches: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for RcvSwitches {
    fn reset(&mut self) {
        self.rcv_pitch_bend = 1;
        self.rcv_chan_aftertouch = 1;
        self.rcv_program_change = 1;
        self.rcv_control_change = 1;
        self.rcv_poly_aftertouch = 1;
        self.rcv_note_message = 1;
        self.rcv_rpn = 1;
        self.rcv_nrpn = 1;
        self.rcv_moduration = 1;
        self.rcv_volume = 1;
        self.rcv_pan = 1;
        self.rcv_expression = 1;
        self.rcv_hold1 = 1;
        self.rcv_portamento = 1;
        self.rcv_sostenuto = 1;
        self.rcv_soft_pedal = 1;
        self.rcv_bank_select = 1;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x30..=0x40) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x30..=0x40) {
            return Err(err);
        }
        self[addr as usize] = value & 1;

        Ok(vec![RAMCallbackEffects::NoEffect])
    }
}
