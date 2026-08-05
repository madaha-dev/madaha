//! Lock-free SPSC ring buffer (interleaved f32 frames)
//!
//! Single writer (audio render thread) / single reader (Jack/PipeWire callback).

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// SPSC ring: single writer + single reader share the buffer via UnsafeCell.
/// Safety: write() is only called by the render thread, read() only by the
/// callback thread (guaranteed by the Jack/PipeWire ownership model).
unsafe impl Sync for SpscRing {}

pub struct SpscRing {
    buf: Vec<UnsafeCell<f32>>,
    /// Write position (in frames, monotonically increasing, never wraps)
    head: AtomicUsize,
    /// Read position (in frames, monotonically increasing, never wraps)
    tail: AtomicUsize,
    /// Capacity in frames (power of two)
    cap: usize,
}

impl SpscRing {
    pub fn new(frames: usize) -> Self {
        let cap = frames.next_power_of_two().max(64);
        Self {
            buf: (0..cap * 2).map(|_| UnsafeCell::new(0.0)).collect(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            cap,
        }
    }

    fn write_idx(&self, pos: usize, i: usize) -> usize {
        (pos + i) & (self.cap * 2 - 1)
    }

    /// Write interleaved frames (L,R pairs); drops the excess when full
    pub fn write(&self, interleaved: &[f32]) {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let used = head - tail;
        let free = self.cap - used;
        let frames = interleaved.len() / 2;
        let n = frames.min(free);
        for i in 0..n * 2 {
            unsafe {
                *self.buf[self.write_idx(head, i)].get() = interleaved[i];
            }
        }
        self.head.store(head + n, Ordering::Release);
    }

    /// Read up to `frames` frames into `out` (interleaved); returns frames read
    pub fn read(&self, out: &mut [f32]) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let used = head - tail;
        let want = out.len() / 2;
        let n = want.min(used);
        for i in 0..n * 2 {
            unsafe {
                out[i] = *self.buf[self.write_idx(tail, i)].get();
            }
        }
        self.tail.store(tail + n, Ordering::Release);
        n
    }

    /// Available frames to read
    #[allow(dead_code)]
    pub fn read_space(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head - tail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_roundtrip() {
        let rb = SpscRing::new(8);
        rb.write(&[1.0, 2.0, 3.0, 4.0]); // 2 frames
        assert_eq!(rb.read_space(), 2);
        let mut out = [0.0f32; 4];
        assert_eq!(rb.read(&mut out), 2);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn full_buffer_drops() {
        // Minimum capacity is 64 (next_power_of_two)
        let rb = SpscRing::new(64);
        let mut data = Vec::new();
        for i in 0..64 {
            data.push(i as f32);
            data.push(i as f32);
        }
        rb.write(&data); // fills 64 frames
        rb.write(&[9.0, 9.0]); // full → dropped
        assert_eq!(rb.read_space(), 64);
    }

    #[test]
    fn wrap_around() {
        let rb = SpscRing::new(4);
        rb.write(&[1.0, 1.0, 2.0, 2.0]);
        let mut out = [0.0f32; 4];
        rb.read(&mut out);
        rb.write(&[3.0, 3.0, 4.0, 4.0]); // wraps
        let mut out2 = [0.0f32; 4];
        rb.read(&mut out2);
        assert_eq!(out2, [3.0, 3.0, 4.0, 4.0]);
    }
}
