use crate::engine::{consts::DRUM_CHANNEL_ID, ram::MemoryAddr};

pub fn gs_xg_addr_remap(addr: MemoryAddr) -> Option<MemoryAddr> {
    let (h, m, l) = addr.split();
    match h {
        0x40 => match m {
            // System
            0x00 => match l {
                0x00..=0x04 => Some(MemoryAddr::new(0x00, 0x00, l)),
                // GS Key Shift -> XG Transpost
                0x05 => Some(MemoryAddr::new(0x00, 0x00, 0x06)),
                // GS Master pan not supported, skip.
                // And reset function skip.
                _ => None,
            },
            // Patch part -> Effect 1
            0x01 => match l {
                // 00 - 0F Patch Name, skip
                // 0x30 Reverb macro -> reverb type msb
                0x30 => Some(MemoryAddr::new(0x02, 0x01, 0x00)),
                // 0x31 Reverb character
                0x31 => Some(MemoryAddr::new(0x02, 0x01, 0x03)),
                // 0x32 Reverb pre-lpf ->
                0x32 => Some(MemoryAddr::new(0x02, 0x01, 0x06)),
                // 0x33 Reverb level
                0x33 => Some(MemoryAddr::new(0x02, 0x01, 0x0C)),
                // 0x34 Reverb time -> reverb time
                0x34 => Some(MemoryAddr::new(0x02, 0x01, 0x02)),
                // 0x35 Reverb delay feedback -> rev delay
                0x35 => Some(MemoryAddr::new(0x02, 0x01, 0x0F)),
                // 0x36 Reverb send to chorus, skip

                // 0x38 Chorus macro -> chorus type msb
                0x38 => Some(MemoryAddr::new(0x02, 0x01, 0x20)),
                // 0x39 Chorus pre-lpf, skip
                // 0x3A Chorus level
                0x3A => Some(MemoryAddr::new(0x02, 0x01, 0x2C)),
                // 0x3B Chorus feedback
                0x3B => Some(MemoryAddr::new(0x02, 0x01, 0x24)),
                // 0x3C Chorus delay
                0x3C => Some(MemoryAddr::new(0x02, 0x01, 0x25)),
                // 0x3D Chorus rate
                0x3D => Some(MemoryAddr::new(0x02, 0x01, 0x22)),
                // 0x3E Chorus depth
                0x3E => Some(MemoryAddr::new(0x02, 0x01, 0x23)),
                // 0x3F Chorus send to reverb
                0x3F => Some(MemoryAddr::new(0x02, 0x01, 0x2E)),

                _ => None,
            },
            // Patch part -> Multi part
            0x10..=0x1F => {
                let channel = roland_channel_to_yamaha_channel(m & 0xF);
                match l {
                    // 0x00 Bank select msb
                    0x00 => Some(MemoryAddr::new(0x08, channel, 0x01)),
                    // 0x01 Program
                    0x01 => Some(MemoryAddr::new(0x08, channel, 0x03)),
                    // 0x02 Rx Channel
                    0x02 => Some(MemoryAddr::new(0x08, channel, 0x04)),
                    // 0x03 Rx pitch bend
                    0x03 => Some(MemoryAddr::new(0x08, channel, 0x30)),
                    // 0x04 Rx CAT
                    0x04 => Some(MemoryAddr::new(0x08, channel, 0x31)),
                    // 0x05 Rx program change
                    0x05 => Some(MemoryAddr::new(0x08, channel, 0x32)),
                    // 0x06 Rx control change
                    0x06 => Some(MemoryAddr::new(0x08, channel, 0x33)),
                    // 0x07 Rx PAT
                    0x07 => Some(MemoryAddr::new(0x08, channel, 0x34)),
                    // 0x08 Rx note message
                    0x08 => Some(MemoryAddr::new(0x08, channel, 0x35)),
                    // 0x09 Rx rpn
                    0x09 => Some(MemoryAddr::new(0x08, channel, 0x36)),
                    // 0x0A Rx nrpn
                    0x0A => Some(MemoryAddr::new(0x08, channel, 0x37)),
                    // 0x0B Rx moduration
                    0x0B => Some(MemoryAddr::new(0x08, channel, 0x38)),
                    // 0x0C Rx volume
                    0x0C => Some(MemoryAddr::new(0x08, channel, 0x39)),
                    // 0x0D Rx pan
                    0x0D => Some(MemoryAddr::new(0x08, channel, 0x3A)),
                    // 0x0E Rx expression
                    0x0E => Some(MemoryAddr::new(0x08, channel, 0x3B)),
                    // 0x0F Rx hold1
                    0x0F => Some(MemoryAddr::new(0x08, channel, 0x3C)),
                    // 0x10 Rx portamento
                    0x10 => Some(MemoryAddr::new(0x08, channel, 0x3D)),
                    // 0x11 Rx sostenuto
                    0x11 => Some(MemoryAddr::new(0x08, channel, 0x3E)),
                    // 0x12 Rx soft
                    0x12 => Some(MemoryAddr::new(0x08, channel, 0x3F)),
                    // 0x13 mono poly mode
                    0x13 => Some(MemoryAddr::new(0x08, channel, 0x05)),
                    // 0x14 assign mode
                    0x14 => Some(MemoryAddr::new(0x08, channel, 0x06)),
                    // 0x15 use for rhythm part, drum for map1(1), other for off(0)
                    0x15 => Some(MemoryAddr::new(0x08, channel, 0x07)),
                    // 0x16 pitch key shift
                    0x16 => Some(MemoryAddr::new(0x08, channel, 0x08)),
                    // 0x17-0x18 pitch offset fine
                    0x17 => Some(MemoryAddr::new(0x08, channel, 0x09)),
                    0x18 => Some(MemoryAddr::new(0x08, channel, 0x0A)),
                    // 0x19 part level
                    0x19 => Some(MemoryAddr::new(0x08, channel, 0x0B)),
                    // 0x1A velocity sense depth
                    0x1A => Some(MemoryAddr::new(0x08, channel, 0x0C)),
                    // 0x1B velocity sense offset
                    0x1B => Some(MemoryAddr::new(0x08, channel, 0x0D)),
                    // 0x1C part pan
                    0x1C => Some(MemoryAddr::new(0x08, channel, 0x0E)),
                    // 0x1D key range low
                    0x1D => Some(MemoryAddr::new(0x08, channel, 0x0F)),
                    // 0x1E key range high
                    0x1E => Some(MemoryAddr::new(0x08, channel, 0x10)),
                    // 0x1F cc1 control number
                    0x1F => Some(MemoryAddr::new(0x08, channel, 0x59)),
                    // 0x20 cc2 control number
                    0x20 => Some(MemoryAddr::new(0x08, channel, 0x60)),
                    // 0x21 chorus send depth
                    0x21 => Some(MemoryAddr::new(0x08, channel, 0x12)),
                    // 0x22 reverb send depth
                    0x22 => Some(MemoryAddr::new(0x08, channel, 0x13)),
                    // 0x30 vibrato rate
                    0x30 => Some(MemoryAddr::new(0x08, channel, 0x15)),
                    // 0x31 vibrato depth
                    0x31 => Some(MemoryAddr::new(0x08, channel, 0x16)),
                    // 0x32 tvf cutoff freq
                    0x32 => Some(MemoryAddr::new(0x08, channel, 0x18)),
                    // 0x33 tvf resonance
                    0x33 => Some(MemoryAddr::new(0x08, channel, 0x19)),
                    // 0x34 tvf&tva env attack
                    0x34 => Some(MemoryAddr::new(0x08, channel, 0x1A)),
                    // 0x35 tvf&tva env decay
                    0x35 => Some(MemoryAddr::new(0x08, channel, 0x1B)),
                    // 0x36 tvf&tva env release
                    0x36 => Some(MemoryAddr::new(0x08, channel, 0x1C)),
                    // 0x37 vibrato delay
                    0x37 => Some(MemoryAddr::new(0x08, channel, 0x17)),
                    // 0x40-0x4B note tuning
                    0x40 => Some(MemoryAddr::new(0x08, channel, 0x41)),
                    0x41 => Some(MemoryAddr::new(0x08, channel, 0x42)),
                    0x42 => Some(MemoryAddr::new(0x08, channel, 0x43)),
                    0x43 => Some(MemoryAddr::new(0x08, channel, 0x44)),
                    0x44 => Some(MemoryAddr::new(0x08, channel, 0x45)),
                    0x45 => Some(MemoryAddr::new(0x08, channel, 0x46)),
                    0x46 => Some(MemoryAddr::new(0x08, channel, 0x47)),
                    0x47 => Some(MemoryAddr::new(0x08, channel, 0x48)),
                    0x48 => Some(MemoryAddr::new(0x08, channel, 0x49)),
                    0x49 => Some(MemoryAddr::new(0x08, channel, 0x4A)),
                    0x4A => Some(MemoryAddr::new(0x08, channel, 0x4B)),
                    0x4B => Some(MemoryAddr::new(0x08, channel, 0x4C)),

                    _ => None,
                }
            }
            0x20..=0x2F => {
                let channel = roland_channel_to_yamaha_channel(m & 0xF);
                match l {
                    // 0x00 mod pitch control
                    0x00 => Some(MemoryAddr::new(0x08, channel, 0x1D)),
                    // 0x01 mod tvf cutoff control
                    0x01 => Some(MemoryAddr::new(0x08, channel, 0x1E)),
                    // 0x02 mod amplitude control
                    0x02 => Some(MemoryAddr::new(0x08, channel, 0x1F)),
                    // 0x03 mod lfo1 rate control
                    // 0x04 mod lfo1 pitch depth
                    0x04 => Some(MemoryAddr::new(0x08, channel, 0x20)),
                    // 0x05 mod lfo1 tvf depth
                    0x05 => Some(MemoryAddr::new(0x08, channel, 0x21)),
                    // 0x06 mod lfo1 tva depth
                    0x06 => Some(MemoryAddr::new(0x08, channel, 0x22)),
                    // 0x07 mod lfo2 rate control
                    // 0x08 mod lfo2 pitch depth
                    // 0x09 mod lfo2 tvf depth
                    // 0x0A mod lfo2 tva depth

                    // 0x10 bend pitch control
                    0x10 => Some(MemoryAddr::new(0x08, channel, 0x23)),
                    // 0x11 bend tvf cutoff controll
                    0x11 => Some(MemoryAddr::new(0x08, channel, 0x24)),
                    // 0x12 bend amplitude control
                    0x12 => Some(MemoryAddr::new(0x08, channel, 0x25)),
                    // 0x13 bend lfo1 rate control
                    // 0x14 bend lfo1 pitch depth
                    0x14 => Some(MemoryAddr::new(0x08, channel, 0x26)),
                    // 0x15 bend lfo1 tvf depth
                    0x15 => Some(MemoryAddr::new(0x08, channel, 0x27)),
                    // 0x16 bend lfo1 tva depth
                    0x16 => Some(MemoryAddr::new(0x08, channel, 0x28)),
                    // 0x17 bend lfo2 rate control
                    // 0x18 bend lfo2 pitch depth
                    // 0x19 bend lfo2 tvf depth
                    // 0x1A bend lfo2 tva depth

                    // 0x20 cat...
                    0x20 => Some(MemoryAddr::new(0x08, channel, 0x4D)),
                    0x21 => Some(MemoryAddr::new(0x08, channel, 0x4E)),
                    0x22 => Some(MemoryAddr::new(0x08, channel, 0x4F)),
                    0x24 => Some(MemoryAddr::new(0x08, channel, 0x50)),
                    0x25 => Some(MemoryAddr::new(0x08, channel, 0x51)),
                    0x26 => Some(MemoryAddr::new(0x08, channel, 0x52)),

                    // 0x30 pat...
                    0x30 => Some(MemoryAddr::new(0x08, channel, 0x53)),
                    0x31 => Some(MemoryAddr::new(0x08, channel, 0x54)),
                    0x32 => Some(MemoryAddr::new(0x08, channel, 0x55)),
                    0x34 => Some(MemoryAddr::new(0x08, channel, 0x56)),
                    0x35 => Some(MemoryAddr::new(0x08, channel, 0x57)),
                    0x36 => Some(MemoryAddr::new(0x08, channel, 0x58)),

                    // 0x40 cc1...
                    0x40 => Some(MemoryAddr::new(0x08, channel, 0x5A)),
                    0x41 => Some(MemoryAddr::new(0x08, channel, 0x5B)),
                    0x42 => Some(MemoryAddr::new(0x08, channel, 0x5C)),
                    0x44 => Some(MemoryAddr::new(0x08, channel, 0x5D)),
                    0x45 => Some(MemoryAddr::new(0x08, channel, 0x5E)),
                    0x46 => Some(MemoryAddr::new(0x08, channel, 0x5F)),

                    // 0x50 cc2...
                    0x50 => Some(MemoryAddr::new(0x08, channel, 0x61)),
                    0x51 => Some(MemoryAddr::new(0x08, channel, 0x62)),
                    0x52 => Some(MemoryAddr::new(0x08, channel, 0x63)),
                    0x53 => Some(MemoryAddr::new(0x08, channel, 0x64)),
                    0x54 => Some(MemoryAddr::new(0x08, channel, 0x65)),
                    0x55 => Some(MemoryAddr::new(0x08, channel, 0x66)),

                    _ => None,
                }
            }

            _ => None,
        },
        // drum setup
        0x41 => {
            let map = m >> 4;
            let param = m & 0xF;
            let key = l;

            if !matches!(key, 0x0D..=0x54) {
                return None;
            }

            match param {
                // Key number (pitch coarse)
                0x01 => Some(MemoryAddr::new(0x30 | map, key, 0x00)),
                // tva level
                0x02 => Some(MemoryAddr::new(0x30 | map, key, 0x02)),
                // assign group
                0x03 => Some(MemoryAddr::new(0x30 | map, key, 0x03)),
                // panpot
                0x04 => Some(MemoryAddr::new(0x30 | map, key, 0x04)),
                // reverb depth
                0x05 => Some(MemoryAddr::new(0x30 | map, key, 0x05)),
                // chorus depth
                0x06 => Some(MemoryAddr::new(0x30 | map, key, 0x06)),
                // rx note off
                0x07 => Some(MemoryAddr::new(0x30 | map, key, 0x09)),
                // rx note on
                0x08 => Some(MemoryAddr::new(0x30 | map, key, 0x0A)),
                _ => None,
            }
        }

        _ => None,
    }
}

fn roland_channel_to_yamaha_channel(roland: u8) -> u8 {
    match roland {
        0 => DRUM_CHANNEL_ID as u8,
        1..=9 => roland - 1,
        10..=15 => roland,
        _ => 0xFF,
    }
}

// TODO: GS -> XG Data map