use std::sync::mpsc::Receiver;

use crate::config::ScoringConfig;

use crate::audio::dsp::{
    EffectProcessor, MultiEqDsp, build_chorus, build_reverb, build_variation,
};
use std::collections::HashMap;

use crate::midi::effect_params::variation_type::XGVariationType;
use crate::midi::ram::xg::multi_eq::EQBand;

use super::AudioShared;
use super::sink::{AudioSink, VecBufferSink};
use super::tone_generator::ToneGenerator;
use super::tone_generator::ToneGeneratorStatus::Running;
use super::AudioRenderActions;

pub struct AudioRender {
    pub tone_generators: Box<[ToneGenerator]>,
    pub rx: Receiver<AudioRenderActions>,
    pub max_polyphony: u16,
    /// Shared effect/system parameters (None until the Init event arrives)
    pub shared: Option<AudioShared>,
    /// Audio output target
    pub sink: Box<dyn AudioSink>,
    /// Target sample rate (the tone_generator chain already operates at this rate)
    pub sample_rate: f32,
    /// DC offset blockers on the master bus (XG Spec: serial chains introduce DC)
    pub dc_l: super::dsp::core::dc_blocker::DcBlocker,
    pub dc_r: super::dsp::core::dc_blocker::DcBlocker,
    /// Master-bus DC blocking enabled (config: audio.dc_blocker, default true)
    pub dc_enabled: bool,

    // ── System effect instances (stage 3) ──
    pub reverb: Box<dyn EffectProcessor>,
    pub chorus: Box<dyn EffectProcessor>,
    pub variation: Box<dyn EffectProcessor>,
    pub multi_eq: MultiEqDsp,
    /// Parameter cache (type + parameters, rebuilt only on change)
    pub reverb_key: (u8, u8, [u16; 16]),
    pub chorus_key: (u8, u8, [u16; 16]),
    pub variation_key: (u8, u8, [u16; 16]),
    pub multi_eq_key: (u8, EQBand, EQBand, EQBand, EQBand, EQBand),

    // ── Insertion effect instance cache (03 nn → processor) ──
    pub insertion_instances: HashMap<u8, Box<dyn EffectProcessor>>,
    pub insertion_key: HashMap<u8, (u8, u8, [u16; 16])>,
}

impl AudioRender {
    pub fn new(
        count: usize,
        max_polyphony: u16,
        source_sample_rate: f32,
        target_sample_rate: f32,
        scoring: ScoringConfig,
        rx: Receiver<AudioRenderActions>,
    ) -> Self {
        Self {
            tone_generators: (0..count)
                .map(|_| {
                    ToneGenerator::new(source_sample_rate, target_sample_rate, scoring.clone())
                })
                .collect(),
            rx,
            max_polyphony,
            shared: None,
            dc_l: super::dsp::core::dc_blocker::DcBlocker::new(),
            dc_r: super::dsp::core::dc_blocker::DcBlocker::new(),
            dc_enabled: true,
            sink: Box::new(VecBufferSink::new()),
            sample_rate: target_sample_rate,
            reverb: build_reverb(target_sample_rate, &[0; 16]),
            chorus: build_chorus(target_sample_rate, &[0; 16]),
            variation: build_variation(
                XGVariationType::NoEffect,
                &[0; 16],
                target_sample_rate,
            ),
            multi_eq: MultiEqDsp::new(),
            reverb_key: (0, 0, [0; 16]),
            chorus_key: (0, 0, [0; 16]),
            variation_key: (0, 0, [0; 16]),
            multi_eq_key: (0, EQBand::default(), EQBand::default(), EQBand::default(), EQBand::default(), EQBand::default()),
            insertion_instances: HashMap::new(),
            insertion_key: HashMap::new(),
        }
    }

    /// 替换输出后端 (启动时按 cfg.audio.engine 选择)
    pub fn set_sink(&mut self, sink: Box<dyn AudioSink>) {
        self.sink = sink;
    }

    /// 输出一帧块积累的样本到后端
    pub fn flush(&mut self) {
        self.sink.flush();
    }

    pub fn get_current_polyphony(&self) -> usize {
        self.tone_generators
            .iter()
            .filter(|&t| t.status == Running)
            .count()
    }

    pub fn find_all_tone_generators_by_channel(
        &mut self,
        channel: u8,
    ) -> Box<[&mut ToneGenerator]> {
        self.tone_generators
            .iter_mut()
            .filter(|t| t.bonded_to_channel(channel))
            .collect()
    }
}
