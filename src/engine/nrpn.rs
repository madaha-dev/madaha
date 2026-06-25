use crate::engine::ram::MemoryAddr;

pub fn nrpn_to_addr(drum_setup: u8, channel: u8, msb: u8, lsb: u8) -> Option<MemoryAddr> {
    let addr = |lo: u8| MemoryAddr::new(0x08, channel, lo);
    let drum = |lo: u8| MemoryAddr::new(0x30 | drum_setup, lsb, lo);
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
        0x14 => Some(drum(0x0B)),
        0x15 => Some(drum(0x0C)),
        0x16 => Some(drum(0x0D)),
        0x17 => Some(drum(0x0E)),
        0x18 => Some(drum(0x00)),
        0x19 => Some(drum(0x01)),
        0x1A => Some(drum(0x02)),
        0x1C => Some(drum(0x04)),
        0x1D => Some(drum(0x05)),
        0x1E => Some(drum(0x06)),
        0x1F => Some(drum(0x07)),
        _ => None,
    }
}
