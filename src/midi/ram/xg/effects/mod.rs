pub mod chorus;
pub mod interface;
pub mod reverb;
pub mod variation;

use std::any::TypeId;
use std::fmt::Debug;

use chorus::Chorus;
use interface::EffectRAM;
use reverb::Reverb;
use variation::Variation;

use crate::midi::{
    effect_params::{
        chorus_type::XGChorusType, interface::EffectType, reverb_type::XGReverbType,
        variation_type::XGVariationType,
    },
    errors::MidiError,
    ram::{MIDICallbackEffects, interface::Memory},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectData {
    // offset = 0x00
    pub reverb: Reverb,
    // offset = 0x20
    pub chorus: Chorus,
    // offset = 0x40,
    pub variation: Variation,
}

impl EffectRAM for EffectData {
    fn new() -> Self {
        Self {
            reverb: Reverb::new(),
            chorus: Chorus::new(),
            variation: Variation::new(),
        }
    }
    fn load_parameter<T>(&mut self, effect_type: T, default_data: [u16; 16])
    where
        T: EffectType + 'static,
    {
        let type_id = TypeId::of::<T>();

        if type_id == TypeId::of::<XGReverbType>() {
            self.reverb.load_parameter(effect_type, default_data);
        } else if type_id == TypeId::of::<XGChorusType>() {
            self.chorus.load_parameter(effect_type, default_data);
        } else if type_id == TypeId::of::<XGVariationType>() {
            self.variation.load_parameter(effect_type, default_data);
        }
    }
    
    fn get_parameter<T>(&mut self, effect_type: T, param_index: u8) -> Option<u16>
    where
        T: EffectType + Debug + 'static,
    {
        let type_id = TypeId::of::<T>();
        if type_id == TypeId::of::<XGReverbType>() {
            self.reverb.get_parameter(effect_type, param_index)
        } else if type_id == TypeId::of::<XGChorusType>() {
            self.chorus.get_parameter(effect_type, param_index)
        } else if type_id == TypeId::of::<XGVariationType>() {
            self.variation.get_parameter(effect_type, param_index)
        } else {
            None
        }
    }
}

impl std::ops::Index<usize> for EffectData {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0x00..=0x15 => &self.reverb[index],
            0x20..=0x35 => &self.chorus[index - 0x20],
            0x40..=0x60 | 0x70..=0x75 => &self.variation[index - 0x40],
            _ => &0xFF,
        }
    }
}

impl std::ops::IndexMut<usize> for EffectData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0x00..=0x15 => &mut self.reverb[index],
            0x20..=0x35 => &mut self.chorus[index - 0x20],
            0x40..=0x60 | 0x70..=0x75 => &mut self.variation[index - 0x40],
            _ => panic!("EffectData: not writable memory at {:?}", index),
        }
    }
}

impl Memory for EffectData {
    fn reset(&mut self) {
        self.reverb.reset();
        self.chorus.reset();
        self.variation.reset();
    }

    fn get(&self, addr: crate::midi::ram::MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x15|0x20..=0x35|0x40..=0x60 | 0x70..=0x75) {
            return Err(err);
        }

        Ok(self[addr as usize])
    }

    fn set(&mut self, addr: crate::midi::ram::MemoryAddr, value: u8) -> Result<Vec<MIDICallbackEffects>, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let addr = addr[2];
        if !matches!(addr, 0x00..=0x15|0x20..=0x35|0x40..=0x60 | 0x70..=0x75) {
            return Err(err);
        }
        self[addr as usize] = value;
        Ok(vec![])
    }
}
