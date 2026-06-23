/*
地址偏移	参数名称	初始默认值	说明
01	混响输出电平	64 (100)	混响效果自身输出音量
02	混响时间	40 (64)	残响衰减时长，中心值对应中等混响长度
03	预延迟	0A (10)	干声到混响启动的延迟
04	高频衰减	3C (60)	混响高频阻尼系数
30	混响类型宏指令	04 (Hall 2)	写入后自动覆盖所有混响精细参数，GS 默认大厅混响
*/
use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reverb {
    pub output_level: u8,
    pub decay_time: u8,
    pub pre_delay: u8,
    pub hi_freq_decay: u8,
    pub reverb_type: u8,
}

impl Reverb {
    pub fn new() -> Self {
        Self {
            output_level: 0x64,
            decay_time: 0x40,
            pre_delay: 0x0A,
            hi_freq_decay: 0x3C,
            reverb_type: 0x04,
        }
    }
}

impl Index<usize> for Reverb {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            1 => &self.output_level,
            2 => &self.decay_time,
            3 => &self.pre_delay,
            4 => &self.hi_freq_decay,
            0x30 => &self.reverb_type,

            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for Reverb {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            1 => &mut self.output_level,
            2 => &mut self.decay_time,
            3 => &mut self.pre_delay,
            4 => &mut self.hi_freq_decay,
            0x30 => &mut self.reverb_type,

            _ => panic!("Reverb: index {} out of bounds", index),
        }
    }
}

impl Memory for Reverb {
    fn reset(&mut self) {
        self.output_level = 0x64;
        self.decay_time = 0x40;
        self.pre_delay = 0x0A;
        self.hi_freq_decay = 0x3C;
        self.reverb_type = 0x04;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 1..=4 | 0x30) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 1..=4 | 0x30) {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }
}
