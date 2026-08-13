use serde::{Deserialize, Serialize};

use crate::{to_f32::u8_to_f32, yxg50::interface::HasSample};

use super::sample_meta::sample_meta_addr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumSetupEntry {
    pub pitch_coarse: u8,
    pub pitch_fine: u8,
    pub level: u8,
    pub alternate_group: u8,
    pub pan: u8,
    pub reverb_send: u8,
    pub chorus_send: u8,
    pub variation_send: u8,
    pub key_assign: u8, // 0 = single, 1 = multi
    pub rcv_note_off: bool,
    pub rcv_note_on: bool,
    pub filter_cutoff_freq: u8,
    pub filter_resonance: u8,
    pub eg_attack: u8,
    pub eg_decay1: u8,
    pub eg_decay2: u8,

    pub drum_key_type: u8, // 0x00 = SFX, 0xFF = Drum
    pub sfx_sound_id: u8,  // not sure
    pub base_key: u8,      // key for sample
    pub start_point_offset: usize,
    pub _reserved_1: u8, // not sure
    pub loop_length: usize,
    pub loop_start: usize, // aka sample base addr
    pub channel_flag: u8,   // 0x80 = 22050Hz, 0x00 = 44100Hz
    pub wave_proc_mode: [u8; 2],

    #[serde(skip)]
    pub pcm: Option<Box<[f32]>>,
}

impl From<Box<[u8]>> for DrumSetupEntry {
    fn from(data: Box<[u8]>) -> Self {
        Self {
            pitch_coarse: data[0],
            pitch_fine: data[1],
            level: data[2],
            alternate_group: data[3],
            pan: data[4],
            reverb_send: data[5],
            chorus_send: data[6],
            variation_send: data[7],
            key_assign: data[8],
            rcv_note_off: data[9] == 1,
            rcv_note_on: data[10] == 1,
            filter_cutoff_freq: data[11],
            filter_resonance: data[12],
            eg_attack: data[13],
            eg_decay1: data[14],
            eg_decay2: data[15],
            drum_key_type: data[16],
            sfx_sound_id: data[17],
            base_key: data[18],
            start_point_offset: (data[19] as usize) << 8 | (data[20] as usize),
            _reserved_1: data[21],
            loop_length: (data[22] as usize) << 8 | (data[23] as usize),
            loop_start: sample_meta_addr([data[24], data[25], data[26]]),
            channel_flag: data[27],
            wave_proc_mode: data[28..=29].try_into().unwrap(),
            pcm: None,
        }
    }
}

impl From<&[u8]> for DrumSetupEntry {
    fn from(data: &[u8]) -> Self {
        Self {
            pcm: None,
            pitch_coarse: data[0],
            pitch_fine: data[1],
            level: data[2],
            alternate_group: data[3],
            pan: data[4],
            reverb_send: data[5],
            chorus_send: data[6],
            variation_send: data[7],
            key_assign: data[8],
            rcv_note_off: data[9] == 1,
            rcv_note_on: data[10] == 1,
            filter_cutoff_freq: data[11],
            filter_resonance: data[12],
            eg_attack: data[13],
            eg_decay1: data[14],
            eg_decay2: data[15],
            drum_key_type: data[16],
            sfx_sound_id: data[17],
            base_key: data[18],
            start_point_offset: (data[19] as usize) << 8 | (data[20] as usize),
            _reserved_1: data[21],
            loop_length: (data[22] as usize) << 8 | (data[23] as usize),
            loop_start: sample_meta_addr([data[24], data[25], data[26]]),
            channel_flag: data[27],
            wave_proc_mode: data[28..=29].try_into().unwrap(),
        }
    }
}

impl HasSample for DrumSetupEntry {
    fn set_wave(&mut self, wave: &Box<[u8]>) -> Self {
        // channel_flag=0x00: 16-bit PCM（word 单位，与 sample_meta 相同语义）；
        // flags=0x80 为 8-bit（字节单位，×1）。
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
