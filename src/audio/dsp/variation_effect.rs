/// XG Variation effect (delay family + reverb family + Thru)
///
/// Implemented types:
/// - Thru / NoEffect: passthrough
/// - DelayLCR / DelayLR / Echo / CrossDelay: delay family
/// - Hall1/2, Room1-3, Stage1/2, Plate, ER1/2, GateReverb, ReverseGate: reuse reverb kernel
/// - Other types: phase 3 extension (currently Thru placeholder)
///
/// Params (effect_obj index): delay_lcr_param etc., 14-bit params merged at the parse layer
use super::core::biquad::{Biquad, make_biquad};
use super::core::eq_chain::EqChain;
use super::distortion_effects::{AmpSimEffect, AuralExciterEffect, DistortionEffect};
use super::dynamics::{CompressorEffect, NoiseGateEffect};
use super::eq_effects::{ThreeBandEqEffect, TwoBandEqEffect};
use super::misc_effects::{KaraokeEffect, PitchChangeEffect, VoiceCancelEffect};
use super::modulation_effects::{ModEffectKind, build_modulation};
use super::params::{dry_wet, feedback_gain, p16};
use super::wah_effects::{AutoWahEffect, TouchWahEffect};
use super::EffectProcessor;
use crate::midi::effect_params::effect_obj::{delay_lcr_param, delay_lr_param, distortion_param, echo_param, phaser_param};
use crate::midi::effect_params::variation_type::XGVariationType;

/// Delay family common params (10=DRY_WET, 15=FEEDBACK_LEVEL, 11/12=HPF/LPF, 13-16=EQ)
struct DelayCommon {
    dry: f32,
    wet: f32,
    feedback: f32,
    high_damp: f32,
    /// Input filtering (11/12) + 3-band EQ (13/14, 15/16)
    hpf: Biquad,
    lpf: Biquad,
    eq: EqChain,
}

/// 2006LE CalcDelay kernel (0x56b64): shared 131072 ring + 4 read taps + 2 write taps
///   per sample: tap1..4 read, 1st-order in-filter, L/R writes (out-feedback cross),
///   L/R outputs = in×d + tap×d mix; idx decrements & 0x1ffff
struct DelayKernel2006 {
    ring: Box<[f32; 131072]>,
    idx: usize,
    /// Read tap offsets (L1/R1/L2/R2)
    tap: [usize; 4],
    /// Write offsets (L/R)
    w: [usize; 2],
    /// Input 1st-order filter states/coefs
    in_prev: [f32; 2],
    in_coef: [[f32; 2]; 2],
    /// Write mixes (L: [fb, lin, rin], R: [fb, rin, lin])
    wm: [[f32; 3]; 2],
    /// Output mixes (L/R: [lin, rin, tap1, tap3, tap4])
    om: [[f32; 5]; 2],
    /// Previous-sample outputs (feedback loop)
    fb_prev: [f32; 2],
    /// Echo swing (2006LE CalcPan: LFO-driven delay swing + amp modulation)
    lfo_phase: u32,
    lfo_inc: u32,
    xor_tbl: [u32; 32],
    swing: f32,
    swing_amp: f32,
}

impl DelayKernel2006 {
    fn new(sample_rate: f32, taps: [usize; 4], fb: f32, dry: f32, wet: f32) -> Self {
        let _ = sample_rate;
        let c = fb.clamp(-0.99, 0.99);
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            tap: taps,
            w: [0, 0],
            in_prev: [0.0; 2],
            in_coef: [[1.0, 0.0]; 2],
            wm: [[c * 0.5, 0.5, 0.0], [c * 0.5, 0.5, 0.0]],
            om: [
                [dry, 0.0, wet * 0.5, wet * 0.25, wet * 0.25],
                [dry, 0.0, wet * 0.5, wet * 0.25, wet * 0.25],
            ],
            fb_prev: [0.0; 2],
            lfo_phase: 0,
            lfo_inc: 0,
            xor_tbl: [0; 32],
            swing: 0.0,
            swing_amp: 0.0,
        }
    }

    /// Enable CalcPan-style swing: LFO (23-bit + XOR) drives the L/R taps apart,
    /// with amplitude modulation (pan movement)
    fn set_swing(&mut self, rate_hz: f32, sample_rate: f32, depth_samples: f32) {
        let mut x = 0x9e3779b9u32;
        for e in &mut self.xor_tbl {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            *e = x >> 16;
        }
        self.lfo_inc = ((rate_hz / sample_rate) * 8388608.0).max(1.0) as u32;
        self.swing = depth_samples.max(0.0);
        self.swing_amp = (depth_samples / 64.0).clamp(0.0, 0.2);
    }

    fn tick(&mut self, l: f32, r: f32) -> (f32, f32) {
        let m = 131071usize;
        let t = self.tap;
        // CalcPan-style swing: L/R taps modulated in opposite directions
        let (off_a, _frac_a, amp_a, amp_b) = if self.swing > 0.0 {
            self.lfo_phase = self.lfo_phase.wrapping_add(self.lfo_inc) & 0x7fffff;
            let p = self.lfo_phase ^ self.xor_tbl[(self.lfo_phase >> 0x16) as usize];
            let lfo = (p as f32 / 8388608.0) * 2.0 - 1.0;
            let v = lfo * self.swing;
            let off = v.floor() as i32;
            (off, v - v.floor(), 1.0 + lfo * self.swing_amp, 1.0 - lfo * self.swing_amp)
        } else {
            (0, 0.0, 1.0, 1.0)
        };
        let rd1 = |idx: usize, off: usize| (idx as isize + off as isize + off_a as isize) & m as isize;
        let t1 = self.ring[rd1(self.idx, t[0]) as usize] * amp_a;
        let t2 = self.ring[rd1(self.idx, t[1]) as usize] * amp_b;
        let t3 = self.ring[(self.idx + t[2]) & m];
        let t4 = self.ring[(self.idx + t[3]) & m];
        // 1st-order input filters (2006LE p[0x9e0-0xa10])
        let f15 = t1 * self.in_coef[0][0] + self.in_prev[0] * self.in_coef[0][1];
        let f14 = t2 * self.in_coef[1][0] + self.in_prev[1] * self.in_coef[1][1];
        self.in_prev = [t1, t2];
        // L write (2006LE p[0xaa0-0xac0]): out-feedback + input + cross
        let [a, b, c] = self.wm[0];
        self.ring[(self.idx + self.w[0]) & m] = self.fb_prev[1] * a + l * b + f14 * c;
        let [a, b, c] = self.wm[1];
        self.ring[(self.idx + self.w[1]) & m] = self.fb_prev[0] * a + r * b + f15 * c;
        // Outputs (2006LE p[0xb20-0xbb0])
        let [d1, d2, d3, d4, d5] = self.om[0];
        let out_l = l * d1 + r * d2 + t1 * d3 + t3 * d4 + t4 * d5;
        let [e1, e2, e3, e4, e5] = self.om[1];
        let out_r = r * e1 + l * e2 + t2 * e3 + t4 * e4 + t3 * e5;
        self.fb_prev = [out_l, out_r];
        self.idx = (self.idx + 131071) & m;
        (out_l, out_r)
    }
}

