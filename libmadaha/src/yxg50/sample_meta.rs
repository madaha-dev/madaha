// sample info for tbl dataseg16

use serde::{Deserialize, Serialize};

use crate::{to_f32::u8_to_f32, yxg50::interface::HasSample};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMeta {
    /// velocity, 0 is the max
    pub velocity: u8,

    /// sample in note key
    pub base_key: u8,

    /// tone in cents
    pub tone: u8,

    /// negtive offset for base_addr, big-endian
    /// start_addr = loop_start - start_point_offset
    pub start_point_offset: usize,

    /// loop end point, big-endian
    /// loop_end = loop_start + loop_length
    pub loop_length: usize,

    /// base address for sample, but not the start, big-endian
    pub loop_start: usize,

    /// channel flag, 0x80=mono, 0x00=stereo
    pub channel_flag: u8,

    /// sample rate for output, 0x00 = 44100
    pub _reserved: u8,

    /// key range fot this sample
    pub pitch_fine: u8,
    pub key_end: u8,

    #[serde(skip)]
    pub pcm: Option<Box<[f32]>>,

}

impl From<&[u8; 16]> for SampleMeta {
    fn from(data: &[u8; 16]) -> Self {
        Self {
            velocity: data[0],
            base_key: data[1],
            tone: data[2],
            start_point_offset: sample_meta_addr([data[3], data[4], data[5]]),
            loop_length: sample_meta_addr([data[6], data[7], data[8]]),
            loop_start: sample_meta_addr([data[9], data[10], data[11]]),
            channel_flag: data[12],
            _reserved: data[13],
            pitch_fine: data[14],
            key_end: data[15],
            pcm: None,
        }
    }
}

impl From<&[u8]> for SampleMeta {
    fn from(data: &[u8]) -> Self {
        Self {
            velocity: data[0],
            base_key: data[1],
            tone: data[2],
            start_point_offset: sample_meta_addr([data[3], data[4], data[5]]),
            loop_length: sample_meta_addr([data[6], data[7], data[8]]),
            loop_start: sample_meta_addr([data[9], data[10], data[11]]),
            channel_flag: data[12],
            _reserved: data[13],
            pitch_fine: data[14],
            key_end: data[15],
            pcm: None,
        }
    }
}

impl SampleMeta {
    pub fn check_key(&self, note: u8) -> bool {
        note <= (self.key_end & 0x7F)
    }

    pub fn is_last(&self) -> bool {
        self.key_end & 0x80 != 0
    }
}

impl HasSample for SampleMeta {
    fn set_wave(&mut self, wave: &Box<[u8]>) -> Self {
        // channel_flag=0x00: 16-bit PCM（word 单位）。negOffset/loopStart 单位是
        // word（16-bit 采样），字节偏移 = 值 ×2。渲染器（0x1001ad60 等）按 word
        // 读取并线性插值：word = 低字节 | 高字节<<8（小端），^0x5C 是原版防 dump
        // 层（madaha 数据已解码，不需要）。flags=0x80 为 8-bit（字节单位，×1）。
        let scale = if self.channel_flag & 0x80 == 0 { 2 } else { 1 };
        let start = self.loop_start - self.start_point_offset * scale;
        let end = self.loop_start + self.loop_length * scale;

        if let Some(wp) = wave.get(start..end) {
            let pcm: Box<[f32]> = if self.channel_flag & 0x80 == 0 {
                wp.chunks_exact(2)
                    .map(|b| {
                        let w = (b[0] as i32 | (b[1] as i32) << 8) - 0x8000;
                        w as f32 / 0x8000 as f32
                    })
                    .collect()
            } else {
                wp.into_iter().map(|&b| u8_to_f32(b)).collect()
            };

            self.pcm = Some(pcm);
        }
        self.clone()
    }
}

pub fn sample_meta_addr(input: [u8; 3]) -> usize {
    (input[0] as usize) << 16 | (input[1] as usize) << 8 | input[2] as usize
}
