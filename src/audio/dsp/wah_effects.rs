/// Filter effects: AutoWah / TouchWah
///
/// Band-pass filter sweep:
/// - AutoWah: LFO sweep
/// - TouchWah: input envelope sweep (SENSITIVITY)
/// - DRIVE: clipping drive
use crate::fast_sine::fast_sin;
use crate::midi::effect_params::effect_obj::{auto_wah_param, touch_wah_param};

use super::core::biquad::{Biquad, make_biquad};
use super::core::eq_chain::EqChain;
use super::params::{dry_wet, lfo_freq, p16};
use super::EffectProcessor;

/// Resonant band-pass (state variable approx: band-pass via biquad peaking high Q)
struct BandPass {
    biquad: Biquad,
    freq: f32,
    q: f32,
    sample_rate: f32,
}

impl BandPass {
    fn new(sample_rate: f32) -> Self {
        Self { biquad: Biquad::new(), freq: 1000.0, q: 1.0, sample_rate }
    }

    /// Set center frequency (Hz), internally a peaking biquad + high Q approximates band-pass
    fn set_freq(&mut self, freq: f32) {
        self.freq = freq.clamp(40.0, 16000.0);
        // peaking 0dB, high Q → narrow band
        self.biquad = make_biquad(0.0, self.freq, self.q.max(0.5), true, self.sample_rate);
    }

    fn tick(&mut self, x: f32) -> f32 {
        self.biquad.tick(x)
    }
}

/// wah common params
struct WahParams {
    cutoff_offset: f32,
    resonance: f32,
    drive: f32,
    dry: f32,
    wet: f32,
    /// EQ (LOW 6/7, HIGH 8/9)
    eq: EqChain,
}

fn wah_params(params: &[u16; 16], sample_rate: f32) -> WahParams {
    let (d, w) = dry_wet(p16(params, auto_wah_param::DRY_WET));
    let mut p = WahParams {
        cutoff_offset: p16(params, auto_wah_param::CUTOFF_FREQ_OFFSET) as f32 / 127.0,
        resonance: 1.0 + p16(params, auto_wah_param::RESONANCE) as f32 / 127.0 * 15.0,
        drive: 1.0 + p16(params, auto_wah_param::DRIVE) as f32 / 127.0 * 6.0,
        dry: d,
        wet: w,
        eq: EqChain::new(),
    };
    // EQ: LOW(6/7), HIGH(8/9) (no MID band param → 64)
    p.eq.set_bands(
        p16(params, 6),
        p16(params, 7),
        0,
        64,
        64,
        p16(params, 8),
        p16(params, 9),
        sample_rate,
    );
    p
}

/// wah sweep range (Hz)
const WAH_MIN: f32 = 400.0;
const WAH_MAX: f32 = 6000.0;

pub struct AutoWahEffect {
    lfo_phase: f32,
    lfo_freq: f32,
    lfo_depth: f32,
    params: WahParams,
    bp_l: BandPass,
    bp_r: BandPass,
    sample_rate: f32,
}

impl AutoWahEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            lfo_phase: 0.0,
            lfo_freq: 1.0,
            lfo_depth: 0.5,
            params: WahParams {
                cutoff_offset: 0.5,
                resonance: 5.0,
                drive: 1.0,
                dry: 0.0,
                wet: 1.0,
                eq: EqChain::new(),
            },
            bp_l: BandPass::new(sample_rate),
            bp_r: BandPass::new(sample_rate),
            sample_rate,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.lfo_freq = lfo_freq(p16(params, auto_wah_param::LFO_FREQ));
        self.lfo_depth = p16(params, auto_wah_param::LFO_DEPTH) as f32 / 127.0;
        self.params = wah_params(params, sample_rate);
        self.bp_l.q = self.params.resonance;
        self.bp_r.q = self.params.resonance;
    }
}