impl DelayCommon {
    /// Input signal processing: HPF → LPF → EQ
    #[inline]
    fn process_input(&mut self, x: f32) -> f32 {
        let x = self.hpf.tick(x);
        let x = self.lpf.tick(x);
        self.eq.tick(x)
    }
}

/// Delay family HPF/LPF params (0-127) → Hz (logarithmic 20Hz-16kHz)
fn delay_cutoff_hz(v: u16) -> f32 {
    let t = v.min(127) as f32 / 127.0;
    20.0 * 800f32.powf(t)
}

fn delay_common(params: &[u16; 16], sample_rate: f32) -> DelayCommon {
    let (d, w) = dry_wet(p16(params, 10));
    let mut common = DelayCommon {
        dry: d,
        wet: w,
        feedback: feedback_gain(p16(params, 15)),
        high_damp: 1.0 - p16(params, 7) as f32 / 127.0,
        hpf: Biquad::new(),
        lpf: Biquad::new(),
        eq: EqChain::new(),
    };
    // HPF(11)/LPF(12): 0=off
    let hpf_v = p16(params, 11);
    common.hpf = if hpf_v > 0 {
        make_biquad(0.0, delay_cutoff_hz(hpf_v), 0.707, false, sample_rate)
    } else {
        Biquad::new()
    };
    let lpf_v = p16(params, 12);
    common.lpf = if lpf_v > 0 {
        make_biquad(-12.0, delay_cutoff_hz(lpf_v), 0.707, false, sample_rate)
    } else {
        Biquad::new()
    };
    // EQ: LOW(13/14), HIGH(15/16) (no MID band param)
    common.eq.set_bands(p16(params, 13), p16(params, 14), 0, 64, 64, p16(params, 15), p16(params, 16), sample_rate);
    common
}

/// Delay LCR: 2006LE kernel with L/C/R tap layout (L delay, R delay, C center taps)
struct DelayLcr {
    k: DelayKernel2006,
    common: DelayCommon,
}

impl DelayLcr {
    fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let common = delay_common(params, sample_rate);
        let d = |v: u16| delay_samples(v, sample_rate);
        let taps = [
            d(p16(params, delay_lcr_param::L_CH_DELAY)) as usize,
            d(p16(params, delay_lcr_param::R_CH_DELAY)) as usize,
            d(p16(params, delay_lcr_param::C_CH_DELAY)) as usize,
            d(p16(params, delay_lcr_param::C_CH_DELAY)) as usize,
        ];
        let c_level = p16(params, 6) as f32 / 127.0;
        let mut k = DelayKernel2006::new(sample_rate, taps, common.feedback * common.high_damp * c_level, common.dry, common.wet);
        // center feedback into both writes; center taps mixed into both outputs
        k.wm[0][0] = common.feedback * common.high_damp * c_level * 0.5;
        k.wm[1][0] = k.wm[0][0];
        k.om[0][3] = common.wet * 0.5;
        k.om[0][4] = common.wet * 0.5;
        k.om[1][3] = common.wet * 0.5;
        k.om[1][4] = common.wet * 0.5;
        Self { k, common }
    }
}

impl EffectProcessor for DelayLcr {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let l = self.common.process_input(l);
        let r = self.common.process_input(r);
        self.k.tick(l, r)
    }
}

/// Delay LR: L/R dual delay + independent feedback
struct DelayLr {
    k: DelayKernel2006,
    common: DelayCommon,
}

impl DelayLr {
    fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let common = delay_common(params, sample_rate);
        let taps = [
            delay_samples(p16(params, delay_lr_param::L_CH_DELAY), sample_rate) as usize,
            delay_samples(p16(params, delay_lr_param::R_CH_DELAY), sample_rate) as usize,
            0,
            0,
        ];
        let mut k = DelayKernel2006::new(sample_rate, taps, 0.0, common.dry, common.wet);
        // per-channel feedback (FEEDBACK_DELAY_1/2)
        k.wm[0][0] = feedback_gain(p16(params, delay_lr_param::FEEDBACK_DELAY_1)) * common.high_damp * 0.5;
        k.wm[1][0] = feedback_gain(p16(params, delay_lr_param::FEEDBACK_DELAY_2)) * common.high_damp * 0.5;
        k.om[0][2] = common.wet;
        k.om[0][3] = 0.0;
        k.om[0][4] = 0.0;
        k.om[1][2] = common.wet;
        k.om[1][3] = 0.0;
        k.om[1][4] = 0.0;
        Self { k, common }
    }
}

impl EffectProcessor for DelayLr {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let l = self.common.process_input(l);
        let r = self.common.process_input(r);
        self.k.tick(l, r)
    }
}

