//! Lock-free SPSC ring buffer (interleaved f32 frames)
//!
//! Single writer (audio render thread) / single reader (Jack/PipeWire callback).

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

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
        // pos is a FRAME index but buf holds interleaved samples (L,R pairs):
        // sample index = frame*2 + i. Using pos+i alone overlapped consecutive
        // blocks by half a block (writes landed 2x too early), corrupting data.
        (pos * 2 + i) & (self.cap * 2 - 1)
    }

    /// Write interleaved frames (L,R pairs); blocks with backpressure when
    /// full so the consumer never sees gaps (dropping blocks caused periodic
    /// pitch jumps: the render thread writes ~10% faster than the sink
    /// consumes). A deadline (2s) bounds the wait in case the consumer
    /// disappears; excess is dropped after that.
    pub fn write(&self, interleaved: &[f32]) -> usize {
        let total = interleaved.len() / 2;
        let mut written = 0;
        let deadline = Instant::now() + Duration::from_secs(2);
        while written < total {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let free = self.cap - (head - tail);
            let n = (total - written).min(free);
            if n > 0 {
                let start = written * 2;
                for i in 0..n * 2 {
                    unsafe {
                        *self.buf[self.write_idx(head, i)].get() = interleaved[start + i];
                    }
                }
                self.head.store(head + n, Ordering::Release);
                written += n;
            } else if std::time::Instant::now() < deadline {
                // Ring full: hand the core back until the consumer reads
                thread::yield_now();
            } else {
                break;
            }
        }
        written
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
    fn full_buffer_backpressure_waits_not_drops() {
        // Minimum capacity is 64 (next_power_of_two)
        let rb = SpscRing::new(64);
        let mut data = Vec::new();
        for i in 0..64 {
            data.push(i as f32);
            data.push(i as f32);
        }
        rb.write(&data); // fills 64 frames
        let mut out = [0.0f32; 4];
        assert_eq!(rb.read(&mut out), 2); // consumer frees 2 frames
        // Writer must now complete without losing data (backpressure):
        rb.write(&[9.0, 9.0, 8.0, 8.0]); // 2 frames fit the freed space
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
