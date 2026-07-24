/// S-YXG2006LE Drum Setup TBL structure
/// for sxgbnw6l.tbl, Section 0x7.
/// Each entry is 24 bytes, matching the binary layout from drumSetup.tbl.h

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrumSetupEntry {
    pub pitch_coarse: u8,
    pub pitch_fine: u8,
    pub level: u8,
    pub alter_group: u8,
    pub pan: u8,
    pub reverb_send: u8,
    pub chorus_send: u8,
    pub variation_send: u8,
    pub key_assign: u8,   // 0=Single, 1=Multi
    pub rcv_note_off: u8, // bool
    pub rcv_note_on: u8,  // bool
    pub filter_cutoff_freq: u8,
    pub filter_resonance: u8,
    pub eg_attack: u8,
    pub eg_decay1: u8,
    pub eg_decay2: u8,
    pub eq_bass: u8,
    pub eq_treble: u8,
    pub eq_bass_freq: u8,
    pub eq_treble_freq: u8,
    pub output_select: u8,
    pub hpf_cutoff_freq: u8,
    pub vel_pitch_sense: u8,
    pub vel_lpf_cutoff_sense: u8,
}

impl From<Box<[u8]>> for DrumSetupEntry {
    fn from(data: Box<[u8]>) -> Self {
        Self {
            pitch_coarse: data[0],
            pitch_fine: data[1],
            level: data[2],
            alter_group: data[3],
            pan: data[4],
            reverb_send: data[5],
            chorus_send: data[6],
            variation_send: data[7],
            key_assign: data[8],
            rcv_note_off: data[9],
            rcv_note_on: data[10],
            filter_cutoff_freq: data[11],
            filter_resonance: data[12],
            eg_attack: data[13],
            eg_decay1: data[14],
            eg_decay2: data[15],
            eq_bass: data[16],
            eq_treble: data[17],
            eq_bass_freq: data[18],
            eq_treble_freq: data[19],
            output_select: data[20],
            hpf_cutoff_freq: data[21],
            vel_pitch_sense: data[22],
            vel_lpf_cutoff_sense: data[23],
        }
    }
}

impl From<[u8; 24]> for DrumSetupEntry {
    fn from(data: [u8; 24]) -> Self {
        Self {
            pitch_coarse: data[0],
            pitch_fine: data[1],
            level: data[2],
            alter_group: data[3],
            pan: data[4],
            reverb_send: data[5],
            chorus_send: data[6],
            variation_send: data[7],
            key_assign: data[8],
            rcv_note_off: data[9],
            rcv_note_on: data[10],
            filter_cutoff_freq: data[11],
            filter_resonance: data[12],
            eg_attack: data[13],
            eg_decay1: data[14],
            eg_decay2: data[15],
            eq_bass: data[16],
            eq_treble: data[17],
            eq_bass_freq: data[18],
            eq_treble_freq: data[19],
            output_select: data[20],
            hpf_cutoff_freq: data[21],
            vel_pitch_sense: data[22],
            vel_lpf_cutoff_sense: data[23],
        }
    }
}

impl Into<[u8; 24]> for DrumSetupEntry {
    fn into(self) -> [u8; 24] {
        [
            self.pitch_coarse,
            self.pitch_fine,
            self.level,
            self.alter_group,
            self.pan,
            self.reverb_send,
            self.chorus_send,
            self.variation_send,
            self.key_assign,
            self.rcv_note_off,
            self.rcv_note_on,
            self.filter_cutoff_freq,
            self.filter_resonance,
            self.eg_attack,
            self.eg_decay1,
            self.eg_decay2,
            self.eq_bass,
            self.eq_treble,
            self.eq_bass_freq,
            self.eq_treble_freq,
            self.output_select,
            self.hpf_cutoff_freq,
            self.vel_pitch_sense,
            self.vel_lpf_cutoff_sense,
        ]
    }
}