/// Echo: 2006LE kernel with two delay groups (L1/R1 + L2/R2, DELAY2_LEVEL)
struct EchoDelay {
    k: DelayKernel2006,
    common: DelayCommon,
}

impl EchoDelay {
    fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let common = delay_common(params, sample_rate);
        let taps = [
            delay_samples(p16(params, echo_param::L_CH_DELAY_1), sample_rate) as usize,
            delay_samples(p16(params, echo_param::R_CH_DELAY_1), sample_rate) as usize,
            delay_samples(p16(params, echo_param::L_CH_DELAY_2), sample_rate) as usize,
            delay_samples(p16(params, echo_param::R_CH_DELAY_2), sample_rate) as usize,
        ];
        let fb = feedback_gain(p16(params, echo_param::L_CH_FEEDBACK_LEVEL)) * common.high_damp;
        let mut k = DelayKernel2006::new(sample_rate, taps, fb, common.dry, common.wet);
        k.wm[0][0] = fb * 0.5;
        k.wm[1][0] = feedback_gain(p16(params, echo_param::R_CH_FEEDBACK_LEVEL)) * common.high_damp * 0.5;
        // CalcPan-style echo swing: LFO pans the echo taps (L/R opposite)
        k.set_swing(0.5, sample_rate, 24.0);
        let level2 = p16(params, echo_param::DELAY2_LEVEL) as f32 / 127.0;
        k.om[0][2] = common.wet * 0.6;
        k.om[0][3] = common.wet * 0.4 * level2;
        k.om[0][4] = common.wet * 0.4 * level2;
        k.om[1][2] = common.wet * 0.6;
        k.om[1][3] = common.wet * 0.4 * level2;
        k.om[1][4] = common.wet * 0.4 * level2;
        let _ = level2;
        Self { k, common }
    }
}

impl EffectProcessor for EchoDelay {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let l = self.common.process_input(l);
        let r = self.common.process_input(r);
        self.k.tick(l, r)
    }
}

/// Cross Delay: 2006LE kernel with cross feedback (L→R, R→L)
struct CrossDelayEffect {
    k: DelayKernel2006,
    common: DelayCommon,
    input_select: bool,
}

impl CrossDelayEffect {
    fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let common = delay_common(params, sample_rate);
        let taps = [
            delay_samples(p16(params, delay_lcr_param::L_CH_DELAY), sample_rate) as usize,
            delay_samples(p16(params, delay_lcr_param::R_CH_DELAY), sample_rate) as usize,
            0,
            0,
        ];
        let fb = feedback_gain(p16(params, delay_lcr_param::FEEDBACK_DELAY)) * common.high_damp;
        let mut k = DelayKernel2006::new(sample_rate, taps, fb, common.dry, common.wet);
        // cross feedback: L write gets R feedback, R write gets L feedback
        k.wm[0][0] = fb * 0.5;
        k.wm[1][0] = fb * 0.5;
        k.om[0][2] = common.wet;
        k.om[0][3] = 0.0;
        k.om[0][4] = 0.0;
        k.om[1][2] = common.wet;
        k.om[1][3] = 0.0;
        k.om[1][4] = 0.0;
        Self {
            k,
            common,
            input_select: p16(params, 4) != 0,
        }
    }
}

impl EffectProcessor for CrossDelayEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let l = self.common.process_input(l);
        let r = self.common.process_input(r);
        // INPUT_SELECT(4): 0=stereo, 1=L→R serial
        if self.input_select {
            self.k.tick(0.0, l)
        } else {
            self.k.tick(l, r)
        }
    }
}

/// Short reverb (ER1/2, GateReverb, ReverseGate): reuse reverb kernel + full params
/// 2006LE CalcEarlyRef kernel (0x5507a): 3 feedback lines + 7-tap ER mix + 7-tap tail mix
///   ring 131072, idx decrements; tap offsets default ER1 layout (44.1k samples)
struct EarlyRef2006 {
    ring: Box<[f32; 131072]>,
    idx: usize,
    /// Feedback line reads/writes
    line_w: [usize; 3],
    line_r: [usize; 3],
    /// ER taps (7 reads for the ER mix, 7 for the tail mix)
    er_taps: [usize; 7],
    tail_taps: [usize; 7],
    /// Tail/ER taps into the output
    tap_out: [usize; 2],
    /// Line 3 filter states
    f_state: [f32; 2],
    /// Coefficients: line feedback, ER tap gains, tail tap gains
    fb: f32,
    er_g: [f32; 7],
    tail_g: [f32; 7],
    dry: f32,
    wet: f32,
}

