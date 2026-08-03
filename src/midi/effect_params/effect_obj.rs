// XG Effect Parameter ID Table (XG Spec v1.26)
//
// Parameter indices extracted from Yamaha XG Specification v1.26.
// Each effect has 16 parameter slots (index 0-15), but the ID design is a masterpiece
// of anti-human engineering: the same index means completely different things
// depending on which effect you're looking at.
//
// For example, index 3 is DELAY_OFFSET in Chorus, FEEDBACK_LEVEL in Flanger,
// LPF_CUTOFF in Distortion, and HPF_CUTOFF in Plate Reverb.
// Even the number of active parameters varies wildly -- some effects use all 15,
// others use only 3. Yamaha's engineers must have thought programmers needed
// more challenges in life.
//
// Even worse, the Delay family had to split into two common parameter sets
// (_delay_common_param1/2) because they couldn't even reuse a single layout.
// In short, this file is a monument to Yamaha's engineers' commitment to
// making every developer's life harder.

pub const XG_EFFECT_INPUT_MODE_MONO: u8 = 0;
pub const XG_EFFECT_INPUT_MODE_STEREO: u8 = 1;

pub mod chorus_param {
    pub const LFO_FREQ: usize = 1;
    pub const LFO_PM_DEPTH: usize = 2;
    pub const FEEDBACK_LEVEL: usize = 3;
    pub const DELAY_OFFSET: usize = 4;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;
    pub const DRY_WET: usize = 10;

    pub const EQ_MID_FREQ: usize = 11;
    pub const EQ_MID_GAIN: usize = 12;
    pub const EQ_MID_WIDTH: usize = 13;
    pub const LFO_AM_DEPTH: usize = 14;
    pub const INPUT_MODE: usize = 15;
}

pub mod celeste_param {
    pub use super::chorus_param::*;
}

pub mod flanger_param {
    pub const LFO_FREQ: usize = 1;
    pub const LFO_DEPTH: usize = 2;
    pub const FEEDBACK_LEVEL: usize = 3;
    pub const DELAY_OFFSET: usize = 4;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;
    pub const DRY_WET: usize = 10;

    pub const EQ_MID_FREQ: usize = 11;
    pub const EQ_MID_GAIN: usize = 12;
    pub const EQ_MID_WIDTH: usize = 13;
    pub const LFO_PHASE_DIFFERENCE: usize = 14;
    pub const INPUT_MODE: usize = 15;
}

pub mod symphonic_param {
    pub const LFO_FREQ: usize = 1;
    pub const LFO_DEPTH: usize = 2;
    pub const DELAY_OFFSET: usize = 3;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;
    pub const DRY_WET: usize = 10;

    pub const EQ_MID_FREQ: usize = 11;
    pub const EQ_MID_GAIN: usize = 12;
    pub const EQ_MID_WIDTH: usize = 13;
}

pub mod rotary_speaker_param {
    pub const LFO_FREQ: usize = 1;
    pub const LFO_DEPTH: usize = 2;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;
    pub const DRY_WET: usize = 10;

    pub const EQ_MID_FREQ: usize = 11;
    pub const EQ_MID_GAIN: usize = 12;
    pub const EQ_MID_WIDTH: usize = 13;
}

pub mod tremolo_param {
    pub const LFO_FREQ: usize = 1;
    pub const AM_DEPTH: usize = 2;
    pub const PM_DEPTH: usize = 3;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;

    pub const EQ_MID_FREQ: usize = 11;
    pub const EQ_MID_GAIN: usize = 12;
    pub const EQ_MID_WIDTH: usize = 13;
    pub const LFO_PHASE_DIFFERENCE: usize = 14;
    pub const INPUT_MODE: usize = 15;
}

pub mod auto_pan_param {
    pub const LFO_FREQ: usize = 1;
    pub const L_R_DEPTH: usize = 2;
    pub const F_R_DEPTH: usize = 3;
    pub const PAN_DIRECTION: usize = 4;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;

    pub const EQ_MID_FREQ: usize = 11;
    pub const EQ_MID_GAIN: usize = 12;
    pub const EQ_MID_WIDTH: usize = 13;
}

pub mod phaser_param {
    pub const LFO_FREQ: usize = 1;
    pub const LFO_DEPTH: usize = 2;
    pub const PHASE_SHIFT_OFFSET: usize = 3;
    pub const FEEDBACK_LEVEL: usize = 4;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;
    pub const DRY_WET: usize = 10;
}

pub mod distortion_param {
    pub const DRIVE: usize = 1;
    pub const EQ_LOW_FREQ: usize = 2;
    pub const EQ_LOW_GAIN: usize = 3;
    pub const LPF_CUTOFF: usize = 4;
    pub const OUTPUT_LEVEL: usize = 5;

