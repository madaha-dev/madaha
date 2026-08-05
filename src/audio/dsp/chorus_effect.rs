/// XG Chorus family (Chorus1-4, Celeste1-4, Flanger1-3, Symphonic, Phaser)
///
/// Topology reverse-engineered from S-YXG2006LE (CSEF::CalcChorus @ 0x537dc):
///   - single 32768-sample delay ring (idx decrements, & 0x7fff)
///   - two read taps (L: idx+0x88+lfo, R: idx+0x9c+lfo) with linear interpolation,
///     two write taps (L: idx, R: idx+0xbc) → L delay = tap_l+lfo, R delay = lfo-32
///   - LFO: 23-bit phase accumulator + 32-entry XOR table (high 5 bits), ×freq1/freq2
///     → integer offset (>>22) + 22-bit fraction → L/R interp coefficients, every 4 samples
///   - feedback: ring[idx] = fb_l + L×p4f0 + fb_r×p500; ring[idx+0xbc] = R×p420 + fb_r
///     fb_l = L×p600 (wet L), fb_r = R×p690 (wet R)
/// Params (effect_obj::chorus_param index):
///   LFO_FREQ(1), LFO_PM_DEPTH(2), FEEDBACK_LEVEL(3), DELAY_OFFSET(4),
///   EQ_LOW_FREQ/GAIN(6/7), EQ_HIGH_FREQ/GAIN(8/9), DRY_WET(10),
///   EQ_MID_FREQ/GAIN/WIDTH(11/12/13), LFO_AM_DEPTH(14), INPUT_MODE(15)
use super::core::eq_chain::EqChain;
use super::params::{dry_wet, lfo_freq, p16};
use super::EffectProcessor;
use crate::midi::effect_params::effect_obj::chorus_param;

const RING_SIZE: usize = 32768;
const RING_MASK: usize = RING_SIZE - 1;
/// L/R read offset difference and R-write offset relative to the base tap (2006LE 0x88/0x9c/0xbc)
const TAP_R_DELTA: usize = 0x14;
const WRITE_R_DELTA: usize = 0xc;

/// DELAY_OFFSET param → seconds (XG 200ms delay table)
fn delay_sec(v: u16) -> f32 {
    crate::midi::effect_params::parameter_table::XG_DELAY_TIME_200MS_TABLE[v.min(127) as usize]
        / 1000.0
}

/// 32-entry pseudo-random XOR table (2006LE LFO: phase ^= table[phase>>22])
fn make_xor_table() -> [u32; 32] {
    let mut t = [0u32; 32];
    let mut x = 0x9e3779b9u32;
    for e in &mut t {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        *e = x >> 16;
    }
    t
}

pub struct ChorusEffect {
    /// Delay ring (f32[32768])
    ring: Box<[f32; RING_SIZE]>,
    /// Ring index (decrements each sample)
    idx: usize,
    /// Base delay (samples) from DELAY_OFFSET
    base_samples: f32,
    /// LFO state (23-bit)
    lfo_phase: u32,
    /// Third-LFO phase reference (3-tap mode, 2006LE 0x1ec)
    lfo_ref3: u32,
    /// LFO rate increment (23-bit per sample)
    lfo_inc: u32,
    /// Modulation amplitudes (2006LE 0x1f0/0x1f4/0x1f8)
    freq1: u32,
    freq2: u32,
    freq3: u32,
    xor_tbl: [u32; 32],
    /// LFO output: integer offset + L/R interp fractions
    lfo_int: i32,
    frac_l: f32,
    frac_r: f32,
    /// Third LFO (3-tap mode, 2006LE 0x1ec/0x1f8)
    lfo3_off: i32,
    frac3: f32,
    /// 3-tap mode (Chorus4/Celeste1-2: CalcChorus3)
    mode3: bool,
    /// Feedback / mix coefficients (2006LE p4f0/p420/p500/p600/p690)
    fb: f32,
    mix_l: f32,
    wet_l_g: f32,
    wet_r_g: f32,
    /// Feedback state (2006LE +0x400 buffer / +0x504 tail)
    fb_l: f32,
    fb_r: f32,
    dry: f32,
    wet: f32,
    /// Current LFO rate (modulatable)
    lfo_hz: f32,
    /// Base LFO rate (for modulation)
    base_lfo_hz: f32,
    /// AM depth
    am_depth: f32,
    /// 3-band EQ (LOW 6/7, HIGH 8/9, MID 11/12/13)
    eq: EqChain,
    sample_rate: f32,
    stereo_input: bool,
    /// Sample counter for the 4-sample LFO update rate
    lfo_tick: u32,
    /// Dyna envelope multiplier (XG2.0 DYNA FLANGER)
    dyna_env: f32,
}

