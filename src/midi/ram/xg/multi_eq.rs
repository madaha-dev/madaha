use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::midi::{errors::MidiError, ram::RAMCallbackEffects};
use std::ops::{Index, IndexMut};

/// Multi-part EQ settings (System, address 0x02)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiEQ {
    /// EQ type (0=Flat, 1=Jazz, 2=Pops, 3=Rock, 4=Concert)
    pub eq_type: u8,
    /// EQ band 1 settings
    pub band1: EQBand,
    /// EQ band 2 settings
    pub band2: EQBand,
    /// EQ band 3 settings
    pub band3: EQBand,
    /// EQ band 4 settings
    pub band4: EQBand,
    /// EQ band 5 settings
    pub band5: EQBand,
}

impl MultiEQ {
    pub fn new() -> Self {
        MULTI_EQ_FLAT
    }
}

impl Index<usize> for MultiEQ {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0x00 => &self.eq_type,
            // band1: 0x01-0x04
            0x01 => &self.band1.gain,
            0x02 => &self.band1.frequency,
            0x03 => &self.band1.q,
            0x04 => &self.band1.shape,
            // band2: 0x05-0x08
            0x05 => &self.band2.gain,
            0x06 => &self.band2.frequency,
            0x07 => &self.band2.q,
            0x08 => &self.band2.shape,
            // band3: 0x09-0x0C
            0x09 => &self.band3.gain,
            0x0A => &self.band3.frequency,
            0x0B => &self.band3.q,
            0x0C => &self.band3.shape,
            // band4: 0x0D-0x10
            0x0D => &self.band4.gain,
            0x0E => &self.band4.frequency,
            0x0F => &self.band4.q,
            0x10 => &self.band4.shape,
            // band5: 0x11-0x14
            0x11 => &self.band5.gain,
            0x12 => &self.band5.frequency,
            0x13 => &self.band5.q,
            0x14 => &self.band5.shape,
            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for MultiEQ {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x00 => &mut self.eq_type,
            0x01 => &mut self.band1.gain,
            0x02 => &mut self.band1.frequency,
            0x03 => &mut self.band1.q,
            0x04 => &mut self.band1.shape,
            0x05 => &mut self.band2.gain,
            0x06 => &mut self.band2.frequency,
            0x07 => &mut self.band2.q,
            0x08 => &mut self.band2.shape,
            0x09 => &mut self.band3.gain,
            0x0A => &mut self.band3.frequency,
            0x0B => &mut self.band3.q,
            0x0C => &mut self.band3.shape,
            0x0D => &mut self.band4.gain,
            0x0E => &mut self.band4.frequency,
            0x0F => &mut self.band4.q,
            0x10 => &mut self.band4.shape,
            0x11 => &mut self.band5.gain,
            0x12 => &mut self.band5.frequency,
            0x13 => &mut self.band5.q,
            0x14 => &mut self.band5.shape,
            _ => panic!("MultiEQ: index {:#X} out of bounds", index),
        }
    }
}

impl Memory for MultiEQ {
    fn reset(&mut self) {
        *self = MULTI_EQ_FLAT;
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x07|0x09..=0x0B|0x0D..=0x0F|0x11..=0x14) {
            return Err(err);
        }
        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<Vec<RAMCallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if addr == 0 {
            match value {
                1 => *self = MULTI_EQ_JAZZ,
                2 => *self = MULTI_EQ_POPS,
                3 => *self = MULTI_EQ_ROCK,
                4 => *self = MULTI_EQ_CONCERT,

                _ => self.reset(),
            }
            return Ok(vec![]);
        }
        if !matches!(addr, 0x01..=0x07|0x09..=0x0B|0x0D..=0x0F|0x11..=0x14) {
            return Err(err);
        }
        self[addr as usize] = value;
        Ok(vec![])
    }
}

/// EQ single band configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EQBand {
    /// EQ band gain
    pub gain: u8,
    /// EQ band frequency
    pub frequency: u8,
    /// EQ band Q (bandwidth)
    pub q: u8,
    /// EQ band shape type
    pub shape: u8,
}

// Flat EQ 预设
const MULTI_EQ_FLAT: MultiEQ = MultiEQ {
    eq_type: 0,
    band1: EQBand {
        gain: 64,
        frequency: 12,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 64,
        frequency: 28,
        q: 7,
        shape: 0,
    },
    band3: EQBand {
        gain: 64,
        frequency: 34,
        q: 7,
        shape: 0,
    },
    band4: EQBand {
        gain: 64,
        frequency: 46,
        q: 7,
        shape: 0,
    },
    band5: EQBand {
        gain: 64,
        frequency: 52,
        q: 7,
        shape: 0,
    },
};

// Jazz EQ 预设
const MULTI_EQ_JAZZ: MultiEQ = MultiEQ {
    eq_type: 1,
    band1: EQBand {
        gain: 58,
        frequency: 8,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 66,
        frequency: 16,
        q: 3,
        shape: 0,
    },
    band3: EQBand {
        gain: 68,
        frequency: 33,
        q: 3,
        shape: 0,
    },
    band4: EQBand {
        gain: 60,
        frequency: 44,
        q: 5,
        shape: 0,
    },
    band5: EQBand {
        gain: 58,
        frequency: 50,
        q: 7,
        shape: 0,
    },
};

// Pops EQ 预设
const MULTI_EQ_POPS: MultiEQ = MultiEQ {
    eq_type: 2,
    band1: EQBand {
        gain: 68,
        frequency: 16,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 60,
        frequency: 24,
        q: 20,
        shape: 0,
    },
    band3: EQBand {
        gain: 67,
        frequency: 34,
        q: 7,
        shape: 0,
    },
    band4: EQBand {
        gain: 60,
        frequency: 40,
        q: 20,
        shape: 0,
    },
    band5: EQBand {
        gain: 70,
        frequency: 48,
        q: 7,
        shape: 0,
    },
};

// Rock EQ 预设
const MULTI_EQ_ROCK: MultiEQ = MultiEQ {
    eq_type: 3,
    band1: EQBand {
        gain: 71,
        frequency: 16,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 68,
        frequency: 20,
        q: 7,
        shape: 0,
    },
    band3: EQBand {
        gain: 60,
        frequency: 36,
        q: 5,
        shape: 0,
    },
    band4: EQBand {
        gain: 68,
        frequency: 41,
        q: 10,
        shape: 0,
    },
    band5: EQBand {
        gain: 66,
        frequency: 50,
        q: 7,
        shape: 0,
    },
};

// Concert EQ 预设
const MULTI_EQ_CONCERT: MultiEQ = MultiEQ {
    eq_type: 4,
    band1: EQBand {
        gain: 67,
        frequency: 12,
        q: 7,
        shape: 0,
    },
    band2: EQBand {
        gain: 68,
        frequency: 24,
        q: 7,
        shape: 0,
    },
    band3: EQBand {
        gain: 64,
        frequency: 34,
        q: 5,
        shape: 0,
    },
    band4: EQBand {
        gain: 66,
        frequency: 50,
        q: 7,
        shape: 0,
    },
    band5: EQBand {
        gain: 61,
        frequency: 52,
        q: 7,
        shape: 0,
    },
};
