//! End-to-end integration test: MIDI NoteOn → Engine → AudioRender → sink audio output

use std::collections::HashMap;
use std::sync::mpsc::sync_channel;

use libmadaha::SoundModuleType;

use crate::audio::AudioRender;
use crate::audio::sink::VecBufferSink;
use crate::config::{
    AudioConfig, AudioDepth, AudioEngine, Config, MidiConfig, ScoringConfig, SoundModuleConfig,
};
use crate::midi::Engine;
use crate::midi::event::MidiEvent;
use crate::midi::note::Note;

const TBL_BIN: &str = "/home/user/Projects/yxg50/from_veg/sxgbin41.tbl";
const TBL_DATA: &str = "/home/user/Projects/yxg50/from_veg/sxgwave4.tbl";

fn test_config() -> Config {
    Config {
        log_level: "warn".into(),
        sound_module: SoundModuleConfig {
            module_type: SoundModuleType::Syxg50,
            tbl_bin_file: TBL_BIN.into(),
            tbl_data_file: TBL_DATA.into(),
        },
        audio: AudioConfig {
            engine: AudioEngine::Alsa,
            sample_rate: 44100,
            depth: AudioDepth::F32bit,
            buffer_size: 256,
            interpolating: crate::audio::tone_generator::oscillator::InterpolatingMethods::Linear,
            device: None,
            channels: 2,
            master_volume: 1.0,
            soft_clip: false,
            dc_blocker: true,
            alsa_buffer_frames: None,
            jack_client_name: "madaha".to_string(),
        },
        midi: MidiConfig {
            poly_replicant: 100,
            max_polyphony: 64,
            device_id: 1,
            master_tune: 440.0,
            channel_size: 256,
            input_engine: crate::config::MidiInputEngine::Alsa,
            scoring: ScoringConfig {
                time_weight: 1000,
                protect_attack: 100,
                penalty_release: 1500,
                protect_sustain_pedal: 100,
                protect_drum: HashMap::new(),
                protect_non_looping: 500,
                notes_config: HashMap::new(),
                volume_config: HashMap::new(),
            },
        },
    }
}

fn setup() -> (Engine, AudioRender) {
    let (tx, rx) = sync_channel(256);
    let cfg = test_config();
    let engine = Engine::new(&cfg, tx);
    engine.send_audio_init();

    let ar = AudioRender::new(64, 64, 44100.0, 44100.0, cfg.midi.scoring.clone(), rx);
    (engine, ar)
}

/// The test thread's default 2MB stack is insufficient for voice table parsing (2.09M-slot traversal),
/// so run the test logic on a 256MB-stack thread
fn run_on_big_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn midi_note_flows_to_audio_output() {
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();

        // NoteOn: channel 0, C4, vel 100
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        // drain + render a number of frames
        for _ in 0..64 {
            ar.audio_render();
        }

        // voice allocated and Running (GM piano single element → 1 voice)
        assert_eq!(ar.get_current_polyphony(), 1, "unexpected polyphony count (dual-element misdetect?)");

        // sink has non-zero audio output
        let buffer: Vec<f32> = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        assert!(!buffer.is_empty(), "sink has no output frames");
        let peak = buffer.iter().fold(0.0f32, |acc, &v| acc.max(v.abs()));
        assert!(peak > 1e-6, "output is silent, peak={peak}");
        assert!(peak <= 4.0, "output abnormally amplified, peak={peak}");

        // NoteOff → release
        engine.on_event(MidiEvent::NoteOff {
            channel: 0,
            note: Note::C4,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }
        // After NoteOff the voice should leave Running (enters Releasing; release not yet finished is normal)
        let still_running = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == crate::audio::tone_generator::ToneGeneratorStatus::Running)
            .count();
        assert_eq!(still_running, 0, "voices still Running after release");
    });
}

