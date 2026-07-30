use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use crate::engine::{errors::MidiError, ram::xg::display_bitmap::bitmap::Bitmap};
use std::ops::{Index, IndexMut};

/// Full LCD display bitmap (8 vertical × 16 horizontal segments, each a `Bitmap`).
///
/// The display is organized as 8 rows of 16 horizontal segments, each segment
/// being a 21×16 pixel block, for a total of 336×128 pixel display area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayBitmap(
    /// Display bitmap segments (8 × 16 grid)
    [[Bitmap; 16]; 8],
);

impl DisplayBitmap {
    pub fn new() -> Self {
        Self([[Bitmap::new(); 16]; 8])
    }
}

impl Index<usize> for DisplayBitmap {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        let mid = (index >> 8) & 0x7F;
        let v = (mid >> 4) & 0x7;
        let h = mid & 0xF;
        let lo = index & 0xFF;

        &self.0[v][h][lo]
    }
}

impl IndexMut<usize> for DisplayBitmap {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let mid = (index >> 8) & 0x7F;
        let v = (mid >> 4) & 0x7;
        let h = mid & 0xF;
        let lo = index & 0xFF;

        &mut self.0[v][h][lo]
    }
}

impl Memory for DisplayBitmap {
    fn reset(&mut self) {
        *self = Self::new();
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let mid = addr[1] as usize;
        let v = (mid >> 4) & 0x7;
        let h = mid & 0xF;

        self.0[v][h].get(addr)
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let mid = addr[1] as usize;
        let v = (mid >> 4) & 0x7;
        let h = mid & 0xF;

        self.0[v][h].set(addr, value)
    }
}
