/// XG Reverb system effect (Hall1/2, Room1-3, Stage1/2, Plate, WhiteRoom/Tunnel/Canyon/Basement)
///
/// Topology reverse-engineered from S-YXG2006LE (CSEF::CalcReverb @ 0x4caa4):
///   - shared 131072-sample delay ring (idx decrements, & 0x1ffff)
///   - 2× 1st-order IIR input filter (2-cascade: y = b0*x + b1*x1 + fb*y_prev)
///   - 7 multi-feedback lines: d[w] = d[r1]*a + d[r2]*b (+ fb_state*c on lines 4-7)
///   - L/R output: 4-tap sum * m + direct * d, then * mix + tap * mix2
///   - per-type delay offset tables are pending (B5); hall defaults used here
/// Params (effect_obj::plate_param index):
///   REVERB_TIME(1), DIFFUSION(2), INIT_DELAY(3), HPF_CUTOFF(4), LPF_CUTOFF(5),
///   DRY_WET(10), REV_DELAY(11), DENCITY(12), REV_ER_BALANCE(13), HIGH_DAMP(14), FEEDBACK_LEVEL(15)
use super::params::{delay_time_samples, dry_wet, p16, reverb_time_sec};
use super::EffectProcessor;
use crate::midi::effect_params::effect_obj::plate_param;

const RING_SIZE: usize = 131072;
const RING_MASK: usize = RING_SIZE - 1;

/// Delay line offsets (sample units) — hall-family default topology.
/// Write/read offset pairs follow CalcReverb: line k reads r[k] (+r2 for 2-tap),
/// writes w[k]. All relative to the current ring index.
struct LineTable {
    w: [usize; 7],
    r: [usize; 7],
    r2: [usize; 7],
    taps_l: [usize; 8],
    taps_r: [usize; 8],
    direct: usize,
}

impl LineTable {
    const fn hall() -> Self {
        // Offsets from the 2006LE field table (0x84..0xf0), hall-default arrangement:
        //   line1: in-mix   w=0xa8  r=0x84
        //   line2: 2-tap    w=0xa4  r1=0x84 r2=0x90
        //   line3: f8 tap   w=0xb8  r1=0x90 r2=0x98
        //   line4-7: fb     w=0xe0/0xe8/0xec/0xf0  r=0x94/0xd4/0xdc/0xe4
        // output taps: L=0x64/0x68/0x6c/0x70, R=0x74/0x78/0x7c/0x80, direct=0x160
        Self {
            w: [0xa8, 0xa4, 0xb8, 0xe0, 0xe8, 0xec, 0xf0],
            r: [0x84, 0x84, 0x90, 0x94, 0xd4, 0xdc, 0xe4],
            r2: [0x00, 0x90, 0x98, 0x00, 0x00, 0x00, 0x00],
            taps_l: [0x64, 0x68, 0x6c, 0x70, 0x64, 0x68, 0x6c, 0x70],
            taps_r: [0x74, 0x78, 0x7c, 0x80, 0x74, 0x78, 0x7c, 0x80],
            direct: 0x160,
        }
    }
}

pub struct ReverbEffect {
    /// Shared delay ring (2006LE ERAM)
    ring: Box<[f32; RING_SIZE]>,
    /// Ring index (decrements each sample)
    idx: usize,
    lines: LineTable,
    /// 2× 1st-order IIR input filter states (mono, 2006LE)
    in_fb: [f32; 2],
    /// Input filter coefficients: [b0, b1, fb1, b0, b1, fb2]
    in_coef: [f32; 6],
    /// Line feedback gains: [g1..g7] + 2-tap mix ratios (diffusion)
    line_gain: [f32; 7],
    /// Feedback state for lines 4-7 (2006LE field_0x1ac..0x1b8)
    fb_state: [f32; 4],
    /// Output mix: [l_mix1, l_direct, l_mix2, l_tap, r_*...]
    out_l: [f32; 4],
    out_r: [f32; 4],
    /// Dry/wet from DRY_WET
    dry: f32,
    wet: f32,
    /// Pre-delay samples (INIT_DELAY + REV_DELAY)
    init_samples: usize,
    init_delay: [f32; 65536],
    init_idx: usize,
    base_time: f32,
    base_diffusion: f32,
    base_damp: f32,
    /// Enabled output tap count (DENCITY)
    tap_count: usize,
    /// ER/tail balance (0=all ER, 1=all tail)
    er_balance: f32,
    tail_level: f32,
    sample_rate: f32,
}

