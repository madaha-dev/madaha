/*
地址偏移	参数名称	初始默认值	说明
04	主音量 Master Volume	7F (127)	全局总输出音量，GS 复位后为最大值
05~06	主调音 Master Tune	40 00	14-bit 精度，中心值对应 0 音分，标准音高 440Hz
07	混响总输出电平	7F (127)	混响效果器全局输出增益
08	合唱总输出电平	7F (127)	合唱效果器全局输出增益
09	Variation 总输出电平	7F (127)	SC-88 及以上机型支持，基础 GS 机型无此参数
0A	输出模式	00	0 = 立体声，1 = 单声道，2 = 耳机优化
7F	GS 模式控制	写入 00 执行复位	仅写入触发，无读取意义；写入 7F 退出 GS 模式
*/

use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct System {
    pub master_volume: u8,
    pub master_tune_msb: u8,
    pub master_tune_lsb: u8,
    pub reverb_output_level: u8,
    pub chorus_output_level: u8,
    pub variation_output_level: u8,
    pub output_mode: u8,
    pub sysetm_reset: u8,
}

impl System {
    pub fn new() -> Self {
        Self {
            master_volume: 0x7F,
            master_tune_msb: 0x40,
            master_tune_lsb: 0x00,
            reverb_output_level: 0x7F,
            chorus_output_level: 0x7F,
            variation_output_level: 0x7F,
            output_mode: 0x00,
            sysetm_reset: 0x00,
        }
    }

    // 14bit
    pub fn get_master_tune(&self) -> u16 {
        (self.master_tune_msb as u16) << 7 | self.master_tune_lsb as u16
    }

    pub fn set_master_tune(&mut self, value: u16) {
        let lsb = value & 0x7F;
        let msb = (value >> 7) & 0x7F;
        self.master_tune_msb = msb as u8;
        self.master_tune_lsb = lsb as u8;
    }
}

impl std::ops::Index<usize> for System {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0x04 => &self.master_volume,
            0x05 => &self.master_tune_msb,
            0x06 => &self.master_tune_lsb,
            0x07 => &self.reverb_output_level,
            0x08 => &self.chorus_output_level,
            0x09 => &self.variation_output_level,
            0x0A => &self.output_mode,
            0x7F => &self.sysetm_reset,

            _ => panic!("System: index {} out of bounds", index),
        }
    }
}

impl std::ops::IndexMut<usize> for System {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x04 => &mut self.master_volume,
            0x05 => &mut self.master_tune_msb,
            0x06 => &mut self.master_tune_lsb,
            0x07 => &mut self.reverb_output_level,
            0x08 => &mut self.chorus_output_level,
            0x09 => &mut self.variation_output_level,
            0x0A => &mut self.output_mode,
            0x7F => &mut self.sysetm_reset,

            _ => panic!("System: index {} out of bounds", index),
        }
    }
}

impl Memory for System {
    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x04..=0x0A | 0x7F) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn reset(&mut self) {
        self.master_volume = 0x7F;
        self.master_tune_msb = 0x40;
        self.master_tune_lsb = 0x00;
        self.reverb_output_level = 0x7F;
        self.chorus_output_level = 0x7F;
        self.variation_output_level = 0x7F;
        self.output_mode = 0x00;
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x04..=0x0A | 0x7F) {
            return Err(err);
        }
        Ok(self[addr as usize] = value)
    }
}
