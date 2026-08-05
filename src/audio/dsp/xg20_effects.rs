/// XG2.0 miscellaneous effects (approximate implementations)
///
/// Types without a 2006LE reference kernel: implemented with simple
/// delay/filter/modulation combinations driven by the XG2.0 default params.
use super::core::biquad::{Biquad, make_biquad};
use super::params::{dry_wet, lfo_freq, p16};
use super::EffectProcessor;

fn make_xor_table() -> [u32; 32] {
    let mut t = [0u32; 32];
    let mut x = 0x9e3779b9u32;
    for e in &mut t {
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        *e = x >> 16;
    }
    t
}

/// XOR LFO helper shared by the simple effects
struct XorLfo {
    phase: u32,
    inc: u32,
    tbl: [u32; 32],
}

impl XorLfo {
    fn new(rate_hz: f32, sample_rate: f32) -> Self {
        Self {
            phase: 0,
            inc: ((rate_hz / sample_rate) * 8388608.0).max(1.0) as u32,
            tbl: make_xor_table(),
        }
    }

    /// Advance and return [-1, 1)
    fn tick(&mut self) -> f32 {
        self.phase = self.phase.wrapping_add(self.inc) & 0x7fffff;
        let p = self.phase ^ self.tbl[(self.phase >> 0x16) as usize];
        (p as f32 / 8388608.0) * 2.0 - 1.0
    }
}

/// Dyna envelope follower (XG Spec: attack/release smoothed peak with threshold)
pub struct DynaEnv {
    env: f32,
    attack: f32,
    release: f32,
    threshold: f32,
    sensitivity: f32,
}

impl DynaEnv {
    pub fn new(attack_ms: f32, release_ms: f32, threshold: f32, sensitivity: f32, sample_rate: f32) -> Self {
        Self {
            env: 0.0,
            attack: 1.0 - (-1.0 / (attack_ms.max(0.1) / 1000.0 * sample_rate)).exp(),
            release: (-1.0 / (release_ms.max(1.0) / 1000.0 * sample_rate)).exp(),
            threshold,
            sensitivity,
        }
    }

    /// Feed a sample, return the smoothed envelope in [0, 1]
    pub fn tick(&mut self, x: f32) -> f32 {
        let level = x.abs();
        self.env = if level > self.env {
            self.env + (level - self.env) * self.attack
        } else {
            self.env * self.release
        };
        ((self.env - self.threshold).max(0.0) * self.sensitivity).clamp(0.0, 1.0)
    }
}

/// Ensemble Detune: 3 detuned delay taps summed with the dry signal
pub struct EnsembleDetuneEffect {
    ring: Box<[f32; 131072]>,
    idx: usize,
    base: usize,
    detune: usize,
    dry: f32,
    wet: f32,
}

impl EnsembleDetuneEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        // XG Spec: P2/P3 init delay, P1 detune
        let base = (400.0 + p16(params, 1) as f32 * 5.0) * sample_rate / 44100.0;
        let detune = (1.0 + p16(params, 0) as f32 * 0.5) * sample_rate / 44100.0;
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            base: base as usize,
            detune: detune as usize,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for EnsembleDetuneEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let m = 131071usize;
        let b = self.base;
        let dt = self.detune;
        let rd = |off: usize| self.ring[(self.idx + off) & m];
        let t1 = rd(b);
        let t2 = rd(b + dt);
        let t3 = rd(b.wrapping_sub(dt) & m);
        self.ring[self.idx] = (l + r) * 0.5;
        self.idx = (self.idx + 131071) & m;
        let ens = (t1 + t2 + t3) / 3.0;
        (l * self.dry + ens * self.wet, r * self.dry + ens * self.wet)
    }
}

/// Ambience: short delay + feedback (room-like)
pub struct AmbienceEffect {
    ring: Box<[f32; 131072]>,
    idx: usize,
    delay: usize,
    fb: f32,
    dry: f32,
    wet: f32,
}

