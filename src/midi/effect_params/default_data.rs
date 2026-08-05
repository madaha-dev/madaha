const NOEFFECT_THRU: [u16; 16] = [0; 16];
// --- GENERAL ---
pub const XG_RESET_DATA: [u16; 16] = [40, 20, 72, 53, 64, 0, 43, 74, 10, 127, 0, 0, 0, 0, 0, 0];

use super::chorus_type::XGChorusType;
use super::reverb_type::XGReverbType;

/// 2006LE Reverb type → default parameter set (UpCalc reload behavior)
pub fn reverb_default(t: XGReverbType) -> [u16; 16] {
    use xg_reverb_data::*;
    match t {
        XGReverbType::NoEffect => NOEFFECT,
        XGReverbType::Hall1 => HALL1,
        XGReverbType::Hall2 => HALL2,
        XGReverbType::Room1 => ROOM1,
        XGReverbType::Room2 => ROOM2,
        XGReverbType::Room3 => ROOM3,
        XGReverbType::Stage1 => STAGE1,
        XGReverbType::Stage2 => STAGE2,
        XGReverbType::Plate => PLATE,
        XGReverbType::WhiteRoom => WHITEROOM,
        XGReverbType::Tunnel => TUNNEL,
        XGReverbType::Canyon => CANYON,
        XGReverbType::Basement => BASEMENT,
        XGReverbType::ReverbExt1 => EXT1,
        XGReverbType::ReverbExt2 => EXT2,
        XGReverbType::ReverbExt3 => EXT3,
        XGReverbType::ReverbExt4 => EXT4,
        XGReverbType::ReverbExt5 => EXT5,
        XGReverbType::ReverbExt6 => EXT6,
    }
}

/// 2006LE Chorus type → default parameter set (UpCalc reload behavior)
pub fn chorus_default(t: XGChorusType) -> [u16; 16] {
    use xg_chorus_data::*;
    match t {
        XGChorusType::NoEffect => NOEFFECT,
        XGChorusType::Chorus1 => CHORUS1,
        XGChorusType::Chorus2 => CHORUS2,
        XGChorusType::Chorus3 => CHORUS3,
        XGChorusType::Chorus4 => CHORUS4,
        XGChorusType::Celeste1 => CELESTE1,
        XGChorusType::Celeste2 => CELESTE2,
        XGChorusType::Celeste3 => CELESTE3,
        XGChorusType::Celeste4 => CELESTE4,
        XGChorusType::Flanger1 => FLANGER1,
        XGChorusType::Flanger2 => FLANGER2,
        XGChorusType::Flanger3 => FLANGER3,
        XGChorusType::Symphonic => SYMPHONIC,
        XGChorusType::Phaser => PHASER,
        XGChorusType::ChorusExt1 => CHORUS_EXT1,
        XGChorusType::ChorusExt2 => CHORUS_EXT2,
        XGChorusType::ChorusExt3 => CHORUS_EXT3,
        XGChorusType::ChorusExt4 => CHORUS_EXT4,
        XGChorusType::ChorusExt5 => CHORUS_EXT5,
    }
}

// --- REVERB ---
// 2006LE authoritative defaults (extracted from S-YXG2006LE 0x5d220, uTG table)
pub mod xg_reverb_data {
    use super::NOEFFECT_THRU;

