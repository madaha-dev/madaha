/// Delay line (ring buffer, supports fractional-delay linear interpolation + LFO modulation)
#[derive(Debug)]
pub struct DelayLine {
    buf: Vec<f32>,
    /// Write pointer (integer sample position)
    write_pos: usize,
}

impl DelayLine {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            buf: vec![0.0; max_delay_samples.max(1)],
            write_pos: 0,
        }
    }

    /// Reset buffer
    pub fn reset(&mut self) {
        self.buf.fill(0.0);
        self.write_pos = 0;
    }

    /// Write one sample and return the read value at delay `delay_samples` (f32, may be fractional)
    #[inline]
    pub fn tick(&mut self, input: f32, delay_samples: f32) -> f32 {
        let len = self.buf.len() as f32;
        let d = delay_samples.clamp(1.0, len - 1.0);
        let read_pos = self.write_pos as f32 - d;
        let read_pos = if read_pos < 0.0 { read_pos + len } else { read_pos };
        let i0 = read_pos.floor() as usize % self.buf.len();
        let i1 = (i0 + 1) % self.buf.len();
        let frac = read_pos - read_pos.floor();

        let delayed = self.buf[i0] * (1.0 - frac) + self.buf[i1] * frac;

        self.buf[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buf.len();
        delayed
    }

    /// Buffer length (samples)
    pub fn buf_len(&self) -> usize {
        self.buf.len()
    }

    /// Write only (no output), for multi-read-point delay structures
    #[inline]
    pub fn push(&mut self, input: f32) {
        self.buf[self.write_pos] = input;
        self.write_pos = (self.write_pos + 1) % self.buf.len();
    }

    /// Read any delay amount (does not advance write pointer)
    #[inline]
    pub fn read(&self, delay_samples: f32) -> f32 {
        let len = self.buf.len() as f32;
        let d = delay_samples.clamp(1.0, len - 1.0);
        let read_pos = self.write_pos as f32 - d;
        let read_pos = if read_pos < 0.0 { read_pos + len } else { read_pos };
        let i0 = read_pos.floor() as usize % self.buf.len();
        let i1 = (i0 + 1) % self.buf.len();
        let frac = read_pos - read_pos.floor();
        self.buf[i0] * (1.0 - frac) + self.buf[i1] * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_delay() {
        let mut dl = DelayLine::new(16);
        for i in 0..16 {
            let out = dl.tick(i as f32, 4.0);
            if i < 4 {
                assert_eq!(out, 0.0);
            } else {
                assert!((out - (i - 4) as f32).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn fractional_delay_interpolates() {
        let mut dl = DelayLine::new(16);
        for i in 0..16 {
            dl.tick(i as f32, 1.0);
        }
        // Write 0..15, last write 15 → read 1.5 samples back = (14+15)/2
        let out = dl.read(1.5);
        assert!((out - 14.5).abs() < 1e-4, "out={out}");
    }

    #[test]
    fn feedback_loop_stable() {
        let mut dl = DelayLine::new(64);
        let mut y = 0.0f32;
        for _ in 0..1000 {
            y = dl.tick(y * 0.5 + 0.01, 8.0);
        }
        // Feedback 0.5 converges
        assert!(y.abs() < 0.5);
    }
}