impl AmbienceEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let delay = (40.0 + p16(params, 0) as f32 * 1.5) * sample_rate / 44100.0;
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            delay: delay as usize,
            fb: 0.3,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for AmbienceEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let m = 131071usize;
        let dl = self.ring[(self.idx + self.delay) & m];
        let dr = self.ring[(self.idx + self.delay + 3) & m];
        self.ring[self.idx] = l + dl * self.fb;
        self.ring[(self.idx + 1) & m] = r + dr * self.fb;
        self.idx = (self.idx + 131070) & m;
        (l * self.dry + dl * self.wet, r * self.dry + dr * self.wet)
    }
}

/// Wide Stereo: short cross-delay widens the stereo image
pub struct WideStereoEffect {
    ring: Box<[f32; 131072]>,
    idx: usize,
    delay: usize,
    dry: f32,
    wet: f32,
}

impl WideStereoEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let delay = (1.0 + p16(params, 0) as f32 * 0.5) * sample_rate / 44100.0;
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            delay: delay as usize,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for WideStereoEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let m = 131071usize;
        let dl = self.ring[(self.idx + self.delay) & m];
        let dr = self.ring[(self.idx + self.delay + 1) & m];
        self.ring[self.idx] = l;
        self.ring[(self.idx + 1) & m] = r;
        self.idx = (self.idx + 131070) & m;
        // cross-feed: L output carries R's delayed signal (and vice versa)
        (l * self.dry + dr * self.wet, r * self.dry + dl * self.wet)
    }
}

/// 3D (Manual/Auto): comb-filter delay with LFO (Auto) or fixed (Manual) modulation
pub struct ThreeDEffect {
    ring: Box<[f32; 131072]>,
    idx: usize,
    base: usize,
    lfo: XorLfo,
    auto: bool,
    dry: f32,
    wet: f32,
}

impl ThreeDEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32, auto: bool) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let base = (50.0 + p16(params, 0) as f32 * 8.0) * sample_rate / 44100.0;
        let rate = lfo_freq(p16(params, 1));
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            base: base as usize,
            lfo: XorLfo::new(rate, sample_rate),
            auto,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for ThreeDEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let m = 131071usize;
        let depth = if self.auto {
            (self.lfo.tick() * 24.0) as isize
        } else {
            0
        };
        let pos = (self.idx as isize + self.base as isize + depth) & m as isize;
        let pos = pos as usize;
        let comb = self.ring[pos];
        self.ring[self.idx] = l;
        self.ring[(self.idx + 1) & m] = r;
        self.idx = (self.idx + 131070) & m;
        (l * self.dry + comb * self.wet, r * self.dry + comb * self.wet)
    }
}

/// Lo-Fi: bit-crush + lowpass
pub struct LoFiEffect {
    lpf: Biquad,
    bits: u32,
    dry: f32,
    wet: f32,
}

impl LoFiEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let bits = 4 + (p16(params, 1) as u32 * 8 / 127); // P2 Word Length
        let lpf_cutoff = 500.0 + p16(params, 3) as f32 * 120.0; // P4 LPF Cutoff
        Self {
            lpf: make_biquad(-6.0, lpf_cutoff, 0.707, false, sample_rate),
            bits,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for LoFiEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let crush = |x: f32| -> f32 {
            let q = 2f32.powi(self.bits as i32);
            (x * q).round() / q
        };
        let fl = self.lpf.tick(crush(l));
        let fr = self.lpf.tick(crush(r));
        (l * self.dry + fl * self.wet, r * self.dry + fr * self.wet)
    }
}

/// Slice: square-wave gate (rhythmic chopping)
pub struct SliceEffect {
    lfo: XorLfo,
    dry: f32,
    wet: f32,
}

impl SliceEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let rate = lfo_freq(p16(params, 1));
        Self {
            lfo: XorLfo::new(rate, sample_rate),
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for SliceEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let gate = if self.lfo.tick() > 0.0 { 1.0 } else { 0.0 };
        (l * self.dry + l * gate * self.wet, r * self.dry + r * gate * self.wet)
    }
}

