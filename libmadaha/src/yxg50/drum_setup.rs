use crate::yxg50::interface::HasSample;

use super::sample_meta::sample_meta_addr;

#[derive(Debug, Clone, Copy)]
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
    pub sample_rate: u8,   // 0x80 = 22050Hz, 0x00 = 44100Hz
    pub wave_proc_mode: [u8; 2],

    pub pcm: Option<&'static [f32]>,
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
            sample_rate: data[27],
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
            sample_rate: data[27],
            wave_proc_mode: data[28..=29].try_into().unwrap(),
        }
    }
}

impl HasSample for DrumSetupEntry {
    fn set_wave(&mut self, wave: &Box<[u8]>) -> Self {
        if let Some(wp) =
            wave.get(self.loop_start - self.start_point_offset..self.loop_start + self.loop_length)
        {
            let pcm: Box<[f32]> = if self.sample_rate & 0x80 == 0 {
                self.start_point_offset /= 2;
                self.loop_length /=2;
                wp.into_iter()
                    .map(|&b| (b as f32 - 128.0) / 128.0)
                    .collect()
            } else {
                wp.into_iter()
                    .map(|&b| (b as f32 - 128.0) / 128.0)
                    .collect()
            };

            self.pcm = Some(Box::leak(pcm));
        }
        *self
    }
}
