/// Modulation effects: Tremolo / AutoPan / RotarySpeaker
///
/// All based on LFO (fast_sin) modulating gain/pan, params indexed via effect_obj const
use crate::fast_sine::{fast_cos, fast_sin};
use crate::midi::effect_params::effect_obj::{
    auto_pan_param, rotary_speaker_param, tremolo_param,
};

use super::core::eq_chain::EqChain;
use super::params::{dry_wet, lfo_freq, p16};
use super::EffectProcessor;

/// LFO state (sine, phase 0-1)
struct Lfo {
    phase: f32,
    freq: f32,
    sample_rate: f32,
}

impl Lfo {
    fn new(sample_rate: f32) -> Self {
        Self { phase: 0.0, freq: 1.0, sample_rate }
    }

    #[inline]
    fn tick(&mut self) -> f32 {
        self.phase += self.freq / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        fast_sin(self.phase * std::f32::consts::PI * 2.0)
    }
}

// ──────────────────────────── Tremolo ────────────────────────────
pub struct TremoloEffect {
    lfo_l: Lfo,
    am_depth: f32,
    /// PM depth: LFO → delay modulation (XG Spec Table #2, ms → samples)
    pm_samples: f32,
    /// R channel delay line for PM
    delay: super::core::delay::DelayLine,
    input_mono: bool,
    /// 3-band EQ (LOW 6/7, HIGH 8/9, MID 11/12/13)
    eq: EqChain,
}

impl TremoloEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            lfo_l: Lfo::new(sample_rate),
            am_depth: 0.0,
            pm_samples: 0.0,
            delay: super::core::delay::DelayLine::new(4096),
            input_mono: false,
            eq: EqChain::new(),
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.lfo_l.freq = lfo_freq(p16(params, tremolo_param::LFO_FREQ));
        self.am_depth = p16(params, tremolo_param::AM_DEPTH) as f32 / 127.0;
        // PM_DEPTH: LFO → delay modulation (XG Spec Table #2, ms → samples)
        let pm_ms = crate::midi::effect_params::parameter_table::XG_MODULATION_DELAY_OFFSET_TABLE
            [p16(params, tremolo_param::PM_DEPTH).min(127) as usize];
        self.pm_samples = pm_ms / 1000.0 * sample_rate;
        self.input_mono = p16(params, tremolo_param::INPUT_MODE) == 0;
        // EQ (chorus layout)
        self.eq.set_chorus_layout(params, sample_rate);
    }
}

impl EffectProcessor for TremoloEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let (l, r) = if self.input_mono {
            let m = (l + r) * 0.5;
            (m, m)
        } else {
            (l, r)
        };
        let llfo = self.lfo_l.tick();
        // PM: LFO → R channel delay modulation (true tremolo pitch shift)
        let rlfo = fast_sin((self.lfo_l.phase % 1.0) * std::f32::consts::PI * 2.0);
        let delay = (1.0 + (1.0 - rlfo) * 0.5 * self.pm_samples + self.pm_samples * 0.5).max(1.0);
        let r_delayed = self.delay.tick(r, delay);
        let am_l = 1.0 + llfo * self.am_depth;
        let am_r = 1.0 + rlfo * self.am_depth;
        (self.eq.tick(l * am_l), self.eq.tick(r_delayed * am_r))
    }
}

// ──────────────────────────── Auto Pan ────────────────────────────
pub struct AutoPanEffect {
    lfo: Lfo,
    lr_depth: f32,
    /// Front/rear depth (F_R_DEPTH): volume swing amplitude
    fr_depth: f32,
    /// Pan direction: 0=L→R, 1=L+R alternating, 2=R→L
    direction: u8,
    /// 3-band EQ
    eq: EqChain,
}

impl AutoPanEffect {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            lfo: Lfo::new(sample_rate),
            lr_depth: 0.0,
            fr_depth: 0.0,
            direction: 0,
            eq: EqChain::new(),
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        self.lfo.freq = lfo_freq(p16(params, auto_pan_param::LFO_FREQ));
        self.lr_depth = p16(params, auto_pan_param::L_R_DEPTH) as f32 / 127.0;
        self.fr_depth = p16(params, auto_pan_param::F_R_DEPTH) as f32 / 127.0;
        self.direction = p16(params, auto_pan_param::PAN_DIRECTION).min(2) as u8;
        self.eq.set_chorus_layout(params, sample_rate);
    }
}