#[test]
fn drum_channel_ignores_pitchbend_and_portamento() {
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        // 先把 part 0 设为鼓通道 (rcv_channel=9, channel 10)
        engine.ram.xg.multi_part[0].write_with(|m| m.rcv_channel = 9);

        // Pitch Bend → 鼓通道忽略 (pitchbend 保持中心); 事件发到 channel 9 (rcv_channel=9)
        engine.on_event(MidiEvent::PitchBend { channel: 9, value: 0x3FFF });
        for _ in 0..4 {
            ar.audio_render();
        }
        let pb = engine.parts[0].snapshot().pitchbend;
        assert_eq!(pb, 0x2000, "drum channel must ignore pitch bend, pb={pb}");

        // CC#65 (Portamento Switch) → 鼓通道忽略
        engine.on_event(MidiEvent::ControlChange {
            channel: 9,
            controller: 65,
            value: 127,
        });
        let port = engine.ram.xg.multi_part[0].snapshot().portamento_switch;
        assert_eq!(port, 0, "drum channel must ignore CC#65, port={port}");

        // 对照: 旋律通道 (part 1, rcv_channel=1) 响应 pitch bend
        engine.on_event(MidiEvent::PitchBend { channel: 1, value: 0x3FFF });
        let pb2 = engine.parts[1].snapshot().pitchbend;
        assert_eq!(pb2, 0x3FFF, "melodic channel must respond to pitch bend");

        // Modulation (CC#1): Spec 无鼓特例 → 鼓通道照常响应
        engine.on_event(MidiEvent::ControlChange {
            channel: 9,
            controller: 1,
            value: 100,
        });
        let mod_val = engine.parts[0].snapshot().controller.modulation;
        assert_eq!(mod_val, 100, "drum channel must respond to modulation (Spec: no drum exception)");

        // YAMAHA 列表: 鼓通道无效果的控制器
        // CC#67 Soft Pedal → 忽略
        engine.on_event(MidiEvent::ControlChange {
            channel: 9,
            controller: 67,
            value: 127,
        });
        assert!(
            !engine.parts[0].snapshot().controller.soft_pedal,
            "drum channel must ignore CC#67 (soft pedal)"
        );
        // CC#32 Bank Select LSB → 忽略
        engine.on_event(MidiEvent::ControlChange {
            channel: 9,
            controller: 32,
            value: 100,
        });
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().bank_select_lsb,
            0,
            "drum channel must ignore CC#32 (bank select LSB)"
        );
        // CC#126 Mono → 忽略 (mode 不改变)
        engine.on_event(MidiEvent::ControlChange {
            channel: 9,
            controller: 126,
            value: 1,
        });
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().mode,
            1,
            "drum channel must ignore CC#126 (mono)"
        );
        // Poly After Touch → 忽略
        engine.on_event(MidiEvent::PolyPressure {
            channel: 9,
            note: Note::C4,
            pressure: 100,
        });
        assert_eq!(
            engine.parts[0].snapshot().pat_values[Note::C4 as usize],
            0,
            "drum channel must ignore poly aftertouch"
        );
        // Sustain (CC#64): 不在 YAMAHA 忽略列表 → 响应
        engine.on_event(MidiEvent::ControlChange {
            channel: 9,
            controller: 64,
            value: 127,
        });
        assert!(
            engine.parts[0].snapshot().controller.sustain,
            "drum channel must respond to sustain (not in YAMAHA ignore list)"
        );
    });
}

/// 构造 GM2 Universal SysEx 事件 (data 已剥离 F0/厂商 ID)
fn gm2_event(data: &[u8]) -> crate::midi::event::MidiEvent {
    use crate::midi::event::MidiEvent;
    use crate::midi::sysex::ManufacturerId;
    MidiEvent::SysEx {
        manufacturer_id: ManufacturerId::UniversalNonRealTime,
        data: data.into(),
    }
}

/// 构造 GM2 Realtime Universal SysEx 事件 (7F)
fn gm2_rt_event(data: &[u8]) -> crate::midi::event::MidiEvent {
    use crate::midi::event::MidiEvent;
    use crate::midi::sysex::ManufacturerId;
    MidiEvent::SysEx {
        manufacturer_id: ManufacturerId::UniversalRealTime,
        data: data.into(),
    }
}