impl ChorusEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            ring: Box::new([0.0; RING_SIZE]),
            idx: 0,
            base_samples: 100.0,
            lfo_phase: 0,
            lfo_ref3: 0x1a0000,
            lfo_inc: 0,
            freq1: 0,
            freq2: 0,
            freq3: 0,
            xor_tbl: make_xor_table(),
            lfo_int: 0,
            frac_l: 0.0,
            frac_r: 0.0,
            lfo3_off: 0,
            frac3: 0.0,
            mode3: false,
            fb: 0.0,
            mix_l: 1.0,
            wet_l_g: 1.0,
            wet_r_g: 1.0,
            fb_l: 0.0,
            fb_r: 0.0,
            dry: 1.0,
            wet: 1.0,
            lfo_hz: 1.0,
            base_lfo_hz: 1.0,
            am_depth: 0.0,
            eq: EqChain::new(),
            sample_rate,
            stereo_input: false,
            lfo_tick: 0,
            dyna_env: 1.0,
        }
    }

    pub fn set_dyna_env(&mut self, env: f32) {
        self.dyna_env = env;
    }

    fn update_lfo(&mut self) {
        let phase = (self.lfo_phase + self.lfo_inc) & 0x7fffff;
        self.lfo_phase = phase;
        let diff = phase.wrapping_sub(0) & 0x7fffff;
        let diff3 = phase.wrapping_sub(self.lfo_ref3) & 0x7fffff;
        let p = phase ^ self.xor_tbl[(phase >> 0x16) as usize];
        let d = diff ^ self.xor_tbl[(diff >> 0x16) as usize];
        let d3 = diff3 ^ self.xor_tbl[(diff3 >> 0x16) as usize];
        let acc1 = (p as u64).wrapping_mul(self.freq1 as u64);
        let acc2 = (d as u64).wrapping_mul(self.freq2 as u64);
        let acc3 = (d3 as u64).wrapping_mul(self.freq3 as u64);
        self.lfo_int = (acc1 >> 22) as i32;
        self.frac_l = ((acc1 & 0x3fffff) as f32) / 4194304.0;
        self.frac_r = ((acc2 & 0x3fffff) as f32) / 4194304.0;
        self.lfo3_off = (acc3 >> 22) as i32;
        self.frac3 = ((acc3 & 0x3fffff) as f32) / 4194304.0;
    }

    pub fn set_params(&mut self, params: &[u16; 16]) {
        self.base_lfo_hz = lfo_freq(p16(params, chorus_param::LFO_FREQ));
        self.lfo_hz = self.base_lfo_hz;
        // 23-bit phase increment
        self.lfo_inc =
            ((self.lfo_hz / self.sample_rate) * 8388608.0).max(1.0).min(8388607.0) as u32;

        // LFO_PM_DEPTH (0-127) → freq1/freq2: integer offset range ≈ ±pm×32 samples
        // (lfo_int = p×freq1>>22, p ≤ 2^23 → max offset = 2×freq1)
        let pm = p16(params, chorus_param::LFO_PM_DEPTH) as f32 / 127.0;
        let amp = (pm * 32.0).max(1.0);
        self.freq1 = amp as u32;
        self.freq2 = (self.freq1 as f64 * 0.9995) as u32; // L/R micro-detune
        self.freq3 = (self.freq1 as f64 * 1.0005) as u32; // third tap detune
        // DELAY_OFFSET (0-127) → base delay (XG delay table)
        self.base_samples = delay_sec(p16(params, chorus_param::DELAY_OFFSET)) * self.sample_rate;

        // XG Spec Table: Chorus feedback level (dedicated chorus table)
        self.fb = crate::midi::effect_params::parameter_table::XG_FEEDBACK_LEVEL_CHORUS
            [p16(params, chorus_param::FEEDBACK_LEVEL).min(127) as usize]
            .clamp(-0.99, 0.99);
        self.mix_l = 1.0;
        self.wet_l_g = 1.0;
        self.wet_r_g = 1.0;

        let (d, w) = dry_wet(p16(params, chorus_param::DRY_WET));
        self.dry = d;
        self.wet = w;

        self.am_depth = p16(params, chorus_param::LFO_AM_DEPTH) as f32 / 127.0;
        self.stereo_input = p16(params, chorus_param::INPUT_MODE) != 0;

        // 3-band EQ (LOW 6/7, HIGH 8/9, MID 11/12/13, XG_EQ_FREQ_TABLE)
        self.eq.set_chorus_layout(params, self.sample_rate);
    }

    /// Enable 3-tap mode (2006LE CalcChorus3: Chorus4/Celeste1-2/Phaser1)
    pub fn set_mode3(&mut self) {
        self.mode3 = true;
    }

    pub fn reset(&mut self) {
        self.ring.fill(0.0);
        self.idx = 0;
        self.fb_l = 0.0;
        self.fb_r = 0.0;
        self.lfo_phase = 0;
    }
}