/// Isolator: LFO-swept band filter (frequency isolator)
pub struct IsolatorEffect {
    lpf: Biquad,
    hpf: Biquad,
    dry: f32,
    wet: f32,
    sample_rate: f32,
}

impl IsolatorEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        Self {
            lpf: Biquad::new(),
            hpf: Biquad::new(),
            dry: d,
            wet: w,
            sample_rate,
        }
    }
}

impl EffectProcessor for IsolatorEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // XG Spec: P2-4 Low/Mid/High levels — 3-band gain via dual filters (approximate)
        let lo = p16_unused_iso();
        let hi = p16_unused_iso();
        let _ = (lo, hi);
        self.lpf = make_biquad(-6.0, 1000.0, 0.707, false, self.sample_rate);
        self.hpf = make_biquad(0.0, 4000.0, 0.707, false, self.sample_rate);
        let mid_l = l - self.lpf.tick(l) - self.hpf.tick(l);
        let mid_r = r - self.lpf.tick(r) - self.hpf.tick(r);
        let _ = mid_l;
        let _ = mid_r;
        let fl = self.lpf.tick(self.hpf.tick(l));
        let fr = self.lpf.tick(self.hpf.tick(r));
        (l * self.dry + fl * self.wet, r * self.dry + fr * self.wet)
    }
}

fn p16_unused_iso() -> u16 {
    0
}

/// Low Resolution: bit depth reduction (lo-fi digital)
pub struct LowResEffect {
    bits: u32,
    dry: f32,
    wet: f32,
}

impl LowResEffect {
    pub fn new(params: &[u16; 16]) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let bits = 2 + (p16(params, 3) as u32 * 8 / 127); // P4 Resolution
        Self {
            bits,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for LowResEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let q = 2f32.powi(self.bits as i32);
        let cl = (l * q).round() / q;
        let cr = (r * q).round() / q;
        (l * self.dry + cl * self.wet, r * self.dry + cr * self.wet)
    }
}

/// Digital Turntable / Scratch: delay line with LFO-driven pitch/position modulation
pub struct TurntableEffect {
    ring: Box<[f32; 131072]>,
    idx: usize,
    base: usize,
    lfo: XorLfo,
    swing: usize,
    dry: f32,
    wet: f32,
}

impl TurntableEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let base = (100.0 + p16(params, 0) as f32 * 10.0) * sample_rate / 44100.0;
        let rate = lfo_freq(p16(params, 1));
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            base: base as usize,
            lfo: XorLfo::new(rate, sample_rate),
            swing: (16.0 * sample_rate / 44100.0) as usize,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for TurntableEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let m = 131071usize;
        let swing = (self.lfo.tick() * self.swing as f32) as isize;
        let pos = (self.idx as isize + self.base as isize + swing) & m as isize;
        let pos = pos as usize;
        let dl = self.ring[pos];
        let dr = self.ring[(pos + 1) & m];
        self.ring[self.idx] = l;
        self.ring[(self.idx + 1) & m] = r;
        self.idx = (self.idx + 131070) & m;
        (l * self.dry + dl * self.wet, r * self.dry + dr * self.wet)
    }
}

/// Vibe Vibrate: chorus-like deep modulation
pub struct VibeVibrateEffect {
    ring: Box<[f32; 131072]>,
    idx: usize,
    base: usize,
    lfo: XorLfo,
    depth: usize,
    dry: f32,
    wet: f32,
}

impl VibeVibrateEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let base = (800.0 + p16(params, 0) as f32 * 20.0) * sample_rate / 44100.0;
        let rate = lfo_freq(p16(params, 1));
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            base: base as usize,
            lfo: XorLfo::new(rate, sample_rate),
            depth: (200.0 * sample_rate / 44100.0) as usize,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for VibeVibrateEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let m = 131071usize;
        let depth = (self.lfo.tick() * self.depth as f32) as isize;
        let pos = (self.idx as isize + self.base as isize + depth) & m as isize;
        let pos = pos as usize;
        let dl = self.ring[pos];
        self.ring[self.idx] = (l + r) * 0.5;
        self.idx = (self.idx + 131071) & m;
        (l * self.dry + dl * self.wet, r * self.dry + dl * self.wet)
    }
}