#[test]
fn gm2_sysex_mapping() {
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        let sy = |d: &[u8]| -> Vec<u8> {
            let mut v = vec![0x7F]; // dev id
            v.extend_from_slice(d);
            v
        };
        let rty = |d: &[u8]| -> Vec<u8> {
            let mut v = vec![0x7F];
            v.extend_from_slice(d);
            v
        };

        // 1. Master Volume: 7E 04 01 01 <LSB> <MSB>
        engine.on_event(gm2_event(&sy(&[0x04, 0x01, 0x01, 0x00, 0x20])));
        assert_eq!(engine.master_volume, 0x2000, "GM2 master volume");
        assert_eq!(*engine.audio_master_volume.snapshot(), 0x2000, "GM2 master volume (audio)");

        // 2. Master Coarse Tuning: 7E 04 02 01 <semi>
        engine.on_event(gm2_event(&sy(&[0x04, 0x02, 0x01, 0x2C]))); // -12 semitones
        assert_eq!(engine.ram.xg.system.snapshot().transpose, 0x2C, "GM2 coarse tuning");

        // 3. Master Fine Tuning: 7E 04 03 01 <LSB> <MSB>
        engine.on_event(gm2_event(&sy(&[0x04, 0x03, 0x01, 0x00, 0x30])));
        assert_eq!(engine.master_tuning, 0x3000, "GM2 fine tuning");

        // 4. Scale/Octave Tuning: 7E 08 01 <note> <adj>
        engine.on_event(gm2_event(&sy(&[0x08, 0x01, 60, 0x50]))); // C4 +16
        let scale = engine.ram.xg.multi_part[0].snapshot().scale_tuning[0];
        assert_eq!(scale, 0x50, "GM2 scale tuning applied to all parts");

        // 5. Reverb Params: 7F 04 05 01 01 01 01 01 <pp> <vv> (fx=01@data[7], pp@8, vv@9)
        engine.on_event(gm2_rt_event(&rty(&[0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x02]))); // type=Room1
        let fx = engine.ram.xg.effect1.snapshot();
        assert_eq!(fx.reverb.type_msb, 0x02, "GM2 reverb type");
        engine.on_event(gm2_rt_event(&rty(&[0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 40]))); // time
        assert_eq!(engine.ram.xg.effect1.snapshot().reverb.param1, 40, "GM2 reverb time");

        // 6. Chorus Params: 7F 04 05 01 01 01 01 02 <pp> <vv> (fx=02@data[7])
        engine.on_event(gm2_rt_event(&rty(&[0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x02, 0x00, 0x03]))); // type=Chorus4
        assert_eq!(engine.ram.xg.effect1.snapshot().chorus.type_msb, 0x41, "GM2 chorus type");
        engine.on_event(gm2_rt_event(&rty(&[0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x02, 0x01, 60]))); // rate
        assert_eq!(engine.ram.xg.effect1.snapshot().chorus.param1, 60, "GM2 chorus rate");

        // 7. Key-Based Controllers: 7F 0A 01 <0n> <kk> <nn> <vv> (drum setup path)
        // 先把 part 0 设为鼓 (part_mode=2 → setup 0), note 60 → note_idx 48
        engine.ram.xg.multi_part[0].write_with(|m| m.part_mode = 2);
        // 触发 swap 使 part_mode 对后续处理可见
        engine.on_event(gm2_event(&sy(&[0x7F, 0x7F])));
        engine.on_event(gm2_rt_event(&rty(&[0x0A, 0x01, 0x00, 60, 0x07, 100]))); // volume
        assert_eq!(
            engine.ram.xg.drum_setup.snapshot()[0][48].level,
            100,
            "GM2 key-based volume"
        );
        engine.on_event(gm2_rt_event(&rty(&[0x0A, 0x01, 0x00, 60, 0x0A, 80]))); // pan
        assert_eq!(
            engine.ram.xg.drum_setup.snapshot()[0][48].pan,
            80,
            "GM2 key-based pan"
        );

        // 8. Channel Pressure Destination: 7F 09 01 <0n> <pp> <rr>
        engine.on_event(gm2_rt_event(&rty(&[0x09, 0x01, 0x00, 0x01, 100]))); // pitch control
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().cat.pitch_control,
            100,
            "GM2 channel pressure destination"
        );

        // 9. CC Destination: 7F 09 03 <0n> <cc> <pp> <rr>
        engine.on_event(gm2_rt_event(&rty(&[0x09, 0x03, 0x00, 0x4C, 0x01, 90]))); // CC#76 → pitch
        let m = engine.ram.xg.multi_part[0].snapshot();
        assert_eq!(m.ac[0].controller_number, 0x4C, "GM2 CC destination cc");
        assert_eq!(m.ac[0].pitch_control, 90, "GM2 CC destination depth");

        // 10. Master Volume 叠加渲染: 半音量 → 输出峰值减半
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        let peak_at = |ar: &mut AudioRender| -> f32 {
            for _ in 0..64 {
                ar.audio_render();
            }
            ar.sink
                .as_any_mut()
                .downcast_mut::<VecBufferSink>()
                .map(|s| s.take_buffer())
                .unwrap()
                .iter()
                .fold(0.0f32, |a, &v| a.max(v.abs()))
        };
        engine.audio_master_volume.write_with(|v| *v = 0x4000);
        let _ = engine.on_event(gm2_event(&sy(&[0x7F, 0x7F]))); // unknown sub → swap only
        let p_full = peak_at(&mut ar);
        engine.audio_master_volume.write_with(|v| *v = 0x2000);
        let _ = engine.on_event(gm2_event(&sy(&[0x7F, 0x7F]))); // unknown sub → swap only
        let p_half = peak_at(&mut ar);
        assert!(p_half > 0.0, "half volume peak must be non-zero");
        assert!(
            (p_half / p_full - 0.5).abs() < 0.1,
            "GM2 master volume must halve output: full={p_full} half={p_half}"
        );
    });
}

