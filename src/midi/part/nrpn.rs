use crate::midi::ram::{MemoryAddr, RAM};

pub fn nrpn_to_addr(id: usize, ram: &RAM, param_id: u16) -> Option<MemoryAddr> {
    let param_msb = (param_id >> 7).min(0x7F) as u8;
    let param_lsb = (param_id & 0x7F) as u8;

    let part_mode = ram.xg.multi_part[id].part_mode;
    let addr = |hi: u8, lo: u8| MemoryAddr::new(hi, id as u8, lo);
    let drum = |lo: u8| {
        if part_mode < 2 {
            None
        } else {
            Some(MemoryAddr::new(
                (0x30 | part_mode - 2).min(0x3F),
                param_lsb,
                lo,
            ))
        }
    };

    match (param_msb, param_lsb) {
        (0x01, 0x08) => Some(addr(0x08, 0x15)), // Vibrato Rate
        (0x01, 0x09) => Some(addr(0x08, 0x16)), // Vibrato Depth
        (0x01, 0x0A) => Some(addr(0x08, 0x17)), // Vibrato Delay
        (0x01, 0x20) => Some(addr(0x08, 0x18)), // Filter Cutoff Frequency
        (0x01, 0x21) => Some(addr(0x08, 0x19)), // Filter Resonance
        (0x01, 0x24) => Some(addr(0x0A, 0x20)), // HPF Cutoff Frequency
        (0x01, 0x25) => Some(addr(0x0A, 0x21)), // HPF Resonance (reserved)
        (0x01, 0x30) => Some(addr(0x08, 0x72)), // EQ BASS
        (0x01, 0x31) => Some(addr(0x08, 0x73)), // EQ TREBLE
        (0x01, 0x32) => Some(addr(0x08, 0x74)), // EQ MID-BASS
        (0x01, 0x33) => Some(addr(0x08, 0x75)), // EQ MID-TREBLE
        (0x01, 0x34) => Some(addr(0x08, 0x76)), // EQ BASS Frequency
        (0x01, 0x35) => Some(addr(0x08, 0x77)), // EQ TREBLE Frequency
        (0x01, 0x36) => Some(addr(0x08, 0x78)), // EQ MID-BASS Frequency
        (0x01, 0x37) => Some(addr(0x08, 0x79)), // EQ MID-TREBLE Frequency
        (0x01, 0x38) => Some(addr(0x08, 0x7A)), // EQ Bass Q
        (0x01, 0x39) => Some(addr(0x08, 0x7B)), // EQ Treble Q
        (0x01, 0x3A) => Some(addr(0x08, 0x7C)), // EQ MID-BASS Q
        (0x01, 0x3B) => Some(addr(0x08, 0x7D)), // EQ MID-TREBLE Q
        (0x01, 0x3C) => Some(addr(0x08, 0x7E)), // EQ BASS SHAPE
        (0x01, 0x3D) => Some(addr(0x08, 0x7F)), // EQ TREBLE SHAPE
        // (0x01, 0x3E), (0x01, 0x3F): reserved for EQ, no memory addr
        (0x01, 0x63) => Some(addr(0x08, 0x1A)), // EG Attack Time
        (0x01, 0x64) => Some(addr(0x08, 0x1B)), // EG Decay Time
        (0x01, 0x66) => Some(addr(0x08, 0x1C)), // EG Release Time

        // Drums — 仅在声部处于节奏模式时有效, rr(NRPN LSB) = 鼓乐器音符编号
        (0x14, _) => drum(0x0B), // Drum Filter Cutoff Frequency
        (0x15, _) => drum(0x0C), // Drum Filter Resonance
        (0x16, _) => drum(0x0D), // Drum EG Attack Rate
        (0x17, _) => drum(0x0E), // Drum EG Decay Rate
        (0x18, _) => drum(0x00), // Drum Pitch Coarse
        (0x19, _) => drum(0x01), // Drum Pitch Fine
        (0x1A, _) => drum(0x02), // Drum Level
        (0x1C, _) => drum(0x04), // Drum Pan
        (0x1D, _) => drum(0x05), // Drum Reverb Send
        (0x1E, _) => drum(0x06), // Drum Chorus Send
        (0x1F, _) => drum(0x07), // Drum Variation Send

        (0x24, _) => drum(0x50),             // Drum HPF Cutoff Frequency
        (0x25, _) => drum(0x51),             // Drum HPF Resonance (reserved)
        (0x30, _) => drum(0x20),             // Drum EQ BASS
        (0x31, _) => drum(0x21),             // Drum EQ TREBLE
        (0x32, _) => drum(param_msb - 0x10), // Drum EQ MID-BASS
        (0x33, _) => drum(param_msb - 0x10), // Drum EQ MID-TREBLE
        (0x34, _) => drum(param_msb - 0x10), // Drum EQ BASS Frequency
        (0x35, _) => drum(param_msb - 0x10), // Drum EQ TREBLE Frequency
        (0x36, _) => drum(param_msb - 0x10), // Drum EQ MID-BASS Frequency
        (0x37, _) => drum(param_msb - 0x10), // Drum EQ MID-TREBLE Frequency
        (0x38, _) => drum(param_msb - 0x10), // Drum EQ Bass Q
        (0x39, _) => drum(param_msb - 0x10), // Drum EQ Treble Q
        (0x3A, _) => drum(param_msb - 0x10), // Drum EQ MID-BASS Q
        (0x3B, _) => drum(param_msb - 0x10), // Drum EQ MID-TREBLE Q
        (0x3C, _) => drum(param_msb - 0x10), // Drum EQ BASS SHAPE
        (0x3D, _) => drum(param_msb - 0x10), // Drum EQ TREBLE SHAPE
        //(0x3E, _) // reserved for EQ
        //(0x3F, _) // reserved for EQ
        (0x40, _) => drum(0x60), // Drum VELOCITY PITCH SENS.
        (0x41, _) => drum(0x61), // Drum VELOCITY LPF CUTOFF SENS.

        _ => None,
    }
}
