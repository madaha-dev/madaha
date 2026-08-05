/// Reverb kernel: multiple comb + allpass (Schroeder/Freeverb style)
///
/// Structure: input → 4×comb (parallel) → 2×allpass (series)
/// - comb: y[n] = x[n] + feedback × y[n-D], in-loop first-order LPF damping
/// - allpass: y[n] = -g·x[n] + x[n-D] + g·y[n-D]
///
/// Params:
/// - `decay`: comb feedback (0-1, larger = longer)
/// - `diffusion`: allpass g (0-1)
/// - `damp`: in-loop damping (0-1, 1=no filtering, small=high damping)
use super::delay::DelayLine;

#[derive(Debug)]
pub struct ReverbKernel {
    combs: [Comb; 4],
    allpasses: [Allpass; 2],
}

#[derive(Debug)]
struct Comb {
    line: DelayLine,
    feedback: f32,
    damp: f32,
    last: f32,
}

#[derive(Debug)]
struct Allpass {
    line: DelayLine,
    g: f32,
}

impl ReverbKernel {
    /// Comb delays (samples) staggered by golden ratio to avoid comb coloration
    const COMB_DELAYS: [usize; 4] = [1116, 1188, 1277, 1356];
    const ALLPASS_DELAYS: [usize; 2] = [556, 441];

    pub fn new(sample_rate: f32) -> Self {
        // Delays scaled by sample rate (base 44100)
        let scale = (sample_rate / 44100.0).max(0.5);
        let comb_delays = Self::COMB_DELAYS.map(|d| (d as f32 * scale) as usize);
        let ap_delays = Self::ALLPASS_DELAYS.map(|d| (d as f32 * scale) as usize);
        Self {
            combs: comb_delays.map(|d| Comb {
                line: DelayLine::new(d + 1),
                feedback: 0.84,
                damp: 0.5,
                last: 0.0,
            }),
            allpasses: ap_delays.map(|d| Allpass {
                line: DelayLine::new(d + 1),
                g: 0.5,
            }),
        }
    }

    /// Set reverb params
    /// `time_sec`: reverb time (RT60)
    /// `diffusion`: 0-1
    /// `damp`: 0-1 (1=bright)
    pub fn set_params(&mut self, time_sec: f32, diffusion: f32, damp: f32) {
        // RT60: 60dB decay time → feedback coefficient = 10^(-3 × D / (time × sr))
        let t = time_sec.clamp(0.05, 60.0);
        for comb in self.combs.iter_mut() {
            let d = comb.line_buf_len() as f32 / 44100.0; // seconds (approx base)
            let g = 10f32.powf(-3.0 * d / t);
            comb.feedback = g.clamp(0.0, 0.99);
            comb.damp = damp.clamp(0.0, 1.0);
        }
        for ap in self.allpasses.iter_mut() {
            ap.g = diffusion.clamp(0.0, 0.9);
        }
    }

    /// Process one mono sample
    pub fn tick(&mut self, input: f32) -> f32 {
        let mut out = 0.0f32;
        for comb in self.combs.iter_mut() {
            out += comb.tick(input);
        }
        out *= 0.25;
        for ap in self.allpasses.iter_mut() {
            out = ap.tick(out);
        }
        out
    }

    pub fn reset(&mut self) {
        for c in self.combs.iter_mut() {
            c.reset();
        }
        for a in self.allpasses.iter_mut() {
            a.reset();
        }
    }
}

impl Comb {
    fn tick(&mut self, input: f32) -> f32 {
        let delayed = self.line.tick(input + self.last * self.feedback, self.delay() as f32);
        // In-loop first-order LPF damping
        self.last = delayed * (1.0 - self.damp) + self.last * self.damp;
        delayed
    }
    fn delay(&self) -> usize {
        self.line.buf_len() - 1
    }
    fn reset(&mut self) {
        self.line.reset();
        self.last = 0.0;
    }
    fn line_buf_len(&self) -> usize {
        self.line.buf_len()
    }
}

impl Allpass {
    fn tick(&mut self, input: f32) -> f32 {
        let delayed = self.line.read(self.delay() as f32);
        self.line.push(input + delayed * self.g);
        -self.g * input + delayed
    }
    fn delay(&self) -> usize {
        self.line.buf_len() - 1
    }
    fn reset(&mut self) {
        self.line.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impulse_decays_exponentially() {
        let mut r = ReverbKernel::new(44100.0);
        r.set_params(1.5, 0.5, 0.5);
        let mut peak_tail: f32 = 0.0;
        let mut early_peak = 0.0f32;
        for i in 0..88200 {
            let input = if i == 0 { 1.0 } else { 0.0 };
            let out = r.tick(input);
            if i < 5000 {
                early_peak = early_peak.max(out.abs());
            }
            if i > 44100 {
                peak_tail = peak_tail.max(out.abs());
            }
        }
        // Tail clearly decayed (< 1/50 of peak)
        assert!(early_peak > 0.05, "early_peak={early_peak}");
        assert!(
            peak_tail < early_peak * 0.02,
            "tail={peak_tail} early={early_peak}"
        );
    }

    #[test]
    fn longer_time_has_longer_tail() {
        let mut short = ReverbKernel::new(44100.0);
        short.set_params(0.5, 0.5, 0.5);
        let mut long = ReverbKernel::new(44100.0);
        long.set_params(5.0, 0.5, 0.5);

        // Tick from 0 (incl. impulse), accumulate target window only
        let energy_after = |r: &mut ReverbKernel, start: usize, end: usize| -> f32 {
            let mut e = 0.0;
            for i in 0..end {
                let input = if i == 0 { 1.0 } else { 0.0 };
                let out = r.tick(input);
                if i >= start {
                    e += out * out;
                }
            }
            e
        };
        let s = energy_after(&mut short, 22050, 44100);
        let l = energy_after(&mut long, 22050, 44100);
        assert!(l > s, "long={l} short={s}");
    }
}