impl EffectProcessor for ChorusEffect {
    fn modulate(&mut self, _source: u8, value: f32) {
        // Rate modulation: ±2 octaves at full depth
        self.lfo_hz = (self.base_lfo_hz * 4f32.powf(value)).clamp(0.05, 40.0);
        self.lfo_inc =
            ((self.lfo_hz / self.sample_rate) * 8388608.0).max(1.0).min(8388607.0) as u32;
    }

    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (mut l, mut r) = input;
        // Mono input: split evenly
        if !self.stereo_input {
            let mono = (l + r) * 0.5;
            l = mono;
            r = mono;
        }
        let l = self.eq.tick(l);
        let r = self.eq.tick(r);

        // LFO update every 4 samples (2006LE: & 0x3 == 3)
        self.lfo_tick = self.lfo_tick.wrapping_add(1);
        if self.lfo_tick & 3 == 3 {
            self.update_lfo();
        }

        // Read taps (2006LE): L: idx+base+lfo, R: idx+base-0x14+lfo, write R: idx+base+0xc
        let lf = (self.lfo_int as f32 * self.dyna_env) as i32;
        let base = self.base_samples as usize;
        let tap_l = base & RING_MASK;
        let tap_r = base.saturating_sub(TAP_R_DELTA) & RING_MASK;
        let write_r = (base + WRITE_R_DELTA) & RING_MASK;
        let pos_l = (self.idx as isize + tap_l as isize + lf as isize) & RING_MASK as isize;
        let pos_l = pos_l as usize;
        let v0 = self.ring[pos_l];
        let v1 = self.ring[(pos_l + 1) & RING_MASK];
        let l_interp = v0 + (v1 - v0) * self.frac_l;

        let pos_r = (self.idx as isize + tap_r as isize + lf as isize) & RING_MASK as isize;
        let pos_r = pos_r as usize;
        let w0 = self.ring[pos_r];
        let w1 = self.ring[(pos_r + 1) & RING_MASK];
        let r_interp = w0 + (w1 - w0) * self.frac_r;

        // Feedback writes (2006LE): input enters via the ring (SetupInput step)
        let wr = l + l_interp * self.fb;
        self.ring[self.idx] = wr;
        let wr2 = r + r_interp * self.mix_l;
        self.ring[(self.idx + write_r) & RING_MASK] = wr2;

        // Feedback states → wet outputs
        self.fb_l = l_interp * self.wet_l_g;
        self.fb_r = r_interp * self.wet_r_g;