impl EffectProcessor for AutoWahEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        self.lfo_phase += self.lfo_freq / self.sample_rate;
        if self.lfo_phase >= 1.0 {
            self.lfo_phase -= 1.0;
        }
        // Sweep: offset center + LFO×depth swing
        let lfo = fast_sin(self.lfo_phase * std::f32::consts::PI * 2.0) * 0.5 + 0.5;
        let t = (self.params.cutoff_offset + (lfo - 0.5) * self.lfo_depth * 2.0).clamp(0.0, 1.0);
        let freq = WAH_MIN * (WAH_MAX / WAH_MIN).powf(t);
        self.bp_l.set_freq(freq);
        self.bp_r.set_freq(freq);
        let l = self.params.eq.tick(l);
        let r = self.params.eq.tick(r);
        let wl = self.bp_l.tick(l * self.params.drive);
        let wr = self.bp_r.tick(r * self.params.drive);
        (l * self.params.dry + wl * self.params.wet, r * self.params.dry + wr * self.params.wet)
    }
}

// ──────────────────────── Touch Wah ────────────────────────
pub struct TouchWahEffect {
    /// Input envelope (0-1, smoothed)
    envelope: f32,
    /// Envelope release smoothing (XG Spec Table #12, ms → samples)
    release_samples: f32,
    sensitivity: f32,
    params: WahParams,
    bp_l: BandPass,
    bp_r: BandPass,
}

impl TouchWahEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            envelope: 0.0,
            release_samples: 20.0,
            sensitivity: 0.5,
            params: WahParams {
                cutoff_offset: 0.5,
                resonance: 5.0,
                drive: 1.0,
                dry: 0.0,
                wet: 1.0,
                eq: EqChain::new(),
            },
            bp_l: BandPass::new(sample_rate),
            bp_r: BandPass::new(sample_rate),
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.sensitivity = p16(params, touch_wah_param::SENSITIVITY) as f32 / 127.0;
        self.params = wah_params(params, sample_rate);
        self.bp_l.q = self.params.resonance;
        self.bp_r.q = self.params.resonance;
        // Envelope release time (XG Spec Table #12, ms → samples)
        let rel_ms = crate::midi::effect_params::parameter_table::XG_WAH_RELEASE_TIME_TABLE
            [(p16(params, touch_wah_param::SENSITIVITY) as usize * 15 / 127).min(15)];
        self.release_samples = rel_ms / 1000.0 * sample_rate;
    }
}

impl EffectProcessor for TouchWahEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // Envelope detection (RMS approx + smoothing)
        let mag = ((l * l + r * r) * 0.5).sqrt();
        let target = (mag * self.sensitivity * 4.0).clamp(0.0, 1.0);
        // Attack fast (1 ms), release per XG Table #12
        let coeff = if target > self.envelope {
            0.05
        } else {
            1.0 - (-1.0 / self.release_samples.max(1.0)).exp()
        };
        self.envelope += (target - self.envelope) * coeff;
        // Sweep: offset + envelope
        let t = (self.params.cutoff_offset + self.envelope * self.sensitivity).clamp(0.0, 1.0);
        let freq = WAH_MIN * (WAH_MAX / WAH_MIN).powf(t);
        self.bp_l.set_freq(freq);
        self.bp_r.set_freq(freq);
        let l = self.params.eq.tick(l);
        let r = self.params.eq.tick(r);
        let wl = self.bp_l.tick(l * self.params.drive);
        let wr = self.bp_r.tick(r * self.params.drive);
        (l * self.params.dry + wl * self.params.wet, r * self.params.dry + wr * self.params.wet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_wah_passes_signal() {
        let mut e = AutoWahEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[auto_wah_param::LFO_FREQ] = 10;
        p[auto_wah_param::LFO_DEPTH] = 64;
        p[auto_wah_param::CUTOFF_FREQ_OFFSET] = 64;
        p[auto_wah_param::DRY_WET] = 1; // full wet
        e.set_params(&p, 44100.0);
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let x = (phase * std::f32::consts::PI * 2.0).sin();
            phase = (phase + 1000.0 / 44100.0) % 1.0;
            let (l, _) = e.process((x, x));
            peak = peak.max(l.abs());
        }
        // Band-pass sweep: peak signal present and shaped
        assert!(peak > 0.05, "peak={peak}");
    }
}
