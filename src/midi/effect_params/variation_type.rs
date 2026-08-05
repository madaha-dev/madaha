use super::interface::EffectType;
use crate::merge_data;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum XGVariationType {
    NoEffect = merge_data!(0x0),

    Hall1 = merge_data!(0x1),
    Hall2 = merge_data!(0x1, 0x1),

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
    DistortionTwoWayRotarySP = merge_data!(0x45, 0x1),
    OverdriveTwoWayRotarySP = merge_data!(0x45, 0x2),
    Tremolo = merge_data!(0x46),
    AutoPan = merge_data!(0x47),

    Phaser1 = merge_data!(0x48),
    Phaser2 = merge_data!(0x48, 0x8),

    Distortion = merge_data!(0x49),
    DistortionDelay = merge_data!(0x49, 0x1),
    DistortionTempoDelay = merge_data!(0x49, 0x2),
    VDistortionHard = merge_data!(0x49, 0x3),
    VDistortionHardDelay = merge_data!(0x49, 0x4),
    VDistortionHardTempoDelay = merge_data!(0x49, 0x5),
    VDistortionSoft = merge_data!(0x49, 0x6),
    VDistortionSoftDelay = merge_data!(0x49, 0x7),
    VDistortionSoftTempoDelay = merge_data!(0x49, 0x8),
    Overdrive = merge_data!(0x4A),
    OverdriveDelay = merge_data!(0x4A, 0x1),
    OverdriveTempoDelay = merge_data!(0x4A, 0x2),

    AmpSimulator = merge_data!(0x4B),
    AmpSimTwoWayRotarySP = merge_data!(0x4B, 0x1),

    ThreeBandEQ = merge_data!(0x4C),
    TwoBandEQ = merge_data!(0x4D),

    AutoWah = merge_data!(0x4E),
    AutoWahDistortion = merge_data!(0x4E, 0x1),
    AutoWahOverdrive = merge_data!(0x4E, 0x2),
    WahDistortionDelay = merge_data!(0x4E, 0x3),
    WahOverdriveDelay = merge_data!(0x4E, 0x4),
    WahDistortionTempoDelay = merge_data!(0x4E, 0x5),
    WahOverdriveTempoDelay = merge_data!(0x4E, 0x6),

    PitchChange = merge_data!(0x50),
    AuralExciter = merge_data!(0x51),

    TouchWah = merge_data!(0x52),
    TouchWahDist = merge_data!(0x52, 0x1),

    Compressor = merge_data!(0x53),
    CompressorDistortionDelay = merge_data!(0x53, 0x1),
    CompressorOverdriveDelay = merge_data!(0x53, 0x2),
    CompressorDistortionTempoDelay = merge_data!(0x53, 0x3),
    CompressorOverdriveTempoDelay = merge_data!(0x53, 0x4),
    NoiseGate = merge_data!(0x54),
    VoiceCancel = merge_data!(0x55),
    DynaFilter = merge_data!(0x56),
    DynaFlanger = merge_data!(0x57),
    DynaPhaser = merge_data!(0x58),
    DynaRingModulator = merge_data!(0x59),
    RingModulator = merge_data!(0x5A),
    EnsembleDetune = merge_data!(0x5B),
    Ambience = merge_data!(0x5C),
    WideStereo = merge_data!(0x5D),
    ThreeDManual = merge_data!(0x5E),
    ThreeDAuto = merge_data!(0x5F),
    VibeVibrate = merge_data!(0x60),
    LoFi = merge_data!(0x61),
    Slice = merge_data!(0x62),
    Isolator = merge_data!(0x63),
    LowResolution = merge_data!(0x64),
    DigitalTurntable = merge_data!(0x65),
    DigitalScratch = merge_data!(0x66),
    MultiBandComp = merge_data!(0x67),
    TempoFlanger = merge_data!(0x68),
    TempoPhaser = merge_data!(0x69),
    VFlanger = merge_data!(0x6A),
    DualRotorSpeaker1 = merge_data!(0x6B),
    DualRotorSpeaker2 = merge_data!(0x6C),
    VocoderHarmony = merge_data!(0x6D),
    ChordalHarmony = merge_data!(0x6E),
    DetuneHarmony = merge_data!(0x6F),
    ChromaticHarmony = merge_data!(0x70),
    TalkingModulator = merge_data!(0x71),
}

//impl_xg_effect_type!(XGVariationType, NoEffect);
impl EffectType for XGVariationType {
    fn no_effect() -> Self {
        Self::NoEffect
    }
}
