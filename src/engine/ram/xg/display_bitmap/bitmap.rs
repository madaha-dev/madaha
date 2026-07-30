/*
The data is related to the display screen as follows.
Each byte of data represents seven horizontal pixels.
Set a bit to 1 to turn on a pixe, and to 0 to turn off a pixel.
This data is arranged on the screen as follows.
      b6 b5 b4 b3 b2 b1 b0 b6 b5 b4 b3 b2 b1 b0 b6 b5 b4 b3 b2 b1 b0 (b stands for bit)
Data00 * * * * * * * Data16 * * * * * * * Data32 * * - - - - -
Data01               Data17               Data33
Data02               Data18               Data34
Data03               Data19               Data35
Data04               Data20               Data36
Data05               Data21               Data37
Data06               Data22               Data38
Data07               Data23               Data39
Data08               Data24               Data40
Data09               Data25               Data41
Data11               Data27               Data43
Data12               Data28               Data44
Data13               Data29               Data45
Data14               Data30               Data46
Data15               Data31               Data47
*/

use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use std::ops::{Index, IndexMut};

/// LCD display bitmap data (48 bytes, 7 pixels per byte, arranged 3 horizontal
/// segments × 16 vertical rows).
///
/// Each byte represents seven horizontal pixels (bit 6 is leftmost).
/// The 48 bytes form a 21×16 pixel display arranged as three 7×16 columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bitmap(
    /// Bitmap pixel data (48 bytes)
    [u8; 48],
);

impl Bitmap {
    pub fn new() -> Self {
        Self([0; 48])
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        let (addr, pixel) = transform(x, y);
        (self.0[addr] & (1 << pixel)) == 1
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, set: bool) {
        let (addr, pixel) = transform(x, y);
        if set {
            self.0[addr] |= 1 << pixel;
        } else {
            let mask = (!(1 << pixel)) & 0x7F;
            self.0[addr] &= mask;
        }
    }
}

impl Index<usize> for Bitmap {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Bitmap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Memory for Bitmap {
    fn reset(&mut self) {
        *self = Self::new();
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x2F) {
            return Err(err);
        }

        Ok(self.0[addr as usize])
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x2F) {
            return Err(err);
        }

        Ok(self.0[addr as usize] = value)
    }
}

fn transform(x: usize, y: usize) -> (usize, usize) {
    (((x & 0xF) / 7) * 16 + (y & 0xF), 6 - x % 7)
}