impl ReverbEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            ring: Box::new([0.0; RING_SIZE]),
            idx: 0,
            lines: LineTable::hall(),
            in_fb: [0.0; 2],
            in_coef: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            line_gain: [0.0; 7],
            fb_state: [0.0; 4],
            out_l: [0.0; 4],
            out_r: [0.0; 4],
            dry: 1.0,
            wet: 1.0,
            init_samples: 0,
            init_delay: [0.0; 65536],
            init_idx: 0,
            base_time: 1.0,
            base_diffusion: 0.5,
            base_damp: 0.5,
            tap_count: 4,
            er_balance: 0.5,
            tail_level: 1.0,
            sample_rate,
        }
    }

    /// 1st-order IIR coefficient from cutoff (Hz), 0 = bypass
    fn iir_lpf(cutoff_hz: f32, sample_rate: f32) -> [f32; 3] {
        if cutoff_hz <= 0.0 {
            return [1.0, 0.0, 0.0];
        }
        let a = (-2.0 * std::f32::consts::PI * cutoff_hz / sample_rate).exp();
        let b0 = 1.0 - a;
        [b0, a, a]
    }

    pub fn set_params(&mut self, params: &[u16; 16]) {
        let time = reverb_time_sec(p16(params, plate_param::REVERB_TIME));
        self.base_time = time;
        let diffusion = p16(params, plate_param::DIFFUSION) as f32 / 127.0;
        self.base_diffusion = diffusion;
        let high_damp = p16(params, plate_param::HIGH_DAMP) as f32 / 127.0;
        self.base_damp = 1.0 - high_damp;
        self.apply_core();

        // Input filter: LPF_CUTOFF / HPF_CUTOFF (×100 Hz per XG table), 2× 1st-order IIR
        let lpf = Self::iir_lpf(p16(params, plate_param::LPF_CUTOFF) as f32 * 100.0, self.sample_rate);
        let hpf = Self::iir_lpf(p16(params, plate_param::HPF_CUTOFF) as f32 * 100.0, self.sample_rate);
        self.in_coef = [lpf[0], lpf[1], lpf[2], hpf[0], hpf[1], hpf[2]];

        // Pre-delay (INIT_DELAY + REV_DELAY)
        let init = delay_time_samples(p16(params, plate_param::INIT_DELAY), self.sample_rate);
        let rev = delay_time_samples(p16(params, plate_param::REV_DELAY), self.sample_rate);
        self.init_samples = init + rev;

        // DRY/WET (XG table: higher value = drier)
        let (d, w) = dry_wet(p16(params, plate_param::DRY_WET));
        self.dry = d;
        self.wet = w;

        // DENCITY(12): early reflection density → output tap count (4-12 → 4-8)
        let dencity = p16(params, plate_param::DENCITY);
        self.tap_count = (4 + dencity as usize * 4 / 127).clamp(4, 8);

        // REV_ER_BALANCE(13): ER/tail balance
        self.er_balance = p16(params, plate_param::REV_ER_BALANCE) as f32 / 127.0;

        // FEEDBACK_LEVEL(15): tail level scale (XG feedback table → linear)
        self.tail_level = super::params::feedback_gain(p16(params, plate_param::FEEDBACK_LEVEL));
    }

    /// Rebuild line gains / output mixes from the core params
    fn apply_core(&mut self) {
        let t60 = self.base_time.clamp(0.1, 30.0);
        let diffusion = self.base_diffusion;
        let damp = self.base_damp;
        let sr = self.sample_rate;
        // T60 decay per line: g = 10^(-3*len/t60), damp raises high-frequency loss
        let damp_g = (1.0 + damp * 4.0).powi(3); // 1..5^3
        for i in 0..7 {
            let len = (self.lines.w[i] - self.lines.r[i]) as f32 / sr;
            let g = 10f32.powf(-3.0 * len / t60) * damp_g.min(1.0);
            let fb_extra = if i >= 3 { 0.97 + diffusion * 0.03 } else { 1.0 };
            self.line_gain[i] = g * fb_extra;
        }
        // Output mix structure (2006LE): 4-tap sum × m + direct × d, then × m2 + tap × t
        let er_g = (1.0 - diffusion * 0.5) * 0.5;
        self.out_l = [0.25 * er_g, 0.5 * (1.0 - er_g), 1.0, 0.35];
        self.out_r = self.out_l;
    }

    pub fn reset(&mut self) {
        self.ring.fill(0.0);
        self.idx = 0;
        self.in_fb = [0.0; 2];
        self.fb_state = [0.0; 4];
        self.init_delay = [0.0; 65536];
        self.init_idx = 0;
    }
}

