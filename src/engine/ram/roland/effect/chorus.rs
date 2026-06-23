/*
地址偏移	参数名称	初始默认值	说明
21	合唱输出电平	5A (90)	合唱效果自身输出音量
22	合唱速率	32 (50)	LFO 调制频率，中等速度
23	合唱深度	28 (40)	音高调制幅度
24	反馈量	1E (30)	信号反馈比例
25	基础延迟	28 (40)	合唱延迟线基础时长
38	合唱类型宏指令	02 (Chorus 3)	写入后自动覆盖所有合唱精细参数，GS 默认标准合唱
*/
use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chorus {
    pub output_level: u8,
    pub chorus_rate: u8,
    pub chorus_depth: u8,
    pub chorus_return: u8,
    pub base_delay: u8,
    pub chorus_type: u8,
}

impl Chorus {
    pub fn new() -> Self {
        Self {
            output_level: 0x5A,
            chorus_rate: 0x32,
            chorus_depth: 0x28,
            chorus_return: 0x1E,
            base_delay: 0x28,
            chorus_type: 0x02,
        }
    }
}

impl Index<usize> for Chorus {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            1 | 0x21 => &self.output_level,
            2 | 0x22 => &self.chorus_rate,
            3 | 0x23 => &self.chorus_depth,
            4 | 0x24 => &self.chorus_return,
            5 | 0x25 => &self.base_delay,
            0x38 => &self.chorus_type,

            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for Chorus {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            1 | 0x21 => &mut self.output_level,
            2 | 0x22 => &mut self.chorus_rate,
            3 | 0x23 => &mut self.chorus_depth,
            4 | 0x24 => &mut self.chorus_return,
            5 | 0x25 => &mut self.base_delay,
            0x38 => &mut self.chorus_type,

            _ => panic!("Chorus: index {} out of bounds", index),
        }
    }
}

impl Memory for Chorus {
    fn reset(&mut self) {
        self.output_level = 0x5A;
        self.chorus_rate = 0x32;
        self.chorus_depth = 0x28;
        self.chorus_return = 0x1E;
        self.base_delay = 0x28;
        self.chorus_type = 0x02;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 1..=5 | 0x21..=0x25 | 0x38) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 1..=5 | 0x21..=0x25 | 0x30) {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }
}
