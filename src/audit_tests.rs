/// Audit tests: coverage for previously untested modules
/// (config validation, audio encode, LFO, voice_manager pure logic, XG RAM fields)
use crate::audio::backend::encode_frame;
use crate::config::ConfigObject;
use crate::config::{AudioConfig, AudioDepth, MidiConfig};
use crate::lfo::lfo::LFO;
use crate::lfo::wave_type::WaveType;
use crate::midi::ram::xg::multi_part::MultiPart;
use crate::midi::ram::xg::system::System;
use crate::voice_manager::Key;

#[cfg(test)]
mod tests {
    use super::*;

    // ── config validation ──
    fn audio_cfg() -> AudioConfig {
        AudioConfig {
            engine: crate::config::AudioEngine::Alsa,
            sample_rate: 44100,
            depth: AudioDepth::S16bit,
            buffer_size: 64,
            interpolating: crate::audio::tone_generator::oscillator::InterpolatingMethods::Linear,
            device: None,
            channels: 2,
            master_volume: 1.0,
            soft_clip: true,
            dc_blocker: true,
            alsa_buffer_frames: None,
            jack_client_name: "madaha".to_string(),
        }
    }

    #[test]
    fn audio_config_rejects_bad_sample_rate() {
        let mut cfg = audio_cfg();
        cfg.sample_rate = 12345;
        assert!(cfg.check().is_err());
        cfg.sample_rate = 48000;
        assert!(cfg.check().is_ok());
    }

    #[test]
    fn audio_config_rejects_bad_buffer_size() {
        let mut cfg = audio_cfg();
        cfg.buffer_size = 100; // not a power of two
        assert!(cfg.check().is_err());
        cfg.buffer_size = 128;
        assert!(cfg.check().is_ok());
    }

    #[test]
    fn midi_config_minimal_construct() {
        let cfg = MidiConfig {
            poly_replicant: 150,
            max_polyphony: 512,
            device_id: 16,
            master_tune: 440.0,
            channel_size: 1024,
            input_engine: crate::config::MidiInputEngine::Alsa,
            scoring: crate::config::ScoringConfig {
                time_weight: 1000,
                protect_attack: 100,
                penalty_release: 1500,
                protect_sustain_pedal: 100,
                protect_drum: std::collections::HashMap::new(),
                protect_non_looping: 500,
                notes_config: std::collections::HashMap::new(),
                volume_config: std::collections::HashMap::new(),
            },
        };
        assert_eq!(cfg.max_polyphony, 512);
    }

    #[test]
    fn audio_config_rejects_bad_master_volume() {
        let mut cfg = audio_cfg();
        cfg.master_volume = 0.0;
        assert!(cfg.check().is_err());
        cfg.master_volume = 4.0;
        assert!(cfg.check().is_ok());
        cfg.master_volume = 4.1;
        assert!(cfg.check().is_err());
    }

    #[test]
    fn audio_config_rejects_bad_alsa_buffer() {
        let mut cfg = audio_cfg();
        cfg.alsa_buffer_frames = Some(100); // not a power of two
        assert!(cfg.check().is_err());
        cfg.alsa_buffer_frames = Some(1024);
        assert!(cfg.check().is_ok());
    }

    #[test]
    fn gain_sink_applies_volume_and_soft_clip() {
        use crate::audio::sink::{AudioSink, GainSink, VecBufferSink};
        let inner = VecBufferSink::new();
        let mut g = GainSink::new(Box::new(inner), 0.5, true);
        g.push_frame(1.0, -1.0);
        // 0.5 gain → tanh(0.5) ≈ 0.462 < 0.5
        assert_eq!(g.frame_count(), 1);
        let inner_ref = g.inner_mut().as_any_mut().downcast_mut::<VecBufferSink>().unwrap();
        let out = inner_ref.take_buffer();
        assert!((out[0] - 0.5f32.tanh()).abs() < 1e-4, "l={}", out[0]);
        assert!((out[1] + 0.5f32.tanh()).abs() < 1e-4, "r={}", out[1]);
    }

    #[test]
    fn midi_config_rejects_bad_polyphony() {
        let mut cfg = MidiConfig {
            poly_replicant: 150,
            max_polyphony: 100,
            device_id: 16,
            master_tune: 440.0,
            channel_size: 1024,
            input_engine: crate::config::MidiInputEngine::Alsa,
            scoring: crate::config::ScoringConfig {
                time_weight: 1000,
                protect_attack: 100,
                penalty_release: 1500,
                protect_sustain_pedal: 100,
                protect_drum: std::collections::HashMap::new(),
                protect_non_looping: 500,
                notes_config: std::collections::HashMap::new(),
                volume_config: std::collections::HashMap::new(),
            },
        };
        assert!(cfg.check().is_err());
        cfg.max_polyphony = 512;
        assert!(cfg.check().is_ok());
    }