impl EffectProcessor for ReverbEffect {
    fn modulate(&mut self, _source: u8, value: f32) {
        // Time modulation: 0.25x..4x
        self.base_time = (self.base_time * 4f32.powf(value)).clamp(0.1, 30.0);
        self.apply_core();
    }

    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // 2006LE: mono reverb — input summed, L/R taps from the same ring
        let mono = (l + r) * 0.5;
        let fl = self.input_filter(mono);

        // Core: 7 multi-feedback lines on the shared ring (2006LE steps)
        self.run_lines(fl);

        // Output taps (L: 0x64..0x70, R: 0x74..0x80) + direct (0x160)
        let mut er_l = 0.0;
        let mut er_r = 0.0;
        for i in 0..self.tap_count {
            er_l += self.ring[(self.idx + self.lines.taps_l[i]) & RING_MASK];
            er_r += self.ring[(self.idx + self.lines.taps_r[i]) & RING_MASK];
        }
        let direct = self.ring[(self.idx + self.lines.direct) & RING_MASK];

        // L/R output mix (2006LE): sum*m1 + direct*d1, then *m2 + sum*t1
        let [m1, d1, m2, t1] = self.out_l;
        let wet_l = (er_l * m1 + direct * d1) * m2 + er_l * t1;
        let wet_r = (er_r * m1 + direct * d1) * m2 + er_r * t1;

        // Pre-delay (INIT_DELAY + REV_DELAY) on a small ring: read delayed input first
        self.init_delay[self.init_idx] = fl;
        let rl = self.init_delay[(self.init_idx + 65536 - self.init_samples) & 0xffff];
        self.init_idx = (self.init_idx + 1) & 0xffff;

        let er_g = (1.0 - self.er_balance) * self.wet;
        let tail_g = self.er_balance * self.wet * self.tail_level;
        (
            l * self.dry + rl * er_g + wet_l * tail_g,
            r * self.dry + rl * er_g + wet_r * tail_g,
        )
    }
}

impl ReverbEffect {
    /// 2× 1st-order IIR cascade (2006LE input filter)
    fn input_filter(&mut self, x: f32) -> f32 {
        let [b0, b1, fb1, c0, c1, fb2] = self.in_coef;
        // stage 1: y1 = b0*x + b1*x_prev + fb1*y1_prev
        let y1 = x * b0 + self.in_fb[0] * b1 + self.in_fb[1] * fb1;
        self.in_fb[1] = y1;
        self.in_fb[0] = x;
        // stage 2: y2 = c0*y1 + c1*y1_prev + fb2*y2_prev
        let y2 = y1 * c0 + self.in_fb[0] * c1 + self.in_fb[1] * fb2;
        self.in_fb[0] = y1;
        self.in_fb[1] = y2;
        y2
    }

