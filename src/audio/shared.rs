use std::sync::Arc;

use crate::double_buffer::DoubleBuffered;
use crate::midi::ram::xg::{
    drum_setup_wrapper::DrumSetupWrapper, effect_insertion::EffectInsertion,
    effects::EffectData, multi_eq::MultiEQ, system::System,
};

/// Effect/system parameters shared between the Audio thread and the MIDI thread (all double-buffered)
#[derive(Debug, Clone)]
pub struct AudioShared {
    pub system: Arc<DoubleBuffered<System>>,
    pub effect1: Arc<DoubleBuffered<EffectData>>,
    pub multi_eq: Arc<DoubleBuffered<MultiEQ>>,
    pub effect_instertion: Arc<DoubleBuffered<[EffectInsertion; 0x80]>>,
    /// Drum note parameters (SysEx 3n, 16 setup banks × 79 notes)
    pub drum_setup: Arc<DoubleBuffered<[DrumSetupWrapper; 16]>>,
    /// GM2/GM1 master volume (14-bit, 0-0x4000), applied on top of System.master_volume
    pub master_volume: Arc<DoubleBuffered<u16>>,
}