impl EarlyRef2006 {
    fn new(params: &[u16; 16], sample_rate: f32, fb: f32) -> Self {
        let (d, w) = dry_wet(p16(params, 10));
        let mut er_g = [0.0f32; 7];
        let mut tail_g = [0.0f32; 7];
        // 2006LE ER layout: params[0..3] are 14-bit delay samples driving the line taps
        let d0 = (params[0] & 0x3FFF) as usize;
        let d1 = (params[1] & 0x3FFF) as usize;
        let d2 = (params[2] & 0x3FFF) as usize;
        let d3 = (params[3] & 0x3FFF) as usize;
        for i in 0..7 {
            er_g[i] = 0.5 * 0.75f32.powi(i as i32);
            tail_g[i] = 0.15 * 0.8f32.powi(i as i32);
        }
        let _ = sample_rate;
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            line_w: [d0 + 32, d1 + 32, d2 + 32],
            line_r: [d0, d1, d2],
            er_taps: [d0, d1, d2, d3, d3 + 16, d2 + 16, d0 + 16],
            tail_taps: [d0 + 512, d1 + 512, d2 + 512, d3 + 512, d3 + 528, d2 + 528, d0 + 528],
            tap_out: [d3 + 64, d3 + 128],
            f_state: [0.0; 2],
            fb: fb.clamp(-0.99, 0.99),
            er_g,
            tail_g,
            dry: d,
            wet: w,
        }
    }

    fn tick(&mut self, x: f32) -> f32 {
        let m = 131071usize;
        let ring = &mut self.ring;
        let idx = self.idx;

        let l1 = ring[(idx + self.line_r[0]) & m];
        ring[(idx + self.line_w[0]) & m] = x * 0.7 + l1 * self.fb * 0.3;
        let f13 = x * 0.5 + l1 * 0.3;
        let l2 = ring[(idx + self.line_r[1]) & m];
        ring[(idx + self.line_w[1]) & m] = f13 * 0.7 + l2 * self.fb * 0.3;
        let l3 = ring[(idx + self.line_r[2]) & m];
        let f11 = l3 * self.fb * 0.6 + self.f_state[0] * 0.4 + self.f_state[1] * 0.15;
        self.f_state[1] = f11;
        self.f_state[0] = l3;
        ring[(idx + self.line_w[2]) & m] = f13 * 0.5 + l2 * 0.3 + f11 * 0.3;

        let mut er = 0.0;
        for (i, &t) in self.er_taps.iter().enumerate() {
            er += ring[(idx + t) & m] * self.er_g[i];
        }
        let mut tail = 0.0;
        for (i, &t) in self.tail_taps.iter().enumerate() {
            tail += ring[(idx + t) & m] * self.tail_g[i];
        }
        ring[(idx + self.line_w[2] + 40) & m] = er;
        let wet_l = er + tail * 0.5;
        ring[(idx + self.line_w[2] + 52) & m] = tail;
        let dry_l = er + ring[(idx + self.tap_out[1]) & m] * 0.4;

        self.idx = (self.idx + 131071) & m;
        dry_l * 0.5 + wet_l
    }
}

/// XG2.0 serial chain: front effect (full wet) feeds a delay back
/// front params occupy [4..15] of the serial type's param set; delay taps = [0..3] (14-bit)
struct SerialChain {
    front: Box<dyn EffectProcessor>,
    mid: Option<Box<dyn EffectProcessor>>,
    back: Box<dyn EffectProcessor>,
}

impl SerialChain {
    /// XG2.0 serial layout (XG Spec): Dist+Delay = P1-3 delay times (14-bit),
    /// P4 feedback level, P5 delay mix, P6-9 distortion (Drive/Output/EQ_Low/EQ_Mid), P10 dry/wet
    fn new_distortion(params: &[u16; 16], sample_rate: f32, distortion: bool) -> Self {
        let mut front = [0u16; 16];
        front[distortion_param::DRIVE] = params[5];       // P6 Dist Drive
        front[distortion_param::OUTPUT_LEVEL] = params[6]; // P7 Dist Output
        front[distortion_param::EQ_LOW_GAIN] = params[7];  // P8 Dist EQ Low Gain
        front[distortion_param::EQ_MID_GAIN] = params[8];  // P9 Dist EQ Mid Gain
        front[distortion_param::DRY_WET] = params[9];      // P10 Dry/Wet (serial: full wet)
        front[distortion_param::DRY_WET] = 1;
        let mut e = DistortionEffect::new(sample_rate);
        e.set_params(&front, distortion);
        // delay back: P1-P3 14-bit delay times, P4 feedback level
        let d0 = (params[0] & 0x3FFF) as usize;
        let d1 = (params[1] & 0x3FFF) as usize;
        let d2 = (params[2] & 0x3FFF) as usize;
        let fb = super::params::feedback_gain(params[3]);
        let back = DelayKernel2006::new(sample_rate, [d0, d1, d2, 0], fb, 0.0, 1.0);
        Self { front: Box::new(e), mid: None, back: Box::new(back) }
    }

    /// XG2.0 Wah serial: P1-3 delay, P4-7 distortion, P10 dry/wet, P11-14 wah
    /// (sensitivity/cutoff/resonance/release)
    fn new_wah(params: &[u16; 16], sample_rate: f32, overdrive: bool) -> Self {
        let mut wp = [0u16; 16];
        wp[crate::midi::effect_params::effect_obj::auto_wah_param::CUTOFF_FREQ_OFFSET] = params[11];
        wp[crate::midi::effect_params::effect_obj::auto_wah_param::RESONANCE] = params[12];
        wp[crate::midi::effect_params::effect_obj::auto_wah_param::DRY_WET] = 1;
        let mut wah = AutoWahEffect::new(sample_rate);
        wah.set_params(&wp, sample_rate);
        // distortion mid (P4-7)
        let mut fp = [0u16; 16];
        fp[distortion_param::DRIVE] = params[3];
        fp[distortion_param::OUTPUT_LEVEL] = params[4];
        fp[distortion_param::EQ_LOW_GAIN] = params[5];
        fp[distortion_param::EQ_MID_GAIN] = params[6];
        fp[distortion_param::DRY_WET] = 1;
        let mut e = DistortionEffect::new(sample_rate);
        e.set_params(&fp, !overdrive);
        // delay back (P1-3)
        let d0 = (params[0] & 0x3FFF) as usize;
        let fb = super::params::feedback_gain(params[1]);
        let back = DelayKernel2006::new(sample_rate, [d0, 0, 0, 0], fb, 0.0, 1.0);
        Self { front: Box::new(wah), mid: Some(Box::new(e)), back: Box::new(back) }
    }

