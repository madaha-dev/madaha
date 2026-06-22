use crate::midi::effects::interface::EffectType;
use std::ops::{Index, IndexMut};

pub trait EffectRAM: Index<usize> + IndexMut<usize> {
    fn new() -> Self;
    fn load_parameter<T>(&mut self, effect_type: T, default_data: [u16; 16])
    where
        T: EffectType + 'static;
}
