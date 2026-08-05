/// Effect type trait
///
/// Lookup rules (XG Spec + fallback):
/// 1. (msb, lsb) exact match
/// 2. (msb, (lsb/32)×32) — LSB alias (XG Spec: round down to a multiple of 32 on miss)
/// 3. (msb, 0) — fallback
/// 4. `no_effect()` — no rule matched
pub trait EffectType: Sized + Copy + TryFrom<u16> + Into<u16> {
    /// Fallback value used when no rule matches
    fn no_effect() -> Self;

    fn get_type(msb: u8, lsb: u8) -> Self {
        let full = crate::merge_data!(msb as u16, lsb as u16);
        if let Ok(r) = Self::try_from(full) {
            return r;
        }
        // LSB alias: integer part of (lsb/32) × 32
        let alias_lsb = (lsb / 32) * 32;
        if alias_lsb != 0 && alias_lsb != lsb {
            if let Ok(r) = Self::try_from(crate::merge_data!(msb as u16, alias_lsb as u16)) {
                return r;
            }
        }
        // Fallback: (msb, 0)
        if let Ok(r) = Self::try_from(crate::merge_data!(msb as u16)) {
            return r;
        }
        Self::no_effect()
    }

    fn to_tuple(&self) -> (u8, u8) {
        let v: u16 = (*self).into();
        (crate::get_msb_u16_u8!(v), crate::get_lsb_u16_u8!(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::effect_params::{
        chorus_type::XGChorusType, reverb_type::XGReverbType, variation_type::XGVariationType,
    };

    #[test]
    fn reverb_exact_match() {
        assert_eq!(XGReverbType::get_type(0x01, 0x01), XGReverbType::Hall2);
        assert_eq!(XGReverbType::get_type(0x02, 0x02), XGReverbType::Room3);
        assert_eq!(XGReverbType::get_type(0x01, 0x00), XGReverbType::Hall1);
    }

    #[test]
    fn reverb_lsb_alias_32_multiple() {
        // (0x01, 0x20): no exact match → alias (0x01, 0x20)=none → (0x01, 0)=Hall1
        assert_eq!(XGReverbType::get_type(0x01, 0x20), XGReverbType::Hall1);
        // (0x02, 0x21): none → alias (0x02, 0x00)=Room1
        assert_eq!(XGReverbType::get_type(0x02, 0x21), XGReverbType::Room1);
        // (0x09, 0x40): no exact match → alias none → (0x09, 0)=WhiteRoom (XG standard)
        assert_eq!(XGReverbType::get_type(0x09, 0x40), XGReverbType::WhiteRoom);
        // (0x10, 0x40): fallback (0x10, 0)=ReverbExt4 (2006LE extended type)
        assert_eq!(XGReverbType::get_type(0x10, 0x40), XGReverbType::ReverbExt4);
    }

    #[test]
    fn reverb_fallback_to_no_effect() {
        assert_eq!(XGReverbType::get_type(0x7F, 0x7F), XGReverbType::NoEffect);
        assert_eq!(XGReverbType::get_type(0x05, 0x40), XGReverbType::NoEffect);
    }

    #[test]
    fn chorus_alias_and_fallback() {
        // (0x41, 0x00) exact match = Chorus1
        assert_eq!(XGChorusType::get_type(0x41, 0x00), XGChorusType::Chorus1);
        // (0x41, 0x20): none → alias (0x41, 0)=Chorus1
        assert_eq!(XGChorusType::get_type(0x41, 0x20), XGChorusType::Chorus1);
        // (0x41, 0x03): none → alias (0x41, 0)=Chorus1
        assert_eq!(XGChorusType::get_type(0x41, 0x03), XGChorusType::Chorus1);
        // Unknown msb → NoEffect
        assert_eq!(XGChorusType::get_type(0x60, 0x00), XGChorusType::NoEffect);
    }

    #[test]
    fn variation_exact_and_alias() {
        // Exact: DelayLCR = (0x05, 0x00)
        assert_eq!(XGVariationType::get_type(0x05, 0x00), XGVariationType::DelayLCR);
        // Alias: (0x41, 0x03) → (0x41, 0)=Chorus1
        assert_eq!(XGVariationType::get_type(0x41, 0x03), XGVariationType::Chorus1);
        // Alias chain: (0x41, 0x21) → (0x41, 0x20) none → (0x41, 0)=Chorus1
        assert_eq!(XGVariationType::get_type(0x41, 0x21), XGVariationType::Chorus1);
        // Fallback: (0x7F, 0x40) → NoEffect
        assert_eq!(XGVariationType::get_type(0x7F, 0x40), XGVariationType::NoEffect);
        // (0x60, 0x40): fallback (0x60, 0)=VibeVibrate (XG2.0 misc)
        assert_eq!(XGVariationType::get_type(0x60, 0x40), XGVariationType::VibeVibrate);
    }

    #[test]
    fn to_tuple_roundtrip() {
        let (msb, lsb) = XGReverbType::Hall2.to_tuple();
        assert_eq!((msb, lsb), (0x01, 0x01));
    }
}
