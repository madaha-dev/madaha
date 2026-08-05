use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum PartEngine {
    /// Default engine, use wavetable.
    #[default]
    AWM2 = 0x7F,

    /// PLG-VL engine, low priority.
    VL = 0x00,
    /// PLG-SG engine, will not work, just silence.
    SG = 0x01,
    /// PLG-DX engine, low priority. Dexed as reference.
    DX = 0x02,
    /// PLG-PF engine, will not work, just silence.
    PF = 0x03,
    /// PLG-AN engine, low priority.
    AN = 0x04,
    /// PLG-DR engine, will not work, just silence.
    DR = 0x05,
    /// PLG-PC engine, will not work, just silence.
    PC = 0x06,
    /// PLG-XG engine, aka AWM2
    XG = 0x07,

    /// OPL-3 FM engine, low priority.
    OPL3 = 0x40,
    /// RP2A03 APU engine, aka Nintendo FC, low priority.
    RP2A03 = 0x41,
}
