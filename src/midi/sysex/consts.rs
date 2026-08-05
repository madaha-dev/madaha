pub const SYSEX_MSG_START: u8 = 0xF0;
pub const SYSEX_MSG_END: u8 = 0xF7;

pub const SYSEX_CHANNEL_ALL_DEVICE: u8 = 0x7F;

// ── Universal SysEx sub-ids (MMA GM/GM2) ─────────────────────────
// Non-Real Time (0x7E) / Real Time (0x7F) share these sub-id layouts
pub const SUB_ID1_DEVICE_CONTROL: u8 = 0x04;
pub const SUB_ID2_MASTER_VOLUME: u8 = 0x01;        // 7E/7F 04 01 [01] <lsb> <msb>
pub const SUB_ID2_MASTER_COARSE_TUNING: u8 = 0x02; // 7E 04 02 01 <semitones>
pub const SUB_ID2_MASTER_FINE_TUNING: u8 = 0x03;   // 7E 04 03 01 <lsb> <msb>
pub const SUB_ID2_GM2_EFFECTS: u8 = 0x05;          // 7F 04 05 01 ... <fx> <pp> <vv>
pub const SUB_ID1_MIDI_TUNING: u8 = 0x08;
pub const SUB_ID2_SCALE_OCTAVE_TUNING: u8 = 0x01;  // 7E 08 01 <note> <adjust>
pub const SUB_ID1_MTS: u8 = 0x08;                  // (realtime) MIDI Tuning Standard
pub const SUB_ID2_SINGLE_NOTE_RETUNE: u8 = 0x02;   // 7F 08 02
pub const SUB_ID2_SINGLE_BANK_NOTE_RETUNE: u8 = 0x07; // 7F 08 07
pub const SUB_ID1_GM2_SYSTEM: u8 = 0x0A;           // 7F 0A 01 (Key-Based)
pub const SUB_ID2_KEY_BASED: u8 = 0x01;
pub const SUB_ID1_GM2_DESTINATION: u8 = 0x09;      // 7F 09 01/03
pub const SUB_ID2_CHANNEL_PRESSURE_DEST: u8 = 0x01;
pub const SUB_ID2_CC_DEST: u8 = 0x03;

pub const DEFAULT_DATA_SEG: &'static [u8] = &[0, 0, 0, 0, 0, 0, 0];