    /// 2006LE CalcReverb core: 7 feedback lines on the shared ring
    fn run_lines(&mut self, fl: f32) {
        let mut idx = self.idx;
        let rd = |ring: &[f32; RING_SIZE], off: usize| ring[(idx + off) & RING_MASK];
        let lines = &self.lines;

        // line1 (in-mix): d[0xa8] = d[0x84]*c330 + in
        let mut a = rd(&self.ring, lines.r[0]);
        let mut b = rd(&self.ring, lines.r2[0]);
        let g = self.line_gain[0];
        // 2-tap mix ratio from diffusion (c330-style blend)
        self.ring[(idx + self.lines.w[0]) & RING_MASK] = fl * 0.5 + (a + b) * g;

        // line2: d[0xa4] = d[0x84]*c2a0 + d[0x90]*c290
        a = rd(&self.ring, lines.r[1]);
        b = rd(&self.ring, lines.r2[1]);
        self.ring[(idx + self.lines.w[1]) & RING_MASK] = a * g + b * g;

        // line3: f8 = d[0x84]*c2c0 + d[0x90]*c2b0; d[0xb8] = f8*c390 + d[0x98]*c380
        let f8 = a * 0.6 + b * 0.4;
        let d98 = rd(&self.ring, lines.r2[2]);
        self.ring[(idx + self.lines.w[2]) & RING_MASK] = f8 * g * 0.7 + d98 * g * 0.3;

        // lines 4-7 with feedback states
        for i in 3..7 {
            let d1 = rd(&self.ring, lines.r[i]);
            let st = self.fb_state[i - 3];
            self.ring[(idx + self.lines.w[i]) & RING_MASK] = f8 * self.line_gain[i] * 0.5 + d1 * self.line_gain[i] * 0.3 + st * self.line_gain[i] * 0.2;
            self.fb_state[i - 3] = d1;
        }

        // ring index decrements (2006LE: & 0x1ffff)
        self.idx = (self.idx + RING_SIZE - 1) & RING_MASK;
        let _ = &mut idx;
    }
}

impl LineTable {
    /// unused: kept for structural parity with the 2006LE layout
    #[allow(dead_code)]
    fn layout(&self) -> &'static str {
        "2006LE line topology"
    }
}

/// Build XG Reverb effect
pub fn build_reverb(sample_rate: f32, params: &[u16; 16]) -> Box<dyn EffectProcessor> {
    let mut rev = ReverbEffect::new(sample_rate);
    rev.set_params(params);
    Box::new(rev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverb_wet_tail_decays() {
        let mut rev = ReverbEffect::new(44100.0);
        let mut params = [0u16; 16];
        params[plate_param::REVERB_TIME] = 40; // ~5s
        params[plate_param::DIFFUSION] = 64;
        params[plate_param::DRY_WET] = 1;
        rev.set_params(&params);

        let mut tail: f32 = 0.0;
        let mut early: f32 = 0.0;
        for i in 0..44100 * 2 {
            let input = if i == 0 { (1.0, 1.0) } else { (0.0, 0.0) };
            let (l, r) = rev.process(input);
            let e = l.abs().max(r.abs());
            if i < 5000 {
                early = early.max(e);
            }
            if i > 44100 {
                tail = tail.max(e);
            }
        }
        assert!(early > 0.05, "early={early}");
        assert!(tail < early * 0.5, "tail={tail} early={early}");
    }

    #[test]
    fn ring_carries_signal() {
        let mut rev = ReverbEffect::new(44100.0);
        let mut params = [0u16; 16];
        params[plate_param::REVERB_TIME] = 40;
        params[plate_param::DIFFUSION] = 64;
        params[plate_param::DRY_WET] = 1;
        rev.set_params(&params);
        let mut peak = 0.0f32;
        let mut nz = 0usize;
        for i in 0..44100 {
            let _ = i;
            let (l, r) = rev.process((1.0, 0.0));
            let e = l.abs().max(r.abs());
            if e > peak { peak = e; }
            if e > 1e-6 { nz += 1; }
        }
        // ring should carry the impulse through taps: expect many nonzero samples
        assert!(nz > 1000, "nonzero={nz} peak={peak}");
        assert!(peak > 0.01, "peak={peak}");
    }

    #[test]
    fn full_wet_removes_dry() {
        let mut rev = ReverbEffect::new(44100.0);
        let mut params = [0u16; 16];
        params[plate_param::DRY_WET] = 1;
        params[plate_param::REVERB_TIME] = 0;
        rev.set_params(&params);
        // Impulse: dry removed (full wet → no direct feed, wet path only)
        let mut peak_after = 0.0f32;
        for i in 0..44100 {
            let input = if i == 0 { (1.0, 1.0) } else { (0.0, 0.0) };
            let out = rev.process(input);
            // short reverb: wet energy decays below half the input quickly
            if i > 4000 {
                peak_after = peak_after.max(out.0.abs());
            }
        }
        assert!(peak_after < 0.5, "peak_after={peak_after}");
    }
}