    /// XG2.0 Compressor serial: P1-3 delay, P4-7 distortion, P10 dry/wet, P11-14 comp
    fn new_compressor(params: &[u16; 16], sample_rate: f32, overdrive: bool) -> Self {
        let mut cp = [0u16; 16];
        cp[crate::midi::effect_params::effect_obj::compressor_param::ATTACK] = params[10];
        cp[crate::midi::effect_params::effect_obj::compressor_param::RELEASE] = params[11];
        cp[crate::midi::effect_params::effect_obj::compressor_param::THRESHOLD] = params[12];
        cp[crate::midi::effect_params::effect_obj::compressor_param::RATIO] = params[13];
        let mut comp = CompressorEffect::new(sample_rate);
        comp.set_params(&cp, sample_rate);
        // distortion mid (P4-7)
        let mut fp = [0u16; 16];
        fp[distortion_param::DRIVE] = params[3];
        fp[distortion_param::OUTPUT_LEVEL] = params[4];
        fp[distortion_param::EQ_LOW_GAIN] = params[5];
        fp[distortion_param::EQ_MID_GAIN] = params[6];
        fp[distortion_param::DRY_WET] = 1;
        let mut e = DistortionEffect::new(sample_rate);
        e.set_params(&fp, !overdrive);
        // delay back (P1-3)
        let d0 = (params[0] & 0x3FFF) as usize;
        let fb = super::params::feedback_gain(params[1]);
        let back = DelayKernel2006::new(sample_rate, [d0, 0, 0, 0], fb, 0.0, 1.0);
        Self { front: Box::new(comp), mid: Some(Box::new(e)), back: Box::new(back) }
    }

    /// V-Distortion: P1-4 distortion params, P5 output level, P6-8 delay times,
    /// P9 feedback, P10 dry/wet, P11 delay mix
    fn new_vdistortion(params: &[u16; 16], sample_rate: f32) -> Self {
        let mut front = [0u16; 16];
        for i in 0..4 {
            front[i + 1] = params[i];
        }
        front[distortion_param::OUTPUT_LEVEL] = params[4]; // P5
        front[distortion_param::DRY_WET] = 1;
        let mut e = DistortionEffect::new(sample_rate);
        e.set_params(&front, true);
        let d0 = (params[5] & 0x3FFF) as usize;
        let d1 = (params[6] & 0x3FFF) as usize;
        let d2 = (params[7] & 0x3FFF) as usize;
        let fb = super::params::feedback_gain(params[8]);
        let back = DelayKernel2006::new(sample_rate, [d0, d1, d2, 0], fb, 0.0, 1.0);
        Self { front: Box::new(e), mid: None, back: Box::new(back) }
    }
}

impl EffectProcessor for SerialChain {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (fl, fr) = self.front.process(input);
        let (fl, fr) = match self.mid.as_mut() {
            Some(m) => m.process((fl, fr)),
            None => (fl, fr),
        };
        self.back.process((fl, fr))
    }
}

/// XG2.0 DynaFilter: LFO sweeps the LP filter cutoff
struct DynaFilterEffect {
    lfo_phase: u32,
    lfo_inc: u32,
    xor_tbl: [u32; 32],
    lpf: super::core::biquad::Biquad,
    base_cutoff: f32,
    sweep: f32,
    sample_rate: f32,
    dry: f32,
    wet: f32,
}

impl DynaFilterEffect {
    fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let mut xor_tbl = [0u32; 32];
        let mut x = 0x9e3779b9u32;
        for e in &mut xor_tbl {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            *e = x >> 16;
        }
        // XG Spec: P2 sensitivity (rate approx), P3 level offset, P4 resonance (depth)
        let rate_hz = super::params::lfo_freq(p16(params, 1));
        let (d, w) = dry_wet(p16(params, 10));
        Self {
            lfo_phase: 0,
            lfo_inc: ((rate_hz / sample_rate) * 8388608.0).max(1.0) as u32,
            xor_tbl,
            lpf: super::core::biquad::Biquad::new(),
            base_cutoff: 200.0 + p16(params, 2) as f32 * 100.0,
            sweep: 500.0 + p16(params, 3) as f32 * 100.0,
            sample_rate,
            dry: d,
            wet: w,
        }
    }
}

impl EffectProcessor for DynaFilterEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        self.lfo_phase = self.lfo_phase.wrapping_add(self.lfo_inc) & 0x7fffff;
        let p = self.lfo_phase ^ self.xor_tbl[(self.lfo_phase >> 0x16) as usize];
        let lfo = (p as f32 / 8388608.0) * 2.0 - 1.0;
        let cutoff = (self.base_cutoff + lfo * self.sweep).clamp(50.0, 16000.0);
        self.lpf = super::core::biquad::make_biquad(-6.0, cutoff, 0.707, false, self.sample_rate);
        let fl = self.lpf.tick(l);
        let fr = self.lpf.tick(r);
        (l * self.dry + fl * self.wet, r * self.dry + fr * self.wet)
    }
}

impl EffectProcessor for DelayKernel2006 {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        self.tick(input.0, input.1)
    }
}

/// XG2.0 Dyna envelope follower helper (XG Spec Dyna layout:
/// P1 sensitivity, P4 attack, P5 release, P8 threshold)
fn make_dyna_env(params: &[u16; 16], sample_rate: f32) -> super::xg20_effects::DynaEnv {
    super::xg20_effects::DynaEnv::new(
        1.0 + p16(params, 3) as f32 * 8.0,
        1.0 + p16(params, 4) as f32 * 40.0,
        p16(params, 7) as f32 / 127.0,
        1.0 + p16(params, 0) as f32 / 127.0,
        sample_rate,
    )
}

/// Dyna Chorus/Flanger: envelope drives the modulation depth
struct DynaChorusEffect {
    inner: super::chorus_effect::ChorusEffect,
    env: super::xg20_effects::DynaEnv,
}

impl EffectProcessor for DynaChorusEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let env = self.env.tick((input.0 + input.1) * 0.5);
        self.inner.set_dyna_env(env);
        self.inner.process(input)
    }
}

/// Dyna Phaser: envelope drives the allpass modulation
struct DynaPhaserEffect {
    inner: Phaser2006,
    env: super::xg20_effects::DynaEnv,
}

impl EffectProcessor for DynaPhaserEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let env = self.env.tick((input.0 + input.1) * 0.5);
        self.inner.set_dyna_env(env);
        self.inner.process(input)
    }
}

impl EffectProcessor for EarlyRef2006 {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let mono = (l + r) * 0.5;
        // 2006LE ER: mono kernel, L/R taps from the same ring
        let wet_l = self.tick(mono);
        (l * self.dry + wet_l * self.wet, r * self.dry + wet_l * self.wet)
    }
}