    pub const NOEFFECT: [u16; 16] = NOEFFECT_THRU;
    pub const HALL1: [u16; 16] = [18, 10, 8, 13, 49, 0, 0, 0, 0, 40, 0, 3, 50, 8, 64, 0];
    pub const HALL2: [u16; 16] = [25, 10, 28, 6, 46, 0, 0, 0, 0, 40, 13, 3, 74, 7, 64, 0];
    pub const ROOM1: [u16; 16] = [5, 10, 16, 4, 49, 0, 0, 0, 0, 40, 5, 3, 64, 8, 64, 0];
    pub const ROOM2: [u16; 16] = [12, 10, 5, 4, 38, 0, 0, 0, 0, 40, 0, 3, 50, 8, 64, 0];
    pub const ROOM3: [u16; 16] = [9, 10, 47, 5, 36, 0, 0, 0, 0, 40, 0, 3, 60, 8, 64, 0];
    pub const STAGE1: [u16; 16] = [19, 10, 16, 7, 54, 0, 0, 0, 0, 40, 0, 3, 64, 6, 64, 0];
    pub const STAGE2: [u16; 16] = [11, 10, 16, 7, 51, 0, 0, 0, 0, 40, 2, 2, 64, 6, 64, 0];
    pub const PLATE: [u16; 16] = [25, 10, 6, 8, 49, 0, 0, 0, 0, 40, 2, 3, 64, 5, 64, 0];
    pub const WHITEROOM: [u16; 16] = [9, 5, 11, 0, 46, 30, 50, 70, 7, 40, 34, 3, 64, 7, 64, 0];
    pub const TUNNEL: [u16; 16] = [48, 6, 19, 0, 44, 33, 52, 70, 16, 40, 20, 3, 64, 7, 64, 0];
    pub const CANYON: [u16; 16] = [59, 6, 63, 0, 45, 34, 62, 91, 13, 40, 25, 3, 64, 4, 64, 0];
    pub const BASEMENT: [u16; 16] = [3, 6, 3, 0, 34, 26, 29, 59, 15, 40, 32, 3, 64, 8, 64, 0];
    // 2006LE extended types 0x0D-0x12 (alias values)
    pub const EXT1: [u16; 16] = [12, 10, 5, 4, 38, 0, 0, 0, 0, 40, 0, 3, 50, 8, 64, 0];
    pub const EXT2: [u16; 16] = [5, 10, 16, 4, 49, 0, 0, 0, 0, 40, 5, 3, 64, 8, 64, 0];
    pub const EXT3: [u16; 16] = [9, 10, 47, 5, 36, 0, 0, 0, 0, 40, 0, 3, 60, 8, 64, 0];
    pub const EXT4: [u16; 16] = [18, 10, 8, 13, 49, 0, 0, 0, 0, 40, 0, 3, 50, 8, 64, 0];
    pub const EXT5: [u16; 16] = [25, 10, 28, 6, 46, 0, 0, 0, 0, 40, 13, 3, 74, 7, 64, 0];
    pub const EXT6: [u16; 16] = [25, 10, 6, 8, 49, 0, 0, 0, 0, 40, 2, 3, 64, 5, 64, 0];
}
// --- CHORUS ---
// 2006LE authoritative defaults (extracted from S-YXG2006LE 0x5cf80, uTG table)
pub mod xg_chorus_data {
    use super::NOEFFECT_THRU;