impl EffectProcessor for AutoPanEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        let lfo = self.lfo.tick();
        let depth = self.lr_depth;
        let (gl, gr) = match self.direction {
            0 => {
                // L→R: single swing
                let t = (lfo + 1.0) * 0.5;
                let ang = t * std::f32::consts::FRAC_PI_2;
                (fast_sin(ang), fast_cos(ang))
            }
            1 => {
                // L+R alternating: 0→L strong / 1→R strong
                let t = (lfo + 1.0) * 0.5;
                (t, 1.0 - t)
            }
            _ => {
                // R→L
                let t = (lfo + 1.0) * 0.5;
                let ang = t * std::f32::consts::FRAC_PI_2;
                (fast_cos(ang), fast_sin(ang))
            }
        };
        // Depth: 1.0 = full swing, 0 = static (center)
        let g = |v: f32| 1.0 - depth + depth * v;
        // F_R_DEPTH: front/rear swing → volume modulation (synced with LFO)
        let fr = 1.0 - self.fr_depth * 0.5 + self.fr_depth * 0.5 * lfo;
        (self.eq.tick(l * g(gl) * fr), self.eq.tick(r * g(gr) * fr))
    }
}

// ──────────────────────── Rotary Speaker ─────────────────────────
/// 2006LE CalcRotarySP kernel (0x56288): 4 modulated delay lines + dual-speed XOR LFOs
pub struct RotarySpeakerEffect {
    ring: Box<[f32; 131072]>,
    idx: usize,
    /// Dual LFO phases (23-bit) + increments
    phase_slow: u32,
    phase_fast: u32,
    inc_slow: u32,
    inc_fast: u32,
    /// 4 LFO outputs: refs (phase offsets), centers, depths, smoothing states
    refs: [u32; 4],
    centers: [f32; 4],
    depths: [f32; 4],
    smooth: [f32; 4],
    lfo_state: [f32; 4],
    xor_tbl: [u32; 32],
    /// Delay taps (read offsets) + write offsets
    tap: [usize; 4],
    w: [usize; 2],
    /// Output mixes (L: [in, tap1, tap3, tap4], R: [in, tap2, tap4, tap3])
    out_l: [f32; 4],
    out_r: [f32; 4],
    /// LFO block offsets for the current 4-sample block
    off: [i32; 4],
    lfo_tick: u32,
    frac: [f32; 4],
    amp: [f32; 4],
    dry: f32,
    wet: f32,
}

impl RotarySpeakerEffect {
    pub fn new(sample_rate: f32) -> Self {
        let mut xor_tbl = [0u32; 32];
        let mut x = 0x9e3779b9u32;
        for e in &mut xor_tbl {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            *e = x >> 16;
        }
        let _ = sample_rate;
        Self {
            ring: Box::new([0.0; 131072]),
            idx: 0,
            phase_slow: 0,
            phase_fast: 0,
            inc_slow: 0,
            inc_fast: 0,
            refs: [0, 0x140000, 0x500000, 0x280000],
            centers: [0.5, 0.5, 0.5, 0.5],
            depths: [0.0; 4],
            smooth: [0.05; 4],
            lfo_state: [0.0; 4],
            xor_tbl,
            tap: [1600, 1680, 1760, 1840],
            w: [0, 8],
            out_l: [0.0, 0.5, 0.25, 0.25],
            out_r: [0.0, 0.5, 0.25, 0.25],
            off: [0; 4],
            frac: [0.0; 4],
            amp: [1.0; 4],
            lfo_tick: 0,
            dry: 1.0,
            wet: 1.0,
        }
    }

    pub fn set_params(&mut self, params: &[u16; 16], sample_rate: f32) {
        let hz = lfo_freq(p16(params, rotary_speaker_param::LFO_FREQ));
        self.inc_slow = ((hz / sample_rate) * 8388608.0).max(1.0) as u32;
        // fast rotor ≈ 3.5× slow horn
        self.inc_fast = (((hz * 3.5) / sample_rate) * 8388608.0).max(1.0) as u32;
        let depth = p16(params, rotary_speaker_param::LFO_DEPTH) as f32 / 127.0;
        self.depths = [depth * 120.0, depth * 120.0, depth * 90.0, depth * 90.0];
        let (d, w) = dry_wet(p16(params, rotary_speaker_param::DRY_WET));
        self.dry = d;
        self.wet = w;
    }

    /// LFO update (every 4 samples): 23-bit phase + XOR table, ×0.0625 scale
    fn update_lfo(&mut self) {
        let ps = (self.phase_slow + self.inc_slow) & 0x7fffff;
        self.phase_slow = ps;
        let pf = (self.phase_fast + self.inc_fast) & 0x7fffff;
        self.phase_fast = pf;
        for i in 0..4 {
            let (phase, ref_off) = if i < 2 { (ps, self.refs[i]) } else { (pf, self.refs[i]) };
            let d = phase.wrapping_sub(ref_off) & 0x7fffff;
            let v = ((d ^ self.xor_tbl[(d >> 0x16) as usize]) as f32 * 0.0625 - self.centers[i])
                * self.depths[i]
                + self.lfo_state[i] * self.smooth[i];
            self.lfo_state[i] = v;
            let s = (v - self.centers[i]) * 4.0;
            let si = s.floor();
            self.off[i] = si as i32;
            self.frac[i] = s - si;
            self.amp[i] = 1.0 + v * 0.02;
        }
    }
}