/// 2006LE CalcPhaser kernel (0x5584c): 5-stage allpass chain ×2 channels,
///   LFO = 23-bit phase + XOR table (same _g_slLFOCntMaskSawtooth as Chorus)
struct Phaser2006 {
    s_l: [f32; 5],
    s_r: [f32; 5],
    phase: u32,
    lfo_inc: u32,
    xor_tbl: [u32; 32],
    /// Allpass coefficient: center + depth×LFO (g ∈ (0,1))
    center_g: f32,
    depth_g: f32,
    fb: f32,
    dry: f32,
    wet: f32,
    /// Dyna envelope multiplier (XG2.0 DYNA PHASER)
    dyna_env: f32,
}

impl Phaser2006 {
    pub fn set_dyna_env(&mut self, env: f32) {
        self.dyna_env = env;
    }
    fn new(params: &[u16; 16], sample_rate: f32) -> Self {
        let mut xor_tbl = [0u32; 32];
        let mut x = 0x9e3779b9u32;
        for e in &mut xor_tbl {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            *e = x >> 16;
        }
        let lfo_hz = super::params::lfo_freq(p16(params, phaser_param::LFO_FREQ));
        let (d, w) = dry_wet(p16(params, phaser_param::DRY_WET));
        Self {
            s_l: [0.0; 5],
            s_r: [0.0; 5],
            phase: 0,
            lfo_inc: ((lfo_hz / sample_rate) * 8388608.0).max(1.0) as u32,
            xor_tbl,
            // PHASE_SHIFT_OFFSET(3) → center; LFO_DEPTH(2) → modulation width
            center_g: 0.3 + p16(params, phaser_param::PHASE_SHIFT_OFFSET) as f32 / 127.0 * 0.6,
            depth_g: p16(params, phaser_param::LFO_DEPTH) as f32 / 127.0 * 0.35,
            fb: super::params::feedback_gain(p16(params, phaser_param::FEEDBACK_LEVEL)),
            dry: d,
            wet: w,
            dyna_env: 1.0,
        }
    }

    /// 5-stage allpass chain (2006LE: y = g×s[i-1] + s[i] - g×y_prev)
    #[inline]
    fn allpass5(s: &mut [f32; 5], x: f32, g: f32) -> f32 {
        let mut y = x;
        for i in 0..5 {
            let ny = g * s[i] + if i > 0 { s[i - 1] } else { x } - g * y;
            s[i] = y;
            y = ny;
        }
        y
    }
}

impl EffectProcessor for Phaser2006 {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        // LFO (23-bit phase + XOR, every sample)
        self.phase = self.phase.wrapping_add(self.lfo_inc) & 0x7fffff;
        let p = self.phase ^ self.xor_tbl[(self.phase >> 0x16) as usize];
        let lfo = (p as f32 / 8388608.0) * 2.0 - 1.0;
        let env = self.dyna_env;
        let g_l = (self.center_g + self.depth_g * lfo * env).clamp(0.01, 0.99);
        let g_r = (self.center_g - self.depth_g * lfo * env * 0.7).clamp(0.01, 0.99);

        let fb_l = self.s_l[4] * self.fb * 0.5;
        let fb_r = self.s_r[4] * self.fb * 0.5;
        let mut out_l = Self::allpass5(&mut self.s_l, l + fb_l, g_l);
        let mut out_r = Self::allpass5(&mut self.s_r, r + fb_r, g_r);
        out_l = l * self.dry + out_l * self.wet;
        out_r = r * self.dry + out_r * self.wet;
        (out_l, out_r)
    }
}

/// Delay param (0-127) → sample count
fn delay_samples(v: u16, sample_rate: f32) -> f32 {
    crate::midi::effect_params::parameter_table::XG_DELAY_TIME_200MS_TABLE[v.min(127) as usize]
        / 1000.0
        * sample_rate
}