#[test]
fn active_sensing_never_beat_stays_inactive() {
    run_on_big_stack(|| {
        let (mut engine, _ar) = setup();

        // 设置控制器值 (模拟用户输入)
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 1,
            value: 100,
        });
        assert_eq!(engine.parts[0].snapshot().controller.modulation, 100);

        // 从未发送 Active Sensing → watchdog 不应激活、不应重置
        std::thread::sleep(std::time::Duration::from_millis(750));
        assert!(
            !engine.active_sensing.is_active(),
            "no heartbeat ever sent → watchdog must stay inactive"
        );
        assert_eq!(
            engine.parts[0].snapshot().controller.modulation,
            100,
            "parts must NOT be reset without a heartbeat"
        );
    });
}

#[test]
fn active_sensing_heartbeat_and_timeout_reset() {
    run_on_big_stack(|| {
        let (mut engine, _ar) = setup();

        // 心跳前: 未激活
        assert!(!engine.active_sensing.is_active());

        // 收到 Active Sensing (0xFE) → 激活
        engine.on_event(MidiEvent::ActiveSensing);
        assert!(engine.active_sensing.is_active(), "heartbeat must activate");

        // 设置控制器/RPN 值, 验证超时后被复位
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 1,
            value: 100,
        });
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 11,
            value: 90,
        });
        engine.on_event(MidiEvent::RPN {
            channel: 0,
            parameter: 0x0000,
            value: 0x3C00, // bend sensitivity 12
        });
        let snap = engine.parts[0].snapshot();
        assert_eq!(snap.controller.modulation, 100);
        assert_eq!(snap.controller.expression, 90);
        assert_eq!(snap.rpn.pitchbend_cents, 0x00);

        // 停止心跳 → watchdog (500ms) 超时 → 自动重置
        std::thread::sleep(std::time::Duration::from_millis(750));
        assert!(
            !engine.active_sensing.is_active(),
            "watchdog must deactivate after heartbeat timeout"
        );
        let reset = engine.parts[0].snapshot();
        assert_eq!(
            reset.controller.modulation, 0,
            "timeout must reset part controllers"
        );
        assert_eq!(
            reset.controller.expression, 0x7F,
            "timeout must reset expression"
        );
        assert_eq!(
            reset.rpn.pitchbend_cents, 0,
            "timeout must reset RPN state"
        );

        // 恢复心跳 → 重新激活
        engine.on_event(MidiEvent::ActiveSensing);
        assert!(engine.active_sensing.is_active(), "re-heartbeat must reactivate");
    });
}

#[test]
fn system_effects_shared_init() {
    run_on_big_stack(|| {
        let (_engine, mut ar) = setup();
        for _ in 0..4 {
            ar.audio_render();
        }
        // Init event delivered → shared is ready
        assert!(ar.shared.is_some(), "AudioShared not initialized");
        let shared = ar.shared.as_ref().unwrap();
        // double-buffered parameters are readable
        let sys = shared.system.snapshot();
        assert_eq!(sys.master_volume, 0x7F, "unexpected master_volume default");
        let fx = shared.effect1.snapshot();
        // XG default: Reverb = Hall1 (msb=1)
        assert_eq!(fx.reverb.type_msb, 1, "unexpected reverb default type");
    });
}
