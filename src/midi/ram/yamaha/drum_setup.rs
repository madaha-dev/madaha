use num_enum::{FromPrimitive, IntoPrimitive};

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
    pub filter_cutoff: u8,
    pub filter_resonance: u8,
    pub eg_attack_rate: u8,
    pub eg_decay1_rate: u8,
    pub eg_decay2_rate: u8,
}

impl DrumSetup {
    pub const fn new() -> Self {
        Self {
            pitch_coarse: 0xFF,
            pitch_fine: 0xFf,
            level: 0xFf,
            alternate_group: 0xFF,
            pan: 0xFF,
            reverb_send: 0xFF,
            chorus_send: 0xFF,
            variation_send: 0xFF,
            key_assign: 0xFF,
            rcv_note_off: 0xFF,
            rcv_note_on: 0xFF,
            filter_cutoff: 0xFF,
            filter_resonance: 0xFF,
            eg_attack_rate: 0xFF,
            eg_decay1_rate: 0xFF,
            eg_decay2_rate: 0xFF,
        }
    }

    pub fn from_tbl(data: Box<[u8]>) -> Self {
        todo!()
    }
}

impl From<[u8; 16]> for DrumSetup {
    fn from(value: [u8; 16]) -> Self {
        Self {
            pitch_coarse: value[0],
            pitch_fine: value[1],
            level: value[2],
            alternate_group: value[3],
            pan: value[4],
            reverb_send: value[5],
            chorus_send: value[6],
            variation_send: value[7],
            key_assign: value[8],
            rcv_note_off: value[9],
            rcv_note_on: value[10],
            filter_cutoff: value[11],
            filter_resonance: value[12],
            eg_attack_rate: value[13],
            eg_decay1_rate: value[14],
            eg_decay2_rate: value[15],
        }
    }
}

impl From<Box<[u8]>> for DrumSetup {
    fn from(value: Box<[u8]>) -> Self {
        Self {
            pitch_coarse: value[0],
            pitch_fine: value[1],
            level: value[2],
            alternate_group: value[3],
            pan: value[4],
            reverb_send: value[5],
            chorus_send: value[6],
            variation_send: value[7],
            key_assign: value[8],
            rcv_note_off: value[9],
            rcv_note_on: value[10],
            filter_cutoff: value[11],
            filter_resonance: value[12],
            eg_attack_rate: value[13],
            eg_decay1_rate: value[14],
            eg_decay2_rate: value[15],
        }
    }
}

impl std::ops::Index<usize> for DrumSetup {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.pitch_coarse,
            1 => &self.pitch_fine,
            2 => &self.level,
            3 => &self.alternate_group,
            4 => &self.pan,
            5 => &self.reverb_send,
            6 => &self.chorus_send,
            7 => &self.variation_send,
            8 => &self.key_assign,
            9 => &self.rcv_note_off,
            10 => &self.rcv_note_on,
            11 => &self.filter_cutoff,
            12 => &self.filter_resonance,
            13 => &self.eg_attack_rate,
            14 => &self.eg_decay1_rate,
            15 => &self.eg_decay2_rate,
            _ => panic!("DrumSetup: index {} out of bounds", index),
        }
    }
}

impl std::ops::IndexMut<usize> for DrumSetup {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.pitch_coarse,
            1 => &mut self.pitch_fine,
            2 => &mut self.level,
            3 => &mut self.alternate_group,
            4 => &mut self.pan,
            5 => &mut self.reverb_send,
            6 => &mut self.chorus_send,
            7 => &mut self.variation_send,
            8 => &mut self.key_assign,
            9 => &mut self.rcv_note_off,
            10 => &mut self.rcv_note_on,
            11 => &mut self.filter_cutoff,
            12 => &mut self.filter_resonance,
            13 => &mut self.eg_attack_rate,
            14 => &mut self.eg_decay1_rate,
            15 => &mut self.eg_decay2_rate,
            _ => panic!("DrumSetup: index {} out of bounds", index),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum KeyAssign {
    #[default]
    Single,
    Multi,
    Drum,
}

/*
impl From<u8> for KeyAssign {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Multi,
            2 => Self::Drum,
            _ => Self::Single,
        }
    }
}

impl Into<u8> for KeyAssign {
    fn into(self) -> u8 {
        match self {
            Self::Drum => 2,
            Self::Multi => 1,
            Self::Single => 0,
        }
    }
}
*/
