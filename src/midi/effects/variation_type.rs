use super::super::ram::EffectData;
use super::interface::Effect;
use crate::{get_lsb_u16_u8, get_msb_u16_u8, merge_data};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
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

//impl_xg_effect_type!(XGVariationType, NoEffect);

impl Effect for XGVariationType {
    fn get_type(msb: u8, lsb: u8) -> Self {
        let full = merge_data!(msb as u16, lsb as u16);
        match Self::try_from(full) {
            Ok(r) => r,
            Err(_) => {
                let msb_only = merge_data!(msb as u16);
                Self::try_from(msb_only).unwrap_or(Self::NoEffect)
            }
        }
    }

    fn load_parameter(
        data: &mut EffectData,
        effect_group: usize,
        effect_type: Self,
        default_data: [u16; 16],
    ) {
        data[effect_group][0x40] = get_msb_u16_u8!(effect_type);
        for i in 0..10 {
            let addr = 0x42 + 2 * i;
            data[effect_group][addr] = get_msb_u16_u8!(default_data[i]);
            data[effect_group][addr + 1] = get_lsb_u16_u8!(default_data[i]);
        }
        for i in 0..6 {
            let addr = 0x70 + i;
            data[effect_group][addr] = default_data[0x0A + i] as u8;
        }
    }
}