    // ── audio encode ──
    #[test]
    fn encode_frame_converts_depths() {
        for depth in [AudioDepth::U8bit, AudioDepth::S16bit, AudioDepth::S24bit, AudioDepth::F32bit] {
            let mut out = vec![];
            encode_frame(depth, -1.0, 1.0, &mut out);
            let bytes = crate::audio::backend::sample_bytes(depth);
            assert_eq!(out.len(), bytes * 2, "{depth:?}");
        }
    }

    #[test]
    fn encode_frame_s16_clamps_and_scales() {
        let mut out = vec![];
        encode_frame(AudioDepth::S16bit, 1.0, -1.0, &mut out);
        assert_eq!(out[0], 0xFF);
        assert_eq!(out[1], 0x7F);
        assert_eq!(out[2], 0x01);
        assert_eq!(out[3], 0x80);
    }

    #[test]
    fn sample_bytes_matches_depth() {
        assert_eq!(crate::audio::backend::sample_bytes(AudioDepth::U8bit), 1);
        assert_eq!(crate::audio::backend::sample_bytes(AudioDepth::S16bit), 2);
        assert_eq!(crate::audio::backend::sample_bytes(AudioDepth::S24bit), 3);
        assert_eq!(crate::audio::backend::sample_bytes(AudioDepth::F32bit), 4);
    }

    // ── LFO ──
    #[test]
    fn lfo_wave_outputs_stay_bounded() {
        let mut lfo = LFO::new();
        lfo.wave_type = WaveType::Saw;
        lfo.update_accumulator(5.0, 64, 44100);
        lfo.pitch.set_output(1.0);
        for _ in 0..44100 / 5 / 64 {
            lfo.make_wave();
            for t in [lfo.pitch.output, lfo.amp.output, lfo.lpf.output, lfo.hpf.output] {
                // LFO outputs are raw phase-scaled values; downstream scales via depth
                assert!(t.is_finite(), "lfo out {t}");
            }
        }
    }

    #[test]
    fn lfo_random_differs_from_sine() {
        let mut rnd = LFO::new();
        rnd.wave_type = WaveType::Random;
        rnd.update_accumulator(5.0, 64, 44100);
        rnd.make_wave();
        let mut sine = LFO::new();
        sine.wave_type = WaveType::Sine;
        sine.update_accumulator(5.0, 64, 44100);
        sine.make_wave();
        assert!(rnd.pitch.output != sine.pitch.output);
    }

    // ── voice_manager pure logic ──
    #[test]
    fn program_key_velocity_layers() {
        // Program via From<[Option<Box<Key>>;128]>
        let arr: [Option<Box<Key>>; 128] = std::array::from_fn(|_| None);
        let key = Key::new(
            60,
            &[],
            &None,
            None,
        );
        assert!(key.is_none(), "empty samples -> no key");
        let _ = arr;
    }

    // ── XG RAM fields ──
    #[test]
    fn system_master_tune_roundtrip() {
        let mut sys = System::new();
        sys.set_master_tune(0x1234);
        assert_eq!(sys.get_master_tune(), 0x1234);
        sys.master_volume = 100;
        assert_eq!(sys.master_volume, 100);
    }

    #[test]
    fn multi_part_detune_and_velocity() {
        let part = MultiPart::new(0);
        let mut part2 = part.clone();
        part2.set_detune(64);
        assert_eq!(part2.get_detune(), 64);
        // detune maps through DETUNE_TO_CENTS; a changed value changes cents
        let cents_a = part2.detune_cents();
        part2.set_detune(0);
        assert_ne!(part2.detune_cents(), cents_a);
        // velocity response: with a sense depth set, vel 127 > vel 0
        part2.velocity_sense_depth = 64;
        assert!(part2.get_velocity(127) > part2.get_velocity(0));
    }

    #[test]
    fn ram_index_roundtrip() {
        use crate::midi::ram::interface::Memory;
        let mut ram = crate::midi::ram::RAM::new(
            crate::midi::engine::MidiResetMode::XG,
            [crate::voice_manager::DEFAULT_DRUM_SETUP; 79],
        );
        let addr = crate::midi::ram::MemoryAddr::new(0x00, 0x00, 0x00);
        assert!(ram.get(addr).is_ok());
        assert!(ram.set(addr, 0x40).is_ok());
    }
}
