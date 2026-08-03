use super::fast_sine::SINE_TABLE;
use super::phase_table::INIT_PHASE_TABLE;
use super::wave_type::{WaveType, WaveVariation};

use crate::utils::random_xorshift;

const FULL_PERIOD: u32 = 0x100000;
const PERIOD_MASK: u32 = FULL_PERIOD - 1;
const HALF_PERIOD: i32 = (FULL_PERIOD / 2) as i32;
const QUAD_PERIOD: i32 = HALF_PERIOD / 2;

#[derive(Debug)]
pub enum LFORunningMode {
    Key,
    Free,
}

#[derive(Debug)]
struct LFOParams {
    pub output: f32,
    pub depth: f32,
    pub offset: f32,
}

impl LFOParams {
    pub fn set_output(&mut self, value: f32) {
        self.output = value * self.depth + self.offset;
    }

    pub fn new() -> Self {
        Self {
            offset: 0.0,
            depth: 0.5,
            output: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct LFO {
    pub enable: bool,
    pub runing_mode: LFORunningMode,
    pub wave_type: WaveType,
    // range: 0x0 - 0xFFFFF
    pub dds_phase_accumulator: u32,
    pub random_state: u32,

    pub pitch: LFOParams,
    pub amp: LFOParams,
    pub lpf: LFOParams,
    pub hpf: LFOParams,
}

impl LFO {
    pub fn new() -> Self {
        Self {
            enable: true,
            runing_mode: LFORunningMode::Free,
            wave_type: WaveType::Sine,
            dds_phase_accumulator: 0,
            random_state: 0,
            pitch: LFOParams::new(),
            amp: LFOParams::new(),
            lpf: LFOParams::new(),
            hpf: LFOParams::new(),
        }
    }

    pub fn update_accumulator(&mut self, freq: f32, samples_per_block: u32, sample_rate: u32) {
        let freq_inc = (freq * FULL_PERIOD as f32) as u32 * samples_per_block / sample_rate;
        self.refresh_accumulator(freq_inc);
    }

    fn set_accumulator(&mut self, deg: usize, freq_inc: u32) {
        match self.runing_mode {
            LFORunningMode::Key => {
                self.dds_phase_accumulator = degree_to_dds(INIT_PHASE_TABLE[deg as usize])
            }
            LFORunningMode::Free => {
                self.refresh_accumulator(freq_inc);
            }
        }
    }

    #[inline(always)]
    fn refresh_accumulator(&mut self, freq_inc: u32) {
        self.dds_phase_accumulator += freq_inc;
        self.dds_phase_accumulator &= PERIOD_MASK;
    }

    pub fn make_wave(&mut self) {
        if !self.enable {
            return;
        }
        use WaveType::*;
        use WaveVariation::*;
        match self.wave_type {
            Sine => self.sine_wave(),
            Saw => self.saw_wave(Normal),
            Square => self.square_wave(Normal),
            Triangle => self.triangle_wave(Normal),
            Random => self.random_wave(Normal),

            SawEG => self.saw_wave(EG),
            TriangleEG => self.triangle_wave(EG),
            SquareEG => self.square_wave(EG),
            RandomEG => self.random_wave(EG),

            SawAlt => self.saw_wave(Alt),
            TriangleAlt => self.triangle_wave(Alt),
            SquareAlt => self.square_wave(Alt),
            RandomAlt => self.random_wave(Alt),
        }
    }

    fn set_params(&mut self, value: f32) {
        self.pitch.set_output(value);
        self.amp.set_output(value);
        self.lpf.set_output(value);
    }

    fn sine_wave(&mut self) {
        let idx = (self.dds_phase_accumulator >> 8) as usize;
        let value = SINE_TABLE[idx & 0x0FFF];

        self.set_params(value);
    }

    fn square_wave(&mut self, is_var: WaveVariation) {
        use WaveVariation::*;
        if is_var == EG {
            self.lpf.offset = 0.5;
        }

        let phase = self.dds_phase_accumulator;
        make_square_wave(phase, &mut self.pitch);
        make_square_wave(phase, &mut self.amp);
        make_square_wave(phase, &mut self.lpf);
        if is_var == Normal {
            make_square_wave(phase, &mut self.hpf);
        }
    }

    fn saw_wave(&mut self, is_var: WaveVariation) {
        use WaveVariation::*;
        if is_var == EG {
            self.lpf.offset = 0.5;
        }

        let phase = self.dds_phase_accumulator;
        make_saw_wave(phase, &mut self.pitch);
        make_saw_wave(phase, &mut self.amp);
        make_saw_wave(phase, &mut self.lpf);
        if is_var == Normal {
            make_saw_wave(phase, &mut self.hpf);
        }
    }

    fn triangle_wave(&mut self, is_var: WaveVariation) {
        use WaveVariation::*;
        if is_var == EG {
            self.lpf.offset = 0.5;
        }

        let phase = self.dds_phase_accumulator;
        make_triangle_wave(phase, &mut self.pitch);
        make_triangle_wave(phase, &mut self.amp);
        make_triangle_wave(phase, &mut self.lpf);

        if is_var == Normal {
            make_triangle_wave(phase, &mut self.hpf);
        }
    }

    fn random_wave(&mut self, is_var: WaveVariation) {
        use WaveVariation::*;
        if is_var == EG {
            self.lpf.offset = 0.5;
        }

        let value: f32 = random_xorshift(&mut self.random_state);

        self.pitch.set_output(value);
        self.amp.set_output(value);
        self.lpf.set_output(value);

        if is_var == Normal {
            self.hpf.set_output(value);
        }
    }
}

#[inline(always)]
fn degree_to_dds(deg: u16) -> u32 {
    ((deg as u32) << 20) / 0x168
}

#[inline(always)]
fn make_square_wave(phase: u32, p: &mut LFOParams) {
    if (phase & (HALF_PERIOD as u32)) == 0 {
        p.output = p.offset;
    } else {
        p.output = p.depth;
    }
}

#[inline(always)]
fn make_saw_wave(phase: u32, p: &mut LFOParams) {
    let raw = (phase + p.offset as u32) & PERIOD_MASK;
    let out = ((raw as i32) - HALF_PERIOD) as f32;

    p.set_output(out);
}

#[inline(always)]
fn make_triangle_wave(phase: u32, p: &mut LFOParams) {
    let mut raw = ((phase + p.offset as u32) & PERIOD_MASK) as i32;
    if (raw & HALF_PERIOD) != 0 {
        raw ^= PERIOD_MASK as i32;
    }
    let out = (raw - QUAD_PERIOD) as f32;
    p.set_output(out);
}