    pub const EQ_MID_FREQ: usize = 7;
    pub const EQ_MID_GAIN: usize = 8;
    pub const EQ_MID_WIDTH: usize = 9;
    pub const DRY_WET: usize = 10;
    pub const EDGE: usize = 11;
}

pub mod overdrive_param {
    pub use super::distortion_param::*;
}

pub mod guitar_amp_simulator_param {
    pub const DRIVE: usize = 1;
    pub const AMP_TYPE: usize = 2;
    pub const LPF_CUTOFF: usize = 3;
    pub const OUTPUT_LEVEL: usize = 4;
    pub const DRY_WET: usize = 10;
    pub const EDGE: usize = 11;
}

// mono eq
pub mod three_band_eq_param {
    pub const EQ_LOW_GAIN: usize = 1;
    pub const EQ_MID_FREQ: usize = 2;
    pub const EQ_MID_GAIN: usize = 3;
    pub const EQ_MID_WIDTH: usize = 4;
    pub const EQ_HIGH_GAIN: usize = 5;
    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_HIGH_FREQ: usize = 7;
}

// stereo eq
pub mod two_band_eq_param {
    pub const EQ_LOW_FREQ: usize = 1;
    pub const EQ_LOW_GAIN: usize = 2;
    pub const EQ_HIGH_FREQ: usize = 3;
    pub const EQ_HIGH_GAIN: usize = 4;

    pub const EQ_MID_FREQ: usize = 11;
    pub const EQ_MID_GAIN: usize = 12;
    pub const EQ_MID_WIDTH: usize = 13;
}

pub mod auto_wah_param {
    pub const LFO_FREQ: usize = 1;
    pub const LFO_DEPTH: usize = 2;
    pub const CUTOFF_FREQ_OFFSET: usize = 3;
    pub const RESONANCE: usize = 4;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;
    pub const DRY_WET: usize = 10;

    pub const DRIVE: usize = 11;
}

pub mod plate_param {
    pub const REVERB_TIME: usize = 1;
    pub const DIFFUSION: usize = 2;
    pub const INIT_DELAY: usize = 3;
    pub const HPF_CUTOFF: usize = 4;
    pub const LPF_CUTOFF: usize = 5;

    pub const DRY_WET: usize = 10;
    pub const REV_DELAY: usize = 11;
    pub const DENCITY: usize = 12;
    pub const REV_ER_BALANCE: usize = 13;
    pub const HIGH_DAMP: usize = 14;
    pub const FEEDBACK_LEVEL: usize = 15;
}

pub mod hall1_param {
    pub use super::plate_param::*;
}

pub mod hall2_param {
    pub use super::plate_param::*;
}

pub mod room1_param {
    pub use super::plate_param::*;
}

pub mod room2_param {
    pub use super::plate_param::*;
}

pub mod room3_param {
    pub use super::plate_param::*;
}

pub mod stage1_param {
    pub use super::plate_param::*;
}

pub mod stage2_param {
    pub use super::plate_param::*;
}

mod _delay_common_param1 {
    pub const DRY_WET: usize = 10;
    pub const HPF_CUTOFF: usize = 11;
    pub const LPF_CUTOFF: usize = 12;
    pub const EQ_LOW_FREQ: usize = 13;
    pub const EQ_LOW_GAIN: usize = 14;
    pub const EQ_HIGH_FREQ: usize = 15;
    pub const EQ_HIGH_GAIN: usize = 16;
}

mod _delay_common_param2 {
    pub const FEEDBACK_LEVEL: usize = 5;
    pub const C_CH_LEVEL: usize = 6;
    pub const HIGH_DAMP: usize = 7;
}

pub mod delay_lcr_param {
    pub const L_CH_DELAY: usize = 1;
    pub const R_CH_DELAY: usize = 2;
    pub const C_CH_DELAY: usize = 3;
    pub const FEEDBACK_DELAY: usize = 4;
    pub use super::_delay_common_param1::*;
    pub use super::_delay_common_param2::*;
}

pub mod delay_lr_param {
    pub const L_CH_DELAY: usize = 1;
    pub const R_CH_DELAY: usize = 2;
    pub const FEEDBACK_DELAY_1: usize = 3;
    pub const FEEDBACK_DELAY_2: usize = 4;
    pub use super::_delay_common_param1::*;
    pub use super::_delay_common_param2::*;
}

pub mod echo_param {
    pub const L_CH_DELAY_1: usize = 1;
    pub const L_CH_FEEDBACK_LEVEL: usize = 2;
    pub const R_CH_DELAY_1: usize = 3;
    pub const R_CH_FEEDBACK_LEVEL: usize = 4;
    pub const HIGH_DAMP: usize = 5;
    pub const L_CH_DELAY_2: usize = 6;
    pub const R_CH_DELAY_2: usize = 7;
    pub const DELAY2_LEVEL: usize = 8;