        // 3-tap mode (CalcChorus3): extra tap pair 0xb0/0xc8 feeds the feedback
        if self.mode3 {
            let off3 = self.lfo3_off;
            let xa = (self.idx as isize + base as isize + 40 + off3 as isize) & RING_MASK as isize;
            let xa = xa as usize;
            let va = self.ring[xa] + (self.ring[(xa + 1) & RING_MASK] - self.ring[xa]) * self.frac3;
            let xb = (self.idx as isize + base as isize + 64 + off3 as isize) & RING_MASK as isize;
            let xb = xb as usize;
            let vb = self.ring[xb] + (self.ring[(xb + 1) & RING_MASK] - self.ring[xb]) * self.frac3;
            self.fb_l += va * 0.35;
            self.fb_r += vb * 0.35;
        }

        // Ring index decrements
        self.idx = (self.idx + RING_SIZE - 1) & RING_MASK;

        // AM (0-1)
        let am_l = 1.0 + self.frac_l * 2.0 * self.am_depth - self.am_depth;
        let am_r = 1.0 + self.frac_r * 2.0 * self.am_depth - self.am_depth;

        (l * self.dry + self.fb_l * self.wet * am_l, r * self.dry + self.fb_r * self.wet * am_r)
    }
}

/// Build XG Chorus effect
pub fn build_chorus(sample_rate: f32, params: &[u16; 16]) -> Box<dyn EffectProcessor> {
    let mut ch = ChorusEffect::new(sample_rate);
    ch.set_params(params);
    Box::new(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chorus_delays_and_wets() {
        let mut ch = ChorusEffect::new(44100.0);
        let mut params = [0u16; 16];
        params[chorus_param::DELAY_OFFSET] = 40; // ~40ms
        params[chorus_param::LFO_FREQ] = 10; // ~1Hz
        params[chorus_param::LFO_PM_DEPTH] = 64;
        params[chorus_param::DRY_WET] = 1; // full wet, test wet path only
        params[chorus_param::INPUT_MODE] = 1;
        ch.set_params(&params);

        // Impulse → wet output appears after the delay (0.05 > dry residue)
        let mut hit = false;
        let mut peak_idx = 0usize;
        for i in 0..44100 / 10 {
            let input = if i == 0 { (1.0, 1.0) } else { (0.0, 0.0) };
            let (l, _r) = ch.process(input);
            if l.abs() > 0.1 && !hit {
                hit = true;
                peak_idx = i;
            }
        }
        assert!(hit, "wet output did not appear");
        // Delay ~40ms = ~1764 samples (allowing LFO modulation range)
        assert!(peak_idx > 800 && peak_idx < 3000, "peak_idx={peak_idx}");
    }

    #[test]
    fn modulate_changes_rate() {
        let mut ch = ChorusEffect::new(44100.0);
        let mut params = [0u16; 16];
        params[chorus_param::LFO_FREQ] = 20;
        params[chorus_param::DRY_WET] = 64;
        ch.set_params(&params);
        let base = ch.lfo_hz;
        ch.modulate(0, 1.0); // full MW → 4x
        assert!((ch.lfo_hz - base * 4.0).abs() < 1e-3, "rate={} base={}", ch.lfo_hz, base);
        ch.modulate(0, -1.0); // 0.25x
        assert!((ch.lfo_hz - base * 0.25).abs() < 1e-3, "rate={}", ch.lfo_hz);
        ch.modulate(0, 0.0); // neutral → base
        assert!((ch.lfo_hz - base).abs() < 1e-3);
    }

    #[test]
    fn flanger_feedback_stable() {
        let mut ch = ChorusEffect::new(44100.0);
        let mut params = [0u16; 16];
        params[chorus_param::DELAY_OFFSET] = 1; // extremely short
        params[chorus_param::FEEDBACK_LEVEL] = 127; // max feedback
        params[chorus_param::DRY_WET] = 64; // passthrough
        ch.set_params(&params);
        // AC input: finite feedback loop (0.99 clamp prevents divergence)
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..44100 {
            let x = (phase * std::f32::consts::PI * 2.0).sin() * 0.5;
            phase = (phase + 440.0 / 44100.0) % 1.0;
            let (l, _r) = ch.process((x, x));
            peak = peak.max(l.abs());
        }
        assert!(peak < 20.0, "peak={peak}");
    }
}
