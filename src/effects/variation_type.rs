use crate::{impl_xg_effect_type, merge_data};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum XGVariationType {
    NoEffect = merge_data!(0x0),

    Hall1 = merge_data!(0x1, 0x1),
    Hall2 = merge_data!(0x1, 0x2),

    Room1 = merge_data!(0x2),
    Room2 = merge_data!(0x2, 0x1),
    Room3 = merge_data!(0x2, 0x2),

    Stage1 = merge_data!(0x3),
    Stage2 = merge_data!(0x3, 0x1),

    Plate = merge_data!(0x4),

    DelayLCR = merge_data!(0x5),
    DelayLR = merge_data!(0x6),

    Echo = merge_data!(0x7),

    CrossDelay = merge_data!(0x8),

    ER1 = merge_data!(0x9),
    ER2 = merge_data!(0x9, 0x1),

    GateReverb = merge_data!(0xA),
    ReverseGate = merge_data!(0xB),

    Karaoke1 = merge_data!(0x14),
    Karaoke2 = merge_data!(0x14, 0x1),
    Karaoke3 = merge_data!(0x14, 0x2),

    Thru = merge_data!(0x40),

    Chorus1 = merge_data!(0x41),
    Chorus2 = merge_data!(0x41, 0x1),
    Chorus3 = merge_data!(0x41, 0x2),
    Chorus4 = merge_data!(0x41, 0x8),

    Celeste1 = merge_data!(0x42),
    Celeste2 = merge_data!(0x42, 0x1),
    Celeste3 = merge_data!(0x42, 0x2),
    Celeste4 = merge_data!(0x42, 0x8),

    Flanger1 = merge_data!(0x43),
    Flanger2 = merge_data!(0x43, 0x1),
    Flanger3 = merge_data!(0x43, 0x8),

    Symphonic = merge_data!(0x44),

    RotarySpeaker = merge_data!(0x45),
    Tremolo = merge_data!(0x46),
    AutoPan = merge_data!(0x47),

    Phaser1 = merge_data!(0x48),
    Phaser2 = merge_data!(0x48, 0x8),

    Distortion = merge_data!(0x49),
    Overdrive = merge_data!(0x4A),

    AmpSimulator = merge_data!(0x4B),

    ThreeBandEQ = merge_data!(0x4C),
    TwoBandEQ = merge_data!(0x4D),

    AutoWah = merge_data!(0x4E),

    PitchChange = merge_data!(0x50),
    AuralExciter = merge_data!(0x51),

    TouchWah = merge_data!(0x52),
    TouchWahDist = merge_data!(0x52, 0x1),

    Compressor = merge_data!(0x53),
    NoiseGate = merge_data!(0x54),
    VoiceCancel = merge_data!(0x55),
}

impl_xg_effect_type!(XGVariationType, NoEffect);