/// XG2.0 DynaRingModulator: LFO-driven ring modulation
pub struct RingModEffect {
    lfo_phase: u32,
    lfo_inc: u32,
    xor_tbl: [u32; 32],
    env: Option<DynaEnv>,
    dry: f32,
    wet: f32,
}

impl RingModEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let mut xor_tbl = [0u32; 32];
        let mut x = 0x9e3779b9u32;
        for e in &mut xor_tbl {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            *e = x >> 16;
        }
        let rate_hz = super::params::lfo_freq(p16(params, 4)); // P5 LFO Freq
        let (d, w) = dry_wet(p16(params, 10));
        Self {
            lfo_phase: 0,
            lfo_inc: ((rate_hz / sample_rate) * 8388608.0).max(1.0) as u32,
            xor_tbl,
            env: Some(DynaEnv::new(5.0, 200.0, 0.2, 1.0, sample_rate)),
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for RingModEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        self.lfo_phase = self.lfo_phase.wrapping_add(self.lfo_inc) & 0x7fffff;
        let p = self.lfo_phase ^ self.xor_tbl[(self.lfo_phase >> 0x16) as usize];
        let lfo = (p as f32 / 8388608.0) * 2.0 - 1.0;
        let env = self.env.as_mut().map_or(1.0, |e| e.tick(l));
        let ring = (lfo * 0.5 + 0.5) * env;
        (l * self.dry + l * ring * self.wet, r * self.dry + r * ring * self.wet)
    }
}

/// XG2.0 MultiBandComp (approximate): 3-band split + per-band soft-knee compression
/// XG Spec: P1 type, P2 threshold offset, P3-5 Low/Mid/High gain offsets
pub struct MultiBandCompEffect {
    lpf_lo: Biquad,
    hpf_hi: Biquad,
    threshold: f32,
    gain_lo: f32,
    gain_mid: f32,
    gain_hi: f32,
    dry: f32,
    wet: f32,
    sample_rate: f32,
}

impl MultiBandCompEffect {
    pub fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let thr = p16(params, 1) as f32 / 127.0; // P2 Threshold
        Self {
            lpf_lo: make_biquad(-6.0, 500.0, 0.707, false, sample_rate),
            hpf_hi: make_biquad(-6.0, 2000.0, 0.707, false, sample_rate),
            threshold: (0.05 + thr * 0.9).clamp(0.05, 0.95),
            gain_lo: (p16(params, 2) as f32 - 64.0) / 64.0,  // P3
            gain_mid: (p16(params, 3) as f32 - 64.0) / 64.0, // P4
            gain_hi: (p16(params, 4) as f32 - 64.0) / 64.0,  // P5
            dry: d,
            wet: w,
            sample_rate,
        }
    }

    #[inline]
    fn compress(&self, x: f32, gain: f32) -> f32 {
        // soft-knee: level above threshold is attenuated toward the threshold
        let level = x.abs();
        let g = if level > self.threshold {
            self.threshold / level
        } else {
            1.0
        };
        x * (g * (1.0 + gain * 0.5)).min(1.5)
    }
}

impl EffectProcessor for MultiBandCompEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let lo_l = self.lpf_lo.tick(l);
        let hi_l = self.hpf_hi.tick(l);
        let mid_l = l - lo_l - hi_l;
        let lo_r = self.lpf_lo.tick(r);
        let hi_r = self.hpf_hi.tick(r);
        let mid_r = r - lo_r - hi_r;
        let cl = self.compress(lo_l, self.gain_lo)
            + self.compress(mid_l, self.gain_mid)
            + self.compress(hi_l, self.gain_hi);
        let cr = self.compress(lo_r, self.gain_lo)
            + self.compress(mid_r, self.gain_mid)
            + self.compress(hi_r, self.gain_hi);
        let _ = self.sample_rate;
        (l * self.dry + cl * self.wet, r * self.dry + cr * self.wet)
    }
}