impl EffectProcessor for RotarySpeakerEffect {
    fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (l, r) = input;
        self.lfo_tick = self.lfo_tick.wrapping_add(1);
        if self.lfo_tick & 3 == 3 {
            self.update_lfo();
        }
        let m = 131071usize;
        let idx = self.idx;
        // 4 modulated taps (linear interp)
        let mut t = [0.0f32; 4];
        for i in 0..4 {
            let pos = (idx as isize + self.tap[i] as isize + self.off[i] as isize) & m as isize;
            let pos = pos as usize;
            let v0 = self.ring[pos];
            let v1 = self.ring[(pos + 1) & m];
            t[i] = (v0 + (v1 - v0) * self.frac[i]) * self.amp[i];
        }
        // Writes (2006LE): L = R_fb×a + L_in, R = R_fb×b
        self.ring[(idx + self.w[0]) & m] = l + t[1] * 0.3;
        self.ring[(idx + self.w[1]) & m] = r * 0.9 + t[0] * 0.1;
        self.idx = (self.idx + 131071) & m;
        // Outputs (2006LE cross mix)
        let ol = l * self.dry + (t[0] * self.out_l[1] + t[2] * self.out_l[2] + t[3] * self.out_l[3]) * self.wet;
        let or_ = r * self.dry + (t[1] * self.out_r[1] + t[3] * self.out_r[2] + t[2] * self.out_r[3]) * self.wet;
        (ol, or_)
    }
}

// ──────────────────────────── Builder ────────────────────────────
pub enum ModEffectKind {
    Tremolo,
    AutoPan,
    RotarySpeaker,
}

pub fn build_modulation(kind: ModEffectKind, params: &[u16; 16], sample_rate: f32) -> Box<dyn EffectProcessor> {
    match kind {
        ModEffectKind::Tremolo => {
            let mut e = TremoloEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        ModEffectKind::AutoPan => {
            let mut e = AutoPanEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
        ModEffectKind::RotarySpeaker => {
            let mut e = RotarySpeakerEffect::new(sample_rate);
            e.set_params(params, sample_rate);
            Box::new(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::effect_params::effect_obj::tremolo_param;

    #[test]
    fn tremolo_amplitude_modulates() {
        let mut e = TremoloEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[tremolo_param::LFO_FREQ] = 20; // ~3Hz
        p[tremolo_param::AM_DEPTH] = 127; // full depth
        p[tremolo_param::INPUT_MODE] = 1;
        // EQ 0dB (64) passthrough
        for i in [tremolo_param::EQ_LOW_GAIN, tremolo_param::EQ_HIGH_GAIN, tremolo_param::EQ_MID_GAIN] {
            p[i] = 64;
        }
        e.set_params(&p, 44100.0);
        // Constant input → output amplitude swings with LFO
        let mut min: f32 = f32::MAX;
        let mut max: f32 = 0.0;
        for _ in 0..44100 {
            let (l, _) = e.process((1.0, 1.0));
            min = min.min(l);
            max = max.max(l);
        }
        // Full depth: swing approaches 0..2
        assert!(max > 1.5, "max={max}");
        assert!(min < 0.5, "min={min}");
    }

    #[test]
    fn auto_pan_swings_stereo() {
        let mut e = AutoPanEffect::new(44100.0);
        let mut p = [0u16; 16];
        p[auto_pan_param::LFO_FREQ] = 20;
        p[auto_pan_param::L_R_DEPTH] = 127;
        for i in [auto_pan_param::EQ_LOW_GAIN, auto_pan_param::EQ_HIGH_GAIN, auto_pan_param::EQ_MID_GAIN] {
            p[i] = 64;
        }
        e.set_params(&p, 44100.0);
        // Stereo input → pan swings, both sides have output and energy is approximately conserved
        let mut l_sum = 0.0f32;
        let mut r_sum = 0.0f32;
        for _ in 0..44100 {
            let (l, r) = e.process((1.0, 1.0));
            l_sum += l;
            r_sum += r;
        }
        assert!(r_sum > 0.3 * l_sum, "r_sum={r_sum} l_sum={l_sum}");
        assert!(l_sum > 0.3 * r_sum, "r_sum={r_sum} l_sum={l_sum}");
    }
}
