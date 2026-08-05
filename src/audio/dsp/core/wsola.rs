/// WSOLA pitch shifter (shared by PitchChange and Harmony effects)
///
/// Waveform-Similarity Overlap-Add: the read pointer scans the delay line at
/// `read_rate` (2^(semitones/12)); each analysis interval, a search around the
/// target position finds the most similar waveform segment (SAD), then the new
/// segment is cross-faded with the previous one using a Hann window. This keeps
/// the phase continuous across segment boundaries — no clicks/metallic artifacts
/// compared to a fixed-period sawtooth cross-fade.
use super::delay::DelayLine;

pub struct WsolaShifter {
    line: DelayLine,
    /// Playback rate (1.0 = unity, >1 pitch up, <1 pitch down)
    read_rate: f32,
    write_pos: f32,
    /// Analysis interval (samples); ~20-50 ms. Shorter = less artifact but more fades
    period: f32,
    /// Half-width of the similarity search window (±period×0.25)
    search_half: f32,
    /// Read pointer (absolute input position, advances at `read_rate`)
    read_pos: f32,
    /// Minimum read delay (samples) before the read pointer wraps back
    min_delay: f32,
    /// Cross-fade progress (samples into the fade)
    fade: f32,
    fade_len: f32,
    /// Wrap target (input position where the next jump will land)
    wrap_target: f32,
    #[cfg(test)]
    wrap_count: u32,
}

impl WsolaShifter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            line: DelayLine::new((sample_rate as usize) * 4),
            read_rate: 1.0,
            write_pos: 0.0,
            period: sample_rate * 0.03, // 30 ms default
            search_half: sample_rate * 0.0075,
            read_pos: 0.0,
            min_delay: sample_rate * 0.03,
            fade: 0.0,
            fade_len: sample_rate * 0.005,
            wrap_target: 0.0,
            #[cfg(test)]
            wrap_count: 0,
        }
    }

    /// Set the shift in semitones (64 = ±12 range from the param)
    pub fn set_shift(&mut self, semitones: f32) {
        self.read_rate = 2f32.powf(semitones / 12.0);
    }

    /// Set the analysis interval (samples)
    pub fn set_period(&mut self, samples: f32) {
        self.period = samples.max(1.0);
        self.min_delay = samples;
        // Narrow search window: fine alignment only (keeps the rate advance,
        // avoids jumping to a wrong cycle on periodic signals)
        self.search_half = self.period * 0.05;
    }

    pub fn reset(&mut self) {
        self.line.reset();
        self.write_pos = 0.0;
        self.read_pos = 0.0;
        self.fade = 0.0;
        self.wrap_target = 0.0;
    }

    /// Sum of absolute differences between two delay-line regions.
    /// `a`/`b` are absolute read positions (write_pos - delay).
    fn sad(&self, a: f32, b: f32, len: f32) -> f32 {
        let n = len as usize;
        let mut acc = 0.0;
        for i in 0..n {
            let da = self.line.read((self.write_pos - a - i as f32).max(1.0));
            let db = self.line.read((self.write_pos - b - i as f32).max(1.0));
            acc += (da - db).abs();
        }
        acc
    }

    /// Search for the most similar segment start around `target` (absolute position),
    /// comparing against the pre-wrap waveform (just before the jump) for continuity
    fn align(&mut self, target: f32) -> f32 {
        let half = self.search_half as usize;
        let mut best = target;
        let mut best_sad = f32::MAX;
        // Reference: the last period×0.5 samples before the jump (at wrap_target)
        let ref_pos = self.wrap_target - self.period * 0.5;
        for i in 0..(half * 2 + 1) {
            let cand = target - self.search_half + i as f32;
            if cand < 1.0 {
                continue;
            }
            let d = self.sad(ref_pos, cand, self.period * 0.5);
            if d < best_sad {
                best_sad = d;
                best = cand;
            }
        }
        best
    }

    /// Process one sample: write `x`, return the shifted output.
    ///
    /// The read pointer scans the delay line at `read_rate` (pitch shift);
    /// when it approaches the write pointer (delay < min_delay) it jumps back
    /// by one period, aligned to the most similar waveform for phase continuity.
    pub fn process_sample(&mut self, x: f32) -> f32 {
        self.line.push(x);
        self.write_pos += 1.0;
        self.read_pos += self.read_rate;

        let delay = self.write_pos - self.read_pos;
        if delay < self.min_delay && self.write_pos > self.period * self.read_rate + 1.0 {
            self.wrap_target = self.read_pos;
            self.read_pos = self.align(self.read_pos - self.period);
            self.fade = 0.0;
            #[cfg(test)]
            {
                self.wrap_count += 1;
            }
        }

        let y_new = self.line.read((self.write_pos - self.read_pos).max(1.0));
        if self.fade < self.fade_len {
            self.fade += 1.0;
            let t = (self.fade / self.fade_len).clamp(0.0, 1.0);
            let hann = 0.5 - 0.5 * (t * std::f32::consts::PI * 2.0).cos();
            // Blend with the pre-wrap position to hide the jump
            let y_old = self.line.read((self.write_pos - self.wrap_target).max(1.0));
            y_old * (1.0 - hann) + y_new * hann
        } else {
            y_new
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_shift_passes_signal() {
        let mut s = WsolaShifter::new(44100.0);
        s.set_shift(0.0);
        s.set_period(1323.0);
        let mut peak = 0.0f32;
        for i in 0..44100 {
            let x = (i as f32 / 44100.0 * 440.0 * std::f32::consts::TAU).sin();
            let y = s.process_sample(x);
            if i > 20000 {
                peak = peak.max(y.abs());
            }
        }
        assert!(peak > 0.7, "unity shift attenuated: {peak}");
    }

    #[test]
    fn shift_up_increases_frequency() {
        let mut s = WsolaShifter::new(44100.0);
        s.set_shift(12.0); // +1 octave
        s.set_period(1323.0);
        // Count zero crossings of a 440 Hz input after shifting
        let mut crossings = 0usize;
        let mut prev = 0.0f32;
        let n = 44100 / 4;
        for i in 0..n {
            let x = (i as f32 / 44100.0 * 440.0 * std::f32::consts::TAU).sin();
            let y = s.process_sample(x);
            if i > 1000 && prev <= 0.0 && y > 0.0 {
                crossings += 1;
            }
            prev = y;
        }
        // 880 Hz over n samples ≈ 2 × crossings of 440 Hz
        let freq = crossings as f32 / (n as f32 / 44100.0);
        assert!(freq > 700.0 && freq < 1100.0, "shifted freq={freq}");
    }
}