    pub use super::_delay_common_param1::*;
}

pub mod cross_delay_param {
    pub const L_R_DELAY: usize = 1;
    pub const R_L_DELAY: usize = 2;
    pub const FEEDBACK_LEVEL: usize = 3;
    pub const INPUT_SELECT: usize = 4;
    pub const HIGH_DAMP: usize = 5;

    pub use super::_delay_common_param1::*;
}

pub mod early_ref1_param {
    pub const TYPE: usize = 1;
    pub const ROOM_SIZE: usize = 2;
    pub const DIFFUSION: usize = 3;
    pub const INIT_DELAY: usize = 4;
    pub const FEEDBACK_LEVEL: usize = 5;
    pub const HPF_CUTOFF: usize = 6;
    pub const LPF_CUTOFF: usize = 7;

    pub const DRY_WET: usize = 10;
    pub const LIVENESS: usize = 11;
    pub const DENSITY: usize = 12;
    pub const HIGH_DAMP: usize = 13;
}

pub mod early_ref2_param {
    pub use super::early_ref1_param::*;
}

pub mod gate_reverb_param {
    pub use super::early_ref1_param::*;
}

pub mod reverse_gate_param {
    pub use super::early_ref1_param::*;
}

// Option effect type

pub mod pitch_change_param {
    pub const PITCH: usize = 1;
    pub const INIT_DEALY: usize = 2;
    pub const FINE_1: usize = 3;
    pub const FINE_2: usize = 4;
    pub const FEEDBACK_GAIN: usize = 5;

    pub const DRY_WET: usize = 10;
    pub const PAN_1: usize = 11;
    pub const OUTPUT_LEVEL_1: usize = 12;
    pub const PAN_2: usize = 13;
    pub const OUTPUT_LEVEL_2: usize = 14;
}

pub mod aural_exiceter_param {
    pub const HPF_CUTOFF: usize = 1;
    pub const DRIVE: usize = 2;
    pub const MIX_LEVEL: usize = 3;
}

pub mod touch_wah_param {
    pub const SENSITIVITY: usize = 1;
    pub const CUTOFF_FREQ_OFFSET: usize = 2;
    pub const RESONANCE: usize = 3;

    pub const EQ_LOW_FREQ: usize = 6;
    pub const EQ_LOW_GAIN: usize = 7;
    pub const EQ_HIGH_FREQ: usize = 8;
    pub const EQ_HIGH_GAIN: usize = 9;
    pub const DRY_WET: usize = 10;
    pub const DRIVE: usize = 11;
}

pub mod touch_wah_dist_param {
    pub use super::touch_wah_param::*;
}

pub mod compressor_param {
    pub const ATTACK: usize = 1;
    pub const RELEASE: usize = 2;
    pub const THRESHOLD: usize = 3;
    pub const RATIO: usize = 4;
    pub const OUTPUT_LEVEL: usize = 5;
}

pub mod noise_gate_param {
    pub const ATTACK: usize = 1;
    pub const RELEASE: usize = 2;
    pub const THRESHOLD: usize = 3;
    pub const OUTPUT_LEVEL: usize = 4;

    pub const RATIO: usize = 11;
}

pub mod white_room_param {
    pub const REVERB_TIME: usize = 1;
    pub const DIFFUSION: usize = 2;
    pub const INIT_DELAY: usize = 3;
    pub const HPF_CUTOFF: usize = 4;
    pub const LPF_CUTOFF: usize = 5;
    pub const WIDTH: usize = 6;
    pub const HEIGHT: usize = 7;
    pub const DEPTH: usize = 8;
    pub const WALL_VARY: usize = 9;
    pub const DRY_WET: usize = 10;
    pub const REV_DELAY: usize = 11;
    pub const DENSITY: usize = 12;
    pub const REV_ER_BALANCE: usize = 13;
    pub const HIGH_DAMP: usize = 14;
    pub const FEEDBACK_LEVEL: usize = 15;
}

pub mod tunnel_param {
    pub use super::white_room_param::*;
}

pub mod canyon_param {
    pub use super::white_room_param::*;
}

pub mod basement_param {
    pub use super::white_room_param::*;
}

pub mod karaoke1_param {
    pub const DELAY_TIME: usize = 1;
    pub const FEEDBACK_LEVEL: usize = 2;
    pub const HPF_CUTOFF: usize = 3;
    pub const LPF_CUTOFF: usize = 4;

    pub const DRY_WET: usize = 10;
}

pub mod voice_cancel_param {
    pub const LOW_ADJUST: usize = 11;
    pub const HIGH_ADJUST: usize = 12;
}
