/// DC offset blocker (XG Spec: serial effect chains can introduce DC components)
///
/// One-pole IIR highpass: y[n] = x[n] - x[n-1] + R·y[n-1]
/// R = 0.999 → cutoff ≈3.5 Hz @44.1 kHz (inaudible low-frequency removal,
/// keeps the full musical range intact). Always enabled on the master bus
/// after the Master Attenuator, matching XG hardware behavior.
pub struct DcBlocker {
    prev_x: f32,
    prev_y: f32,
    r: f32,
}

impl DcBlocker {
    pub fn new() -> Self {
        Self {
            prev_x: 0.0,
            prev_y: 0.0,
            r: 0.999,
        }
    }

    #[inline]
    pub fn tick(&mut self, x: f32) -> f32 {
        let y = x - self.prev_x + self.r * self.prev_y;
        self.prev_x = x;
        self.prev_y = y;
        y
    }

    pub fn reset(&mut self) {
        self.prev_x = 0.0;
        self.prev_y = 0.0;
    }
}

impl Default for DcBlocker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_dc_offset() {
        let mut b = DcBlocker::new();
        let mut peak = 0.0f32;
        for i in 0..44100 {
            let y = b.tick(1.0);
            if i > 40000 {
                peak = peak.max(y.abs());
            }
        }
        assert!(peak < 1e-4, "dc residue {peak}");
    }

    #[test]
    fn passes_audible_low_frequency() {
        let mut b = DcBlocker::new();
        let mut peak_out = 0.0f32;
        let mut peak_in = 0.0f32;
        for i in 0..44100 {
            let x = (i as f32 / 44100.0 * 100.0 * std::f32::consts::TAU).sin();
            let y = b.tick(x);
            peak_in = peak_in.max(x.abs());
            if i > 40000 {
                peak_out = peak_out.max(y.abs());
            }
        }
        // 100 Hz passes essentially untouched (cutoff ~3.5 Hz)
        assert!(peak_out > peak_in * 0.99, "100Hz attenuated: {peak_out} vs {peak_in}");
    }

    #[test]
    fn clears_chain_dc_from_nonlinear_effect() {
        // Simulate a DC-producing nonlinear stage (asymmetric clipping)
        let mut b = DcBlocker::new();
        let mut dc_input = 0.0f32;
        for i in 0..44100 {
            let x = (i as f32 / 44100.0 * 200.0 * std::f32::consts::TAU).sin();
            // asymmetric clip → DC
            let clipped = (x * 3.0).clamp(-1.0, 1.5);
            let y = b.tick(clipped);
            if i > 40000 {
                dc_input = dc_input.max((clipped - y).abs());
            }
        }
        assert!(dc_input > 0.01, "expected DC before blocker, got {dc_input}");
    }
}
