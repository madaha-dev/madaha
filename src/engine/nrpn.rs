use crate::engine::ram::{MemoryAddr, RAM};

pub fn nrpn_to_addr(ram: &RAM, channel: u8, msb: u8, lsb: u8) -> Option<MemoryAddr> {
    let part_mode = ram.xg.multi_part[channel as usize].part_mode;

    let addr = |lo: u8| MemoryAddr::new(0x08, channel, lo);
    let drum = |lo: u8| {
        if part_mode < 2 {
            None
        } else {
            Some(MemoryAddr::new(0x30 | (part_mode - 2), lsb, lo))
        }
    };

    match msb {
        // Multi part
        0x01 => match lsb {
            0x08 => Some(addr(0x15)),
            0x09 => Some(addr(0x16)),
            0x0A => Some(addr(0x17)),

            0x20 => Some(addr(0x18)),
            0x21 => Some(addr(0x19)),

            0x63 => Some(addr(0x1A)),
            0x64 => Some(addr(0x1B)),
            0x66 => Some(addr(0x1C)),
            _ => None,
        },
        // Drum setup
        0x14 => drum(0x0B),
        0x15 => drum(0x0C),
        0x16 => drum(0x0D),
        0x17 => drum(0x0E),
        0x18 => drum(0x00),
        0x19 => drum(0x01),
        0x1A => drum(0x02),
        0x1C => drum(0x04),
        0x1D => drum(0x05),
        0x1E => drum(0x06),
        0x1F => drum(0x07),
        _ => None,
    }
}