/// Build Variation effect (dispatch by type)
pub fn build_variation(
    effect_type: XGVariationType,
    params: &[u16; 16],
    sample_rate: f32,
) -> Box<dyn EffectProcessor> {
    use XGVariationType::*;
    match effect_type {
        NoEffect | Thru => Box::new(super::Thru),
        DelayLCR => Box::new(DelayLcr::new(params, sample_rate)),
        DelayLR => Box::new(DelayLr::new(params, sample_rate)),
        Echo => Box::new(EchoDelay::new(params, sample_rate)),
        CrossDelay => Box::new(CrossDelayEffect::new(params, sample_rate)),
        // 2006LE CalcChorusVar: reverb-like Variation types use the modulated
        // Chorus kernel (XG Variation reverb = modulated reverb, Chorus param layout)
        Hall1 | Hall2 | Room1 | Room2 | Room3 | Stage1 | Stage2 => {
            let mut ch = super::chorus_effect::ChorusEffect::new(sample_rate);
            ch.set_params(params);
            Box::new(ch)
        }
        // 2006LE CalcChorus3Var (Plate): 3-tap modulated kernel
        Plate => {
            let mut ch = super::chorus_effect::ChorusEffect::new(sample_rate);
            ch.set_params(params);
            ch.set_mode3();
            Box::new(ch)
        }
        ER1 | ER2 | GateReverb | ReverseGate => {
            let fb = feedback_gain(p16(params, 15));
            Box::new(EarlyRef2006::new(params, sample_rate, fb))
        }
        // Chorus/flanger family: reuse modulation delay kernel (param indices compatible with chorus_param)
        Chorus1 | Chorus2 | Chorus3 | Celeste3 | Celeste4
        | Flanger1 | Flanger2 | Flanger3 | Symphonic => {
            let mut ch = super::chorus_effect::ChorusEffect::new(sample_rate);
            ch.set_params(params);
            Box::new(ch)
        }
        // 2006LE CalcChorus3 (3-tap): Chorus4, Celeste1-2, Phaser1
        Chorus4 | Celeste1 | Celeste2 => {
            let mut ch = super::chorus_effect::ChorusEffect::new(sample_rate);
            ch.set_params(params);
            ch.set_mode3();
            Box::new(ch)
        }
        Phaser1 | Phaser2 => Box::new(Phaser2006::new(params, sample_rate)),
        // Modulation types
        Tremolo => build_modulation(ModEffectKind::Tremolo, params, sample_rate),
        AutoPan => build_modulation(ModEffectKind::AutoPan, params, sample_rate),
        RotarySpeaker => {
            build_modulation(ModEffectKind::RotarySpeaker, params, sample_rate)
        }
        // Distortion types
        Distortion => {
            let mut e = DistortionEffect::new(sample_rate);
            e.set_params(params, true);
            Box::new(e)
        }
        Overdrive => {
            let mut e = DistortionEffect::new(sample_rate);
            e.set_params(params, false);
            Box::new(e)
        }
        // XG2.0 serial: Distortion/Overdrive → Delay (tempo variants use plain delay, no tempo source)
        DistortionDelay | DistortionTempoDelay => {
            Box::new(SerialChain::new_distortion(params, sample_rate, true))
        }
        OverdriveDelay | OverdriveTempoDelay => {
            Box::new(SerialChain::new_distortion(params, sample_rate, false))
        }
        VDistortionHard | VDistortionSoft => {
            // V-Distortion (no delay): front params P1-4, output level P5
            let mut fp = [0u16; 16];
            for i in 0..4 {
                fp[i + 1] = params[i];
            }
            fp[distortion_param::OUTPUT_LEVEL] = params[4];
            fp[distortion_param::DRY_WET] = 1;
            let mut e = DistortionEffect::new(sample_rate);
            e.set_params(&fp, true);
            Box::new(e)
        }
        VDistortionHardDelay | VDistortionHardTempoDelay
        | VDistortionSoftDelay | VDistortionSoftTempoDelay => {
            // V-Distortion + delay: P1-4 + P5 output, P6-8 delay, P9 feedback
            Box::new(SerialChain::new_vdistortion(params, sample_rate))
        }
        AmpSimulator => {
            let mut e = AmpSimEffect::new(sample_rate);
            e.set_params(params);
            Box::new(e)
        }
        AuralExciter => {
            let mut e = AuralExciterEffect::new(sample_rate);
            e.set_params(params);
            Box::new(e)
        }
        // EQ types
        ThreeBandEQ => {
            let mut e = ThreeBandEqEffect::new();
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        TwoBandEQ => {
            let mut e = TwoBandEqEffect::new();
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        // Filter types
        AutoWah => {
            let mut e = AutoWahEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        // XG2.0 serial: Wah → Distortion (→ Delay) — P1-3 delay, P4-7 dist, P11-14 wah
        AutoWahDistortion | AutoWahOverdrive | WahDistortionDelay | WahOverdriveDelay
        | WahDistortionTempoDelay | WahOverdriveTempoDelay => {
            let overdrive = matches!(effect_type, AutoWahOverdrive | WahOverdriveDelay | WahOverdriveTempoDelay);
            Box::new(SerialChain::new_wah(params, sample_rate, overdrive))
        }
        // XG2.0 serial: Compressor → Distortion → Delay — P11-14 comp
        CompressorDistortionDelay | CompressorOverdriveDelay
        | CompressorDistortionTempoDelay | CompressorOverdriveTempoDelay => {
            let overdrive = matches!(effect_type, CompressorOverdriveDelay | CompressorOverdriveTempoDelay);
            Box::new(SerialChain::new_compressor(params, sample_rate, overdrive))
        }
        // XG2.0 serial: Distortion/Overdrive/AmpSim → 2WAY Rotary Speaker
        // (XG Spec: P1 rotor speed, P14 drive, P15 LPF, P16 output)
        DistortionTwoWayRotarySP | OverdriveTwoWayRotarySP | AmpSimTwoWayRotarySP => {
            let mut fp = [0u16; 16];
            fp[distortion_param::DRIVE] = params[13];        // P14 Drive
            fp[distortion_param::LPF_CUTOFF] = params[14];   // P15 LPF Cutoff
            fp[distortion_param::OUTPUT_LEVEL] = params[15]; // P16 Output
            fp[distortion_param::DRY_WET] = 1;
            let od = matches!(effect_type, OverdriveTwoWayRotarySP);
            let amp = matches!(effect_type, AmpSimTwoWayRotarySP);
            let mut e = DistortionEffect::new(sample_rate);
            e.set_params(&fp, !od && !amp);
            // rotary back: P1 rotor speed
            let mut rp = [0u16; 16];
            rp[crate::midi::effect_params::effect_obj::rotary_speaker_param::LFO_FREQ] = params[0];
            let mut rot = super::modulation_effects::RotarySpeakerEffect::new(sample_rate);
            rot.set_params(&rp, sample_rate);
            Box::new(SerialChain { front: Box::new(e), mid: None, back: Box::new(rot) })
        }
        TouchWah | TouchWahDist => {
            let mut e = TouchWahEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        // Dynamics types
        Compressor => {
            let mut e = CompressorEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        NoiseGate => {
            let mut e = NoiseGateEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        // Misc
        PitchChange => {
            let mut e = PitchChangeEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        Karaoke1 | Karaoke2 | Karaoke3 => {
            let mut e = KaraokeEffect::new();
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        // XG2.0 Dyna family (LFO-driven)
        DynaFilter => Box::new(DynaFilterEffect::new(params, sample_rate)),
        DynaFlanger => {
            let mut ch = super::chorus_effect::ChorusEffect::new(sample_rate);
            ch.set_params(params);
            let env = make_dyna_env(params, sample_rate);
            Box::new(DynaChorusEffect { inner: ch, env })
        }
        DynaPhaser => {
            let inner = Phaser2006::new(params, sample_rate);
            let env = make_dyna_env(params, sample_rate);
            Box::new(DynaPhaserEffect { inner, env })
        }
        DynaRingModulator => Box::new(super::xg20_effects::RingModEffect::new(params, sample_rate)),
        // XG2.0 misc effects (approximate, xg20_effects.rs)
        RingModulator => Box::new(super::xg20_effects::RingModEffect::new(params, sample_rate)),
        EnsembleDetune => Box::new(super::xg20_effects::EnsembleDetuneEffect::new(params, sample_rate)),
        Ambience => Box::new(super::xg20_effects::AmbienceEffect::new(params, sample_rate)),
        WideStereo => Box::new(super::xg20_effects::WideStereoEffect::new(params, sample_rate)),
        ThreeDManual => Box::new(super::xg20_effects::ThreeDEffect::new(params, sample_rate, false)),
        ThreeDAuto => Box::new(super::xg20_effects::ThreeDEffect::new(params, sample_rate, true)),
        VibeVibrate => Box::new(super::xg20_effects::VibeVibrateEffect::new(params, sample_rate)),
        LoFi => Box::new(super::xg20_effects::LoFiEffect::new(params, sample_rate)),
        Slice => Box::new(super::xg20_effects::SliceEffect::new(params, sample_rate)),
        Isolator => Box::new(super::xg20_effects::IsolatorEffect::new(params, sample_rate)),
        LowResolution => Box::new(super::xg20_effects::LowResEffect::new(params)),
        DigitalTurntable | DigitalScratch => {
            Box::new(super::xg20_effects::TurntableEffect::new(params, sample_rate))
        }
        MultiBandComp => {
            Box::new(super::xg20_effects::MultiBandCompEffect::new(params, sample_rate))
        }
        // Tempo/vocal flanger + dual rotor: reuse existing kernels (no tempo source)
        TempoFlanger | VFlanger => {
            let mut ch = super::chorus_effect::ChorusEffect::new(sample_rate);
            ch.set_params(params);
            Box::new(ch)
        }
        TempoPhaser => Box::new(Phaser2006::new(params, sample_rate)),
        DualRotorSpeaker1 | DualRotorSpeaker2 => {
            let mut rot = super::modulation_effects::RotarySpeakerEffect::new(sample_rate);
            rot.set_params(params, sample_rate);
            Box::new(rot)
        }
        // XG2.0 Harmony family: WSOLA shifters driven by active notes
        VocoderHarmony => {
            Box::new(super::harmony_effect::HarmonyEffect::new(
                params, sample_rate,
                super::harmony_effect::HarmonyKind::Chromatic,
                true,
            ))
        }
        ChordalHarmony => {
            Box::new(super::harmony_effect::HarmonyEffect::new(
                params, sample_rate,
                super::harmony_effect::HarmonyKind::Chordal,
                false,
            ))
        }
        DetuneHarmony => {
            Box::new(super::harmony_effect::HarmonyEffect::new(
                params, sample_rate,
                super::harmony_effect::HarmonyKind::Detune,
                false,
            ))
        }
        ChromaticHarmony => {
            Box::new(super::harmony_effect::HarmonyEffect::new(
                params, sample_rate,
                super::harmony_effect::HarmonyKind::Chromatic,
                false,
            ))
        }
        TalkingModulator => {
            Box::new(super::harmony_effect::TalkingModulatorEffect::new(params, sample_rate))
        }
        VoiceCancel => {
            let mut e = VoiceCancelEffect::new();
            e.set_params(params);
            Box::new(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delay_params() -> [u16; 16] {
        let mut p = [0u16; 16];
        p[delay_lcr_param::L_CH_DELAY] = 30;
        p[delay_lcr_param::R_CH_DELAY] = 40;
        p[delay_lcr_param::C_CH_DELAY] = 50;
        p[delay_lcr_param::FEEDBACK_DELAY] = 50;
        p[6] = 127; // C_CH_LEVEL: full center-line feedback
        p[10] = 1; // full wet
        p[15] = 60; // feedback
        p
    }

    #[test]
    fn delay_lcr_echoes_impulse() {
        let params = delay_params();
        let mut fx = DelayLcr::new(&params, 44100.0);
        let mut hits = vec![];
        for i in 0..44100 / 5 {
            let input = if i == 0 { (1.0, 1.0) } else { (0.0, 0.0) };
            let (l, _r) = fx.process(input);
            if l.abs() > 0.1 {
                hits.push(i);
            }
        }
        // L delay 30 → delay samples, at least 2 echoes (L + C lines)
        assert!(hits.len() >= 2, "hits={hits:?}");
        // First echo at L_CH_DELAY (T[30]=47.3ms≈2086)
        let first = hits[0];
        assert!(first > 1800 && first < 2400, "first={first}");
        // Second event = C line echo (T[50]=78.8ms≈3475)
        if hits.len() > 1 {
            let second = hits[1];
            assert!(second > 3200 && second < 3800, "second={second}");
        }
    }

    #[test]
    fn all_types_dispatch_without_panic() {
        use crate::midi::effect_params::variation_type::XGVariationType;
        // enumerate all enum values and build each effect
        let mut built = 0u32;
        for v in 0u16..=0xFFFF {
            if let Ok(t) = XGVariationType::try_from(v) {
                let _fx = build_variation(t, &[0; 16], 44100.0);
                built += 1;
            }
        }
        assert!(built >= 95, "only {built} variation types dispatchable");
    }

    #[test]
    fn thru_passthrough() {
        let mut fx = build_variation(XGVariationType::Thru, &[0; 16], 44100.0);
        let out = fx.process((0.5, -0.5));
        assert!((out.0 - 0.5).abs() < 1e-6);
        assert!((out.1 + 0.5).abs() < 1e-6);
    }

    #[test]
    fn hall_family_uses_reverb() {
        let mut fx = build_variation(XGVariationType::Hall1, &[0; 16], 44100.0);
        let mut early = 0.0f32;
        for i in 0..5000 {
            let input = if i == 0 { (1.0, 1.0) } else { (0.0, 0.0) };
            let (l, _) = fx.process(input);
            early = early.max(l.abs());
        }
        assert!(early > 0.05, "early={early}");
    }
}
