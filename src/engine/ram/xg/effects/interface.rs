use crate::engine::effect_params::interface::EffectType;
use std::ops::{Index, IndexMut};
use std::fmt::Debug;

pub trait EffectRAM: Index<usize> + IndexMut<usize>{
    fn new() -> Self;
    fn load_parameter<T>(&mut self, effect_type: T, default_data: [u16; 16])
    where
        T: EffectType + 'static;
    fn get_parameter<T>(&mut self, effect_type: T, param_index: u8) -> Option<u16>
    where
        T: EffectType + Debug + 'static;
}