    pub const NOEFFECT: [u16; 16] = NOEFFECT_THRU;
    pub const CHORUS1: [u16; 16] = [6, 54, 77, 106, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS2: [u16; 16] = [8, 63, 64, 30, 0, 28, 62, 42, 58, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS3: [u16; 16] = [4, 44, 64, 110, 0, 28, 64, 46, 66, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS4: [u16; 16] = [12, 32, 64, 0, 0, 28, 64, 46, 64, 127, 40, 68, 10, 0, 0, 0];
    pub const CELESTE1: [u16; 16] = [28, 18, 90, 2, 0, 28, 62, 42, 60, 84, 40, 68, 10, 0, 0, 0];
    pub const CELESTE2: [u16; 16] = [4, 63, 44, 2, 0, 28, 64, 46, 68, 127, 40, 68, 10, 0, 0, 0];
    pub const CELESTE3: [u16; 16] = [14, 14, 104, 2, 0, 28, 64, 46, 64, 96, 40, 64, 10, 4, 1, 0];
    pub const CELESTE4: [u16; 16] = [32, 17, 26, 2, 0, 28, 64, 46, 60, 96, 40, 64, 10, 4, 1, 0];
    pub const FLANGER1: [u16; 16] = [44, 66, 88, 88, 84, 95, 10, 0, 0, 24, 0, 60, 28, 64, 46, 64];
    pub const FLANGER2: [u16; 16] = [41, 99, 44, 99, 10, 41, 44, 0, 0, 40, 0, 60, 28, 64, 46, 64];
    pub const FLANGER3: [u16; 16] = [28, 70, 46, 70, 0, 0, 0, 0, 0, 127, 34, 64, 10, 0, 0, 0];
    pub const SYMPHONIC: [u16; 16] = [9, 32, 69, 104, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 1, 0];
    pub const PHASER: [u16; 16] = [8, 29, 64, 0, 0, 28, 64, 51, 66, 127, 40, 68, 10, 0, 1, 0];
    // 2006LE extended Chorus types (t16-20)
    pub const CHORUS_EXT1: [u16; 16] = [26, 34, 67, 105, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS_EXT2: [u16; 16] = [9, 34, 69, 105, 0, 28, 64, 46, 66, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS_EXT3: [u16; 16] = [26, 29, 75, 102, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS_EXT4: [u16; 16] = [6, 43, 107, 111, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS_EXT5: [u16; 16] = [3, 21, 120, 1, 0, 28, 64, 46, 64, 96, 40, 64, 10, 4, 0, 0];
}
// --- VARIATION ---
// family 0 (standard) types: reverb-like (Chorus layout) + delay family (param layout)
// from S-YXG2006LE 0x5c9a0 (uTG table); ER family keeps XG Spec param layout (see below)
pub mod xg_variation_data {
    use super::NOEFFECT_THRU;
    pub const NOEFFCT: [u16; 16] = NOEFFECT_THRU;
    pub const HALL1: [u16; 16] = [6, 54, 77, 106, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 0, 0];
    pub const HALL2: [u16; 16] = [8, 63, 64, 30, 0, 28, 62, 42, 58, 64, 46, 64, 10, 0, 0, 0];
    pub const ROOM1: [u16; 16] = [4, 44, 64, 110, 0, 28, 64, 46, 66, 64, 46, 64, 10, 0, 0, 0];
    pub const ROOM2: [u16; 16] = [12, 32, 64, 0, 0, 28, 64, 46, 64, 127, 40, 68, 10, 0, 0, 0];
    pub const ROOM3: [u16; 16] = [28, 18, 90, 2, 0, 28, 62, 42, 60, 84, 40, 68, 10, 0, 0, 0];
    pub const STAGE1: [u16; 16] = [4, 63, 44, 2, 0, 28, 64, 46, 68, 127, 40, 68, 10, 0, 0, 0];
    pub const STAGE2: [u16; 16] = [14, 14, 104, 2, 0, 28, 64, 46, 64, 96, 40, 64, 10, 4, 1, 0];
    pub const PLATE: [u16; 16] = [32, 17, 26, 2, 0, 28, 64, 46, 60, 96, 40, 64, 10, 4, 1, 0];
    pub const WHITEROOM: [u16; 16] = [12, 25, 16, 0, 0, 28, 64, 46, 64, 127, 46, 64, 10, 0, 0, 0];
    pub const TUNNEL: [u16; 16] = [81, 35, 0, 0, 0, 24, 60, 45, 54, 127, 33, 52, 30, 0, 0, 0];
    pub const CANYON: [u16; 16] = [83, 56, 0, 0, 0, 28, 64, 46, 64, 127, 40, 64, 10, 64, 0, 0];
    pub const BASEMENT: [u16; 16] = [76, 80, 32, 5, 0, 28, 64, 46, 64, 127, 40, 64, 10, 0, 0, 0];
    pub const DELAY_LCR: [u16; 16] = [12, 25, 16, 0, 0, 28, 64, 46, 64, 127, 46, 64, 10, 0, 0, 0];
    pub const DELAY_LR: [u16; 16] = [81, 35, 0, 0, 0, 24, 60, 45, 54, 127, 33, 52, 30, 0, 0, 0];
    pub const ECHO: [u16; 16] = [83, 56, 0, 0, 0, 28, 64, 46, 64, 127, 40, 64, 10, 64, 0, 0];
    pub const CROSSDELAY: [u16; 16] = [76, 80, 32, 5, 0, 28, 64, 46, 64, 127, 40, 64, 10, 0, 0, 0];
    pub const ER1: [u16; 16] = [3333, 1667, 5000, 5000, 74, 100, 10, 0, 0, 32, 0, 60, 28, 64, 46, 64];
    pub const ER2: [u16; 16] = [2500, 3750, 3752, 3750, 87, 10, 0, 0, 0, 32, 0, 60, 28, 64, 46, 64];
    pub const GATE_REVERB: [u16; 16] = [1700, 80, 1780, 80, 10, 1700, 1780, 0, 0, 40, 0, 60, 28, 64, 46, 64];
    pub const REVERSE_GATE: [u16; 16] = [1700, 1750, 111, 1, 10, 0, 0, 0, 0, 32, 0, 60, 28, 64, 46, 64];
    pub const KARAOKE1: [u16; 16] = [63, 97, 0, 48, 0, 0, 0, 0, 0, 64, 2, 0, 0, 0, 0, 0];
    pub const KARAOKE2: [u16; 16] = [55, 105, 0, 50, 0, 0, 0, 0, 0, 64, 1, 0, 0, 0, 0, 0];
    pub const KARAOKE3: [u16; 16] = [43, 110, 14, 53, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0];
    pub const CHORUS1: [u16; 16] = [6, 54, 77, 106, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS2: [u16; 16] = [8, 63, 64, 30, 0, 28, 62, 42, 58, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS3: [u16; 16] = [4, 44, 64, 110, 0, 28, 64, 46, 66, 64, 46, 64, 10, 0, 0, 0];
    pub const CHORUS4: [u16; 16] = [9, 32, 69, 104, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 1, 0];
    pub const CELESTE1: [u16; 16] = [12, 32, 64, 0, 0, 28, 64, 46, 64, 127, 40, 68, 10, 0, 0, 0];
    pub const CELESTE2: [u16; 16] = [28, 18, 90, 2, 0, 28, 62, 42, 60, 84, 40, 68, 10, 0, 0, 0];
    pub const CELESTE3: [u16; 16] = [4, 63, 44, 2, 0, 28, 64, 46, 68, 127, 40, 68, 10, 0, 0, 0];
    pub const CELESTE4: [u16; 16] = [8, 29, 64, 0, 0, 28, 64, 51, 66, 127, 40, 68, 10, 0, 1, 0];
    pub const FLANGER1: [u16; 16] = [14, 14, 104, 2, 0, 28, 64, 46, 64, 96, 40, 64, 10, 4, 0, 0];
    pub const FLANGER2: [u16; 16] = [32, 17, 26, 2, 0, 28, 64, 46, 60, 96, 40, 64, 10, 4, 0, 0];
    pub const FLANGER3: [u16; 16] = [4, 109, 109, 2, 0, 28, 64, 46, 64, 127, 40, 64, 10, 4, 0, 0];
    pub const SYMPHONIC: [u16; 16] = [12, 25, 16, 0, 0, 28, 64, 46, 64, 127, 46, 64, 10, 0, 0, 0];
    pub const ROTARY_SPEAKER: [u16; 16] =
        [81, 35, 0, 0, 0, 24, 60, 45, 54, 127, 33, 52, 30, 0, 0, 0];
    pub const TREMOLO: [u16; 16] = [83, 56, 0, 0, 0, 28, 64, 46, 64, 127, 40, 64, 10, 64, 0, 0];
    pub const AUTOPAN: [u16; 16] = [76, 80, 32, 5, 0, 28, 64, 46, 64, 127, 40, 64, 10, 0, 0, 0];
    pub const PHASER1: [u16; 16] = [8, 111, 74, 104, 0, 28, 64, 46, 64, 64, 6, 1, 64, 0, 0, 0];
    pub const PHASER2: [u16; 16] = [8, 111, 74, 108, 0, 28, 64, 46, 64, 64, 5, 1, 4, 0, 0, 0];
    pub const DISTORTION: [u16; 16] = [40, 20, 72, 53, 48, 0, 43, 74, 10, 127, 120, 0, 0, 0, 0, 0];
    pub const OVERDRIVE: [u16; 16] = [29, 24, 68, 45, 55, 0, 41, 72, 10, 127, 104, 0, 0, 0, 0, 0];
    pub const AMP_SIM: [u16; 16] = [39, 1, 48, 55, 0, 0, 0, 0, 0, 127, 112, 0, 0, 0, 0, 0];
    pub const THREE_BAND_EQ: [u16; 16] = [70, 34, 60, 10, 70, 28, 46, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const TWO_BAND_EQ: [u16; 16] = [28, 70, 46, 70, 0, 0, 0, 0, 0, 127, 34, 64, 10, 0, 0, 0];
    pub const AUTOWAH_LFO: [u16; 16] = [70, 56, 39, 25, 0, 28, 66, 46, 64, 127, 0, 0, 0, 0, 0, 0];
    pub const PITCH_CHANGE: [u16; 16] = [64, 0, 74, 54, 64, 0, 0, 0, 0, 64, 1, 127, 127, 127, 0, 0];
    pub const AURAL_EXCITER: [u16; 16] = [44, 30, 48, 0, 0, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const TOUCH_WAH: [u16; 16] = [36, 0, 30, 0, 0, 28, 66, 46, 64, 127, 0, 0, 0, 0, 0, 0];
    pub const TOUCH_WAH_DIST: [u16; 16] = [36, 0, 30, 0, 0, 28, 66, 46, 64, 127, 30, 0, 0, 0, 0, 0];
    pub const COMPRESSOR: [u16; 16] = [6, 2, 100, 4, 96, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const NOISE_GATE: [u16; 16] = [0, 11, 82, 50, 0, 0, 0, 0, 0, 127, 3, 0, 0, 0, 0, 0];
    pub const VOICE_CANCEL: [u16; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 8, 25, 0, 0, 0, 0];
    pub const THRU: [u16; 16] = NOEFFECT_THRU;
}

pub mod xg_distortion_data {
    pub const DISTORTION: [u16; 16] = [40, 20, 72, 53, 48, 0, 43, 74, 10, 127, 120, 0, 0, 0, 0, 0];
    pub const OVERDRIVE: [u16; 16] = [29, 24, 68, 45, 55, 0, 41, 72, 10, 127, 104, 0, 0, 0, 0, 0];
    pub const THREE_BAND_EQ: [u16; 16] = [70, 34, 60, 10, 70, 28, 46, 0, 0, 127, 0, 0, 0, 0, 0, 0];
}
// From XG Spec 2.0

    // From XG Spec 2.0
pub mod xg_20_data {
pub const AUTO_WAH: [u16; 16] = [70, 56, 39, 25, 0, 28, 66, 46, 64, 127, 0, 0, 0, 0, 0, 0];
    pub const AUTO_WAH_DISTORTION: [u16; 16] = [40, 73, 26, 29, 0, 28, 66, 46, 64, 127, 30, 72, 74, 53, 48, 0];
    pub const AUTO_WAH_OVER_DRIVE: [u16; 16] = [48, 64, 32, 23, 0, 28, 66, 46, 64, 127, 29, 68, 72, 45, 55, 0];
    pub const PITCH_CHANGE: [u16; 16] = [64, 0, 74, 54, 64, 0, 0, 0, 0, 64, 1, 127, 127, 127, 0, 0];
    pub const PITCH_CHANGE_2: [u16; 16] =
        [65, 50, 67, 61, 87, 0, 0, 0, 0, 32, 1, 127, 127, 127, 0, 0];
    pub const HARMONIC_ENHANCER: [u16; 16] = [44, 30, 48, 0, 0, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const TOUCH_WAH_1: [u16; 16] = [36, 0, 30, 0, 0, 28, 66, 46, 64, 127, 0, 0, 0, 0, 0, 0];
    pub const TOUCH_WAH_DISTORTION: [u16; 16] = [36, 0, 30, 0, 0, 28, 66, 46, 64, 127, 30, 0, 0, 0, 0, 0];
    pub const TOUCH_WAH_2: [u16; 16] = [68, 18, 60, 0, 0, 28, 66, 46, 64, 127, 0, 72, 74, 53, 57, 64];
    pub const COMPRESSOR: [u16; 16] = [6, 2, 100, 4, 96, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const NOISE_GATE: [u16; 16] = [0, 11, 82, 50, 0, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const VOICE_CANCEL: [u16; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 8, 25, 0, 0, 0, 0];
    pub const TWO_WAY_ROTARY_SPEAKER: [u16; 16] =
        [16, 6, 35, 70, 0, 24, 66, 46, 54, 127, 31, 36, 0, 0, 0, 0];
    pub const DISTORTION_TWO_WAY_ROTARY_SP: [u16; 16] = [
        26, 28, 30, 64, 0, 24, 66, 59, 127, 127, 45, 60, 3, 48, 60, 0,
    ];
    pub const OVER_DRIVE_TWO_WAY_ROTARY_SPEAKER: [u16; 16] =
        [5, 28, 30, 62, 0, 20, 67, 56, 52, 127, 33, 60, 0, 4, 46, 50];
    pub const AMP_SIMULATOR_TWO_WAY_ROTARY_SPEAKER: [u16; 16] =
        [8, 27, 29, 64, 0, 17, 66, 58, 60, 127, 28, 64, 3, 3, 48, 52];
    pub const ENSEMBLE_DETUNE: [u16; 16] = [54, 0, 0, 0, 0, 0, 0, 0, 0, 64, 28, 64, 46, 64, 0, 0];
    pub const AMBIENCE: [u16; 16] = [114, 0, 0, 0, 0, 28, 64, 46, 64, 64, 0, 0, 0, 0, 0, 0];
    pub const VOCODER_HARMONY: [u16; 16] = [0, 0, 0, 64, 0, 1, 1, 84, 44, 64, 39, 47, 0, 0, 0, 0];
    pub const CHORDAL_HARMONY: [u16; 16] = [0, 0, 0, 64, 0, 1, 1, 84, 44, 64, 39, 47, 0, 0, 0, 0];
    pub const DETUNE_HARMONY: [u16; 16] = [0, 0, 0, 64, 0, 0, 0, 0, 0, 64, 39, 47, 0, 0, 0, 0];
    pub const CHROMATIC_HARMONY: [u16; 16] = [0, 0, 0, 64, 0, 1, 1, 84, 44, 64, 39, 47, 0, 0, 0, 0];
    pub const TALKING_MODULATOR: [u16; 16] = [0, 15, 31, 63, 0, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const LO_FI: [u16; 16] = [2, 60, 6, 54, 5, 10, 1, 1, 0, 127, 0, 0, 0, 0, 1, 0];
    pub const DISTORTION_DELAY: [u16; 16] = [2500, 3000, 3750, 74, 70, 40, 48, 72, 74, 127, 0, 0, 0, 0, 0, 0];
    pub const OVER_DRIVE_DELAY: [u16; 16] = [1900, 1400, 2500, 78, 60, 29, 55, 68, 72, 127, 0, 0, 0, 0, 0, 0];
    pub const COMPRESSOR_DISTORTION_DELAY: [u16; 16] = [3000, 72, 66, 40, 48, 72, 74, 0, 0, 127, 6, 2, 100, 4, 0, 0];
    pub const COMPRESSOR_OVER_DRIVE_DELAY: [u16; 16] = [3000, 72, 66, 29, 55, 68, 72, 0, 0, 127, 6, 2, 100, 4, 0, 0];
    pub const WAH_DISTORTION_DELAY: [u16; 16] = [1600, 84, 64, 30, 48, 69, 72, 0, 0, 127, 40, 0, 30, 64, 0, 0];
    pub const WAH_OVER_DRIVE_DELAY: [u16; 16] = [1600, 84, 64, 24, 55, 65, 70, 0, 0, 127, 40, 0, 30, 64, 0, 0];
    pub const V_DISTORTION_HARD: [u16; 16] = [22, 3, 2, 6, 88, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const V_DISTORTION_HARD_DELAY: [u16; 16] = [22, 3, 2, 5, 82, 2500, 5000, 5000, 85, 127, 46, 0, 0, 0, 0, 0];
    pub const V_DISTORTION_SOFT: [u16; 16] = [13, 3, 2, 6, 98, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const V_DISTORTION_SOFT_DELAY: [u16; 16] = [14, 3, 2, 6, 92, 2500, 5000, 5000, 76, 127, 44, 0, 0, 0, 0, 0];
    pub const DUAL_ROTOR_SPEAKER_1: [u16; 16] = [15, 18, 89, 91, 54, 22, 20, 22, 52, 127, 14, 72, 34, 61, 60, 0];
    pub const DUAL_ROTOR_SPEAKER_2: [u16; 16] = [14, 18, 91, 95, 54, 22, 22, 29, 64, 127, 34, 64, 34, 64, 60, 0];
    pub const DISTORTION_TEMPO_DELAY: [u16; 16] = [11, 86, 62, 8, 88, 70, 68, 64, 64, 127, 0, 0, 0, 0, 0, 0];
    pub const OVER_DRIVE_TEMPO_DELAY: [u16; 16] = [11, 78, 64, 10, 110, 68, 70, 70, 64, 127, 0, 0, 0, 0, 0, 0];
    pub const COMPRESSOR_DISTORTION_TEMPO_DELAY: [u16; 16] = [11, 76, 62, 10, 90, 72, 74, 70, 64, 127, 4, 4, 101, 3, 0, 0];
    pub const COMPRESSOR_OVER_DRIVE_TEMPO_DELAY: [u16; 16] = [11, 74, 62, 9, 94, 74, 74, 70, 64, 127, 8, 9, 103, 3, 0, 0];
    pub const WAH_OVER_DRIVE_TEMPO_DELAY: [u16; 16] = [11, 84, 58, 16, 127, 68, 65, 70, 64, 127, 64, 0, 30, 64, 0, 0];
    pub const WAH_DISTORTION_TEMPO_DELAY: [u16; 16] = [11, 86, 52, 10, 120, 68, 64, 70, 64, 127, 50, 0, 30, 64, 0, 0];
    pub const V_DISTORTION_HARD_TEMPO_DELAY: [u16; 16] = [22, 3, 2, 5, 82, 11, 90, 72, 64, 127, 64, 0, 0, 0, 0, 0];
    pub const V_DISTORTION_SOFT_TEMPO_DELAY: [u16; 16] = [14, 3, 2, 6, 90, 11, 92, 77, 64, 127, 66, 0, 0, 0, 0, 0];
    pub const V_FLANGER: [u16; 16] = [5, 45, 0, 17, 184, 28, 64, 46, 64, 127, 46, 64, 10, 16, 9, 5];
    pub const MULTI_BAND_COMP_BASIC: [u16; 16] = [9, 64, 64, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    pub const TEMPO_FLANGER: [u16; 16] = [17, 10, 12, 2, 0, 28, 64, 46, 64, 96, 40, 64, 10, 64, 0, 0];
    pub const TEMPO_PHASER: [u16; 16] = [17, 48, 67, 108, 0, 28, 64, 46, 64, 64, 6, 0, 64, 0, 0, 0];
    pub const DYNA_FILTER: [u16; 16] = [1, 110, 0, 66, 19, 40, 110, 0, 0, 96, 0, 0, 28, 64, 46, 64];
    pub const DYNA_FLANGER: [u16; 16] = [122, 1, 6, 63, 65, 100, 0, 0, 0, 96, 0, 0, 28, 64, 46, 64];
    pub const DYNA_PHASER: [u16; 16] = [98, 0, 113, 30, 52, 25, 0, 0, 0, 32, 6, 0, 28, 64, 46, 64];
    pub const DYNA_RING_MODULATOR: [u16; 16] = [70, 0, 60, 12, 58, 70, 0, 0, 10, 64, 0, 0, 28, 64, 46, 64];
    pub const RING_MODULATOR: [u16; 16] = [98, 0, 0, 0, 64, 0, 60, 0, 0, 127, 0, 0, 28, 64, 46, 64];
    pub const SLICE: [u16; 16] = [5, 30, 2, 127, 64, 1, 0, 0, 0, 127, 0, 0, 28, 64, 46, 64];
    pub const ISOLATOR: [u16; 16] = [1, 64, 64, 64, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    pub const LOW_RESOLUTION: [u16; 16] = [3, 1, 66, 0, 64, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const DIGITAL_TURNTABLE: [u16; 16] = [1, 20, 2, 15, 72, 4, 52, 15, 20, 127, 127, 49, 0, 0, 0, 0];
    pub const DIGITAL_SCRATCH: [u16; 16] = [80, 1800, 9, 90, 16, 127, 46, 64, 20, 64, 12, 0, 0, 0, 0, 0];
    pub const VIBE_VIBRATE: [u16; 16] = [69, 56, 12, 0, 0, 28, 64, 46, 64, 38, 40, 64, 10, 64, 1, 1];
    pub const THREE_D_MANUAL: [u16; 16] = [64, 64, 40, 10, 0, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const THREE_D_AUTO: [u16; 16] = [64, 64, 40, 2, 10, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
    pub const WIDE_STEREO: [u16; 16] = [89, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 0, 0, 0, 0];
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverb_hall1_matches_2006le() {
        // 2006LE table @0x5d220 (uTG): [18,10,8,13,49,0,0,0,0,40,0,3,50,8,64,0]
        assert_eq!(xg_reverb_data::HALL1, [18, 10, 8, 13, 49, 0, 0, 0, 0, 40, 0, 3, 50, 8, 64, 0]);
        // WhiteRoom matches the full 2006LE row (DENCITY=3, not the old XG Spec 4)
        assert_eq!(xg_reverb_data::WHITEROOM, [9, 5, 11, 0, 46, 30, 50, 70, 7, 40, 34, 3, 64, 7, 64, 0]);
    }

    #[test]
    fn chorus4_uses_2006le_value() {
        // 2006LE table @0x5cfe0: Chorus4 is the 4th row, distinct from Symphonic
        assert_eq!(xg_chorus_data::CHORUS4, [12, 32, 64, 0, 0, 28, 64, 46, 64, 127, 40, 68, 10, 0, 0, 0]);
        assert_eq!(xg_chorus_data::SYMPHONIC, [9, 32, 69, 104, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 1, 0]);
    }

    #[test]
    fn variation_hall_is_chorus_layout() {
        // 2006LE: Variation reverb-like types use the Chorus parameter layout (CalcChorusVar)
        assert_eq!(xg_variation_data::HALL1, xg_chorus_data::CHORUS1);
        assert_eq!(xg_variation_data::DELAY_LCR, [12, 25, 16, 0, 0, 28, 64, 46, 64, 127, 46, 64, 10, 0, 0, 0]);
    }

    #[test]
    fn type_defaults_match_table() {
        assert_eq!(reverb_default(XGReverbType::Hall2), xg_reverb_data::HALL2);
        assert_eq!(chorus_default(XGChorusType::Celeste1), xg_chorus_data::CELESTE1);
        assert_eq!(reverb_default(XGReverbType::Basement), xg_reverb_data::BASEMENT);
    }

    #[test]
    fn xg20_serial_layouts_match_doc() {
        use super::xg_20_data::*;
        // XG Spec: DISTORTION+DELAY = P1-3 delay(14-bit), P6-10 distortion
        assert_eq!(DISTORTION_DELAY[0], 2500); // Lch Delay
        assert_eq!(DISTORTION_DELAY[1], 3000); // Rch Delay
        assert_eq!(DISTORTION_DELAY[2], 3750); // Feedback Time
        assert_eq!(DISTORTION_DELAY[5], 40);   // P6 Dist Drive
        assert_eq!(DISTORTION_DELAY[9], 127);  // P10 Dry/Wet
        // AUTO WAH+DISTORTION: P5 unused(0), P11 Drive, P15 Output
        assert_eq!(AUTO_WAH_DISTORTION[4], 0);
        assert_eq!(AUTO_WAH_DISTORTION[10], 30); // P11 Drive
        // V-DISTORTION HARD+DELAY: P5 output, P6-8 delay
        assert_eq!(V_DISTORTION_HARD_DELAY[4], 82); // P5 Output
        assert_eq!(V_DISTORTION_HARD_DELAY[5], 2500); // P6 Delay L
        assert_eq!(V_DISTORTION_HARD_DELAY[8], 85); // P9 Feedback
    }

    #[test]
    fn extended_types_match_2006le() {
        // 2006LE Reverb t13-18 (0x0D-0x12): alias values
        assert_eq!(reverb_default(XGReverbType::ReverbExt1), xg_reverb_data::EXT1);
        assert_eq!(xg_reverb_data::EXT1, xg_reverb_data::ROOM2); // = Room2 params
        assert_eq!(xg_reverb_data::EXT4, xg_reverb_data::HALL1); // = Hall1 params
        // 2006LE Chorus t16-20 (0x10-0x14): independent values
        assert_eq!(chorus_default(XGChorusType::ChorusExt1), xg_chorus_data::CHORUS_EXT1);
        assert_eq!(xg_chorus_data::CHORUS_EXT1, [26, 34, 67, 105, 0, 28, 64, 46, 64, 64, 46, 64, 10, 0, 0, 0]);
    }
}
