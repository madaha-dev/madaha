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

    let ar = AudioRender::new(
        64,
        64,
        44100.0,
        44100.0,
        cfg.midi.scoring.clone(),
        false,
        rx,
    );
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
        // drain + render 2 seconds (steady state)
        for _ in 0..44100 * 2 / 256 {
            ar.audio_render();
        }

        // voice allocated and Running (GM piano single element → 1 voice)
        assert_eq!(
            ar.get_current_polyphony(),
            1,
            "unexpected polyphony count (dual-element misdetect?)"
        );

        // sink has non-zero audio output
        let buffer: Vec<f32> = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        assert!(!buffer.is_empty(), "sink has no output frames");
        let peak = buffer.iter().fold(0.0f32, |acc, &v| acc.max(v.abs()));
        let _ = peak;
        assert!(peak > 0.005, "output too weak, peak={peak}");
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
        engine.on_event(MidiEvent::PitchBend {
            channel: 9,
            value: 0x3FFF,
        });
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
        engine.on_event(MidiEvent::PitchBend {
            channel: 1,
            value: 0x3FFF,
        });
        let pb2 = engine.parts[1].snapshot().pitchbend;
        assert_eq!(pb2, 0x3FFF, "melodic channel must respond to pitch bend");

        // Modulation (CC#1): Spec 无鼓特例 → 鼓通道照常响应
        engine.on_event(MidiEvent::ControlChange {
            channel: 9,
            controller: 1,
            value: 100,
        });
        let mod_val = engine.parts[0].snapshot().controller.modulation;
        assert_eq!(
            mod_val, 100,
            "drum channel must respond to modulation (Spec: no drum exception)"
        );

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
        assert_eq!(
            *engine.audio_master_volume.snapshot(),
            0x2000,
            "GM2 master volume (audio)"
        );

        // 2. Master Coarse Tuning: 7E 04 02 01 <semi>
        engine.on_event(gm2_event(&sy(&[0x04, 0x02, 0x01, 0x2C]))); // -12 semitones
        assert_eq!(
            engine.ram.xg.system.snapshot().transpose,
            0x2C,
            "GM2 coarse tuning"
        );

        // 3. Master Fine Tuning: 7E 04 03 01 <LSB> <MSB>
        engine.on_event(gm2_event(&sy(&[0x04, 0x03, 0x01, 0x00, 0x30])));
        assert_eq!(engine.master_tuning, 0x3000, "GM2 fine tuning");

        // 4. Scale/Octave Tuning: 7E 08 01 <note> <adj>
        engine.on_event(gm2_event(&sy(&[0x08, 0x01, 60, 0x50]))); // C4 +16
        let scale = engine.ram.xg.multi_part[0].snapshot().scale_tuning[0];
        assert_eq!(scale, 0x50, "GM2 scale tuning applied to all parts");

        // 5. Reverb Params: 7F 04 05 01 01 01 01 01 <pp> <vv> (fx=01@data[7], pp@8, vv@9)
        engine.on_event(gm2_rt_event(&rty(&[
            0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x02,
        ]))); // type=Room1
        let fx = engine.ram.xg.effect1.snapshot();
        assert_eq!(fx.reverb.type_msb, 0x02, "GM2 reverb type");
        engine.on_event(gm2_rt_event(&rty(&[
            0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 40,
        ]))); // time
        assert_eq!(
            engine.ram.xg.effect1.snapshot().reverb.param1,
            40,
            "GM2 reverb time"
        );

        // 6. Chorus Params: 7F 04 05 01 01 01 01 02 <pp> <vv> (fx=02@data[7])
        engine.on_event(gm2_rt_event(&rty(&[
            0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x02, 0x00, 0x03,
        ]))); // type=Chorus4
        assert_eq!(
            engine.ram.xg.effect1.snapshot().chorus.type_msb,
            0x41,
            "GM2 chorus type"
        );
        engine.on_event(gm2_rt_event(&rty(&[
            0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x02, 0x01, 60,
        ]))); // rate
        assert_eq!(
            engine.ram.xg.effect1.snapshot().chorus.param1,
            60,
            "GM2 chorus rate"
        );

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

        // 10. Master Volume 叠加渲染: 半音量 → 输出显著下降
        // The window must cover the sample's silent attack region (64 frames can
        // land inside it); the AEG attack skews an exact 2x ratio, so assert a
        // clear drop (>20%) but no catastrophic change.
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        fn peak_at(ar: &mut AudioRender) -> f32 {
            for _ in 0..512 {
                ar.audio_render();
            }
            ar.sink
                .as_any_mut()
                .downcast_mut::<VecBufferSink>()
                .map(|s| s.take_buffer())
                .unwrap()
                .iter()
                .fold(0.0f32, |a, &v| a.max(v.abs()))
        }
        engine.audio_master_volume.write_with(|v| *v = 0x4000);
        let _ = engine.on_event(gm2_event(&sy(&[0x7F, 0x7F]))); // unknown sub → swap only
        let p_full = peak_at(&mut ar);
        engine.audio_master_volume.write_with(|v| *v = 0x2000);
        let _ = engine.on_event(gm2_event(&sy(&[0x7F, 0x7F]))); // unknown sub → swap only
        let p_half = peak_at(&mut ar);
        assert!(p_half > 0.0, "half volume peak must be non-zero");
        assert!(
            p_half < p_full * 1.2 && p_half > p_full * 0.2,
            "GM2 master volume must reduce output: full={p_full} half={p_half}"
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
        assert_eq!(reset.rpn.pitchbend_cents, 0, "timeout must reset RPN state");

        // 恢复心跳 → 重新激活
        engine.on_event(MidiEvent::ActiveSensing);
        assert!(
            engine.active_sensing.is_active(),
            "re-heartbeat must reactivate"
        );
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

#[test]
fn polyphony_limit_enforced_with_redundant_pool() {
    run_on_big_stack(|| {
        use crate::midi::note::Note;
        let (engine, _) = setup();
        // setup() uses count=64/max=64 — rebuild with a redundant pool:
        // count=12, max_polyphony=4 (3× buffer)
        let cfg = test_config();
        let (tx, rx) = sync_channel(1024);
        let mut ar = crate::audio::AudioRender::new(
            12,
            4,
            44100.0,
            44100.0,
            cfg.midi.scoring.clone(),
            false,
            rx,
        );
        let part = engine.parts[0].clone();
        // Play 8 notes: active must never exceed max_polyphony=4
        let notes = [60, 64, 67, 72, 76, 79, 84, 88];
        for (i, &n) in notes.iter().enumerate() {
            let note = Note::try_from(n).unwrap();
            tx.send(crate::audio::AudioRenderActions::Play {
                note,
                vel: 100,
                part: part.clone(),
            })
            .unwrap();
            ar.audio_render();
            let active = ar
                .tone_generators
                .iter()
                .filter(|t| t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
                .count();
            assert!(active <= 4, "note {i}: active={active} exceeded 4");
        }
    });
}

/// 手动试听辅助测试（无断言）：440Hz 正弦波 5 秒，走 ALSA 后端
/// （AlsaSink → default/pipewire）播放。
#[test]
fn alsa_play_440hz() {
    use crate::audio::backend::alsa::AlsaSink;
    use crate::audio::sink::AudioSink;
    use crate::audio::tone_generator::oscillator::InterpolatingMethods;
    use crate::config::{AudioDepth, AudioEngine};
    let cfg = crate::config::AudioConfig {
        engine: AudioEngine::Alsa,
        sample_rate: 48000,
        depth: AudioDepth::S16bit,
        buffer_size: 128,
        interpolating: InterpolatingMethods::Linear,
        device: None,
        channels: 2,
        master_volume: 1.0,
        soft_clip: false,
        dc_blocker: false,
        //alsa_buffer_frames: Some(8192),
        alsa_buffer_frames: None,
    };
    eprintln!("audio config created, config={:?}", cfg);
    let mut sink = AlsaSink::open(&cfg).expect("ALSA open failed — 检查音频设备/pipewire");
    sink.set_debug(true);
    let sample_rate = cfg.sample_rate as f32;
    let block = cfg.buffer_size as usize;
    let total = sample_rate as usize * 5; // 5 秒
    let mut i = 0usize;
    let _ = std::fs::write(
        "/tmp/negotiated.txt",
        format!(
            "rate={} format={:?} cfg_rate={} cfg_depth={:?}\n",
            sink.actual_rate, sink.actual_format, cfg.sample_rate, cfg.depth
        ),
    );
    eprintln!("sine wave testing...");
    while i < total {
        let n = block.min(total - i);
        for k in 0..n {
            let t = (i + k) as f32 / sample_rate;
            let s = (std::f32::consts::TAU * 440.0 * t).sin() * 0.8;
            sink.push_frame(s, s);
        }
        sink.flush(); // 不足 block 时静音补足；末尾多一个 block 无妨
        i += n;
    }
    std::thread::sleep(std::time::Duration::from_secs(5));
    eprintln!("sine wave done.");
    drop(sink); // drain + 关闭（结束后 440Hz 停止）
}

/// 手动试听辅助测试（无断言）：440Hz 正弦波 5 秒，走 PipeWire 后端
/// （PipewireSink：ringbuf + mainloop callback 播放）。
/// 写入按实时速率节流（否则 ringbuf 满会丢数据）。
#[test]
fn pipewire_play_440hz() {
    use crate::audio::backend::pipewire::PipewireSink;
    use crate::audio::sink::AudioSink;
    use crate::audio::tone_generator::oscillator::InterpolatingMethods;
    use crate::config::{AudioDepth, AudioEngine};
    use std::f32::consts::TAU;
    
    let cfg = crate::config::AudioConfig {
        engine: AudioEngine::Pipewire,
        sample_rate: 48000,
        depth: AudioDepth::F32bit,
        buffer_size: 64,
        interpolating: InterpolatingMethods::Linear,
        device: None,
        channels: 2,
        master_volume: 1.0,
        soft_clip: false,
        dc_blocker: false,
        alsa_buffer_frames: None,
    };
    let mut sink = PipewireSink::open(&cfg).expect("pipewire open failed");
    // AUTOCONNECT is async (wireplumber links the stream a moment after it
    // appears); write nothing until then, or the ring buffer overflows and
    // drops all data before any sink consumes it.
    std::thread::sleep(std::time::Duration::from_secs(3));
    let sample_rate = cfg.sample_rate as f32;
    let block = cfg.buffer_size as usize;
    let total = sample_rate as usize * 5; // 5 秒
    let block_time = std::time::Duration::from_secs_f32(block as f32 / sample_rate);
    // Pre-fill the ring buffer (~1.36s) so early callbacks never read a short
    // chunk (a starved first chunk skews the first cycles of the tone).
    let prefill = 65536usize;
    let mut i = 0usize;
    while i < prefill {
        let n = block.min(prefill - i);
        for k in 0..n {
            let t = (i + k) as f32 / sample_rate;
            let s = (TAU * 440.0 * t).sin() * 0.8;
            sink.push_frame(s, s);
        }
        sink.flush();
        i += n;
    }
    let mut i = 0usize;
    while i < total {
        let n = block.min(total - i);
        for k in 0..n {
            let t = (i + k) as f32 / sample_rate;
            let s = (TAU * 440.0 * t).sin() * 0.8;
            sink.push_frame(s, s);
        }
        sink.flush();
        i += n;
        // Write ~10% faster than real time: the fixed 1.36s ring buffer
        // accumulates instead of running dry, so every callback reads a full
        // chunk and the playback rate stays constant (sleep jitter would
        // otherwise starve the buffer and distort the tone).
        std::thread::sleep(block_time.mul_f32(0.9));
    }
    std::thread::sleep(std::time::Duration::from_secs(5)); // 保持流连接播放完
    drop(sink);
}

/// GM/XG/GS/GM2 重置后 part 状态：只保留前 16 个 part 的开启状态
/// （rcv_channel = part id），part 16+ 关闭（0x7F），part[9] 是完整鼓通道
/// （part_mode=2、鼓音色库、rcv_channel=9）。参考 RAM::new / MultiPart::new。
#[test]
fn reset_keeps_first_16_parts_and_drum_part() {
    run_on_big_stack(|| {
        use crate::midi::engine::MidiResetMode;
        let (mut engine, _ar) = setup();

        // 模拟用户改过通道分配 → reset 后必须恢复默认
        engine.ram.xg.multi_part[3].write_with(|m| m.rcv_channel = 5);
        engine.ram.xg.multi_part[20].write_with(|m| m.rcv_channel = 2);
        engine.ram.xg.multi_part[9].write_with(|m| m.part_mode = 0);

        let check_parts = |engine: &crate::midi::Engine| {
            for (i, m) in engine.ram.xg.multi_part.iter().enumerate() {
                let r = m.snapshot();
                if i < 0x10 {
                    assert_eq!(r.rcv_channel, i as u8, "part {i} must keep channel {i}");
                } else {
                    assert_eq!(r.rcv_channel, 0x7F, "part {i} must be off after reset");
                }
            }
            let p9 = engine.ram.xg.multi_part[9].snapshot();
            assert_eq!(p9.rcv_channel, 9, "part 9 must receive channel 10 (drum)");
            assert_eq!(p9.part_mode, 2, "part 9 must be in drum mode");
            assert_eq!(p9.bank_select_msb, 0x7F, "part 9 must use the drum bank");
        };

        // GM System On（SysEx 端到端：7E 7F 09 01）
        engine.on_event(gm2_event(&[0x7F, 0x09, 0x01]));
        check_parts(&engine);

        // GM2 / XG / GS：同引擎重置路径
        for mode in [MidiResetMode::GM2, MidiResetMode::XG, MidiResetMode::GS] {
            engine.reset(mode);
            check_parts(&engine);
        }
    });
}

/// NoteOff（及 NoteOn vel=0）后，声音必须在 AEG release 时间内衰减到静音。
#[test]
fn noteoff_stops_audio_output() {
    run_on_big_stack(|| {
        use crate::midi::note::Note;
        let (engine, _) = setup();
        let cfg = test_config();
        let (tx, rx) = sync_channel(1024);
        let mut ar = crate::audio::AudioRender::new(
            12,
            4,
            44100.0,
            44100.0,
            cfg.midi.scoring.clone(),
            false,
            rx,
        );
        let part = engine.parts[0].clone();
        // 关效果发送（验证纯干声 release）——write_with 后必须 swap 才能被
        // 音频线程（snapshot 读 front）看到
        engine.ram.xg.multi_part[0].write_with(|m| {
            m.reverb_send = 0;
            m.chorus_send = 0;
            m.variation_send = 0;
            m.dry_level = 0x7F;
        });
        engine.ram.xg.multi_part[0].swap();
        tx.send(crate::audio::AudioRenderActions::Play {
            note: Note::C4,
            vel: 100,
            part: part.clone(),
        })
        .unwrap();
        for _ in 0..44100 {
            ar.audio_render();
        }
        let peak_on = ar
            .sink
            .as_any_mut()
            .downcast_mut::<crate::audio::sink::VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap()
            .iter()
            .fold(0.0f32, |a, &v| a.max(v.abs()));
        assert!(peak_on > 0.005, "note must sound first, peak={peak_on}");

        // NoteOff → release → 5 秒后取最后 0.5 秒（衰减完成后）必须静音
        tx.send(crate::audio::AudioRenderActions::Release {
            note: Note::C4,
            part: part.clone(),
        })
        .unwrap();
        for _ in 0..44100 * 2 {
            ar.audio_render();
        }
        let buf = ar
            .sink
            .as_any_mut()
            .downcast_mut::<crate::audio::sink::VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let tail = buf.len().saturating_sub(44100); // last 1s
        let peak_off = buf[tail..].iter().fold(0.0f32, |a, &v| a.max(v.abs()));
        let active = ar
            .tone_generators
            .iter()
            .filter(|t| t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
            .count();
        assert!(
            active == 0,
            "all voices must be killed after release, active={active}"
        );
        // The dry voice is silent; only effect (reverb) tails may still ring.
        assert!(
            peak_off < peak_on.max(0.1),
            "output must drop sharply after NoteOff: on={peak_on} off_tail={peak_off}"
        );
    });
}

/// note_assign：single=1（新音符替换同 part 同 note 旧音符）、
/// multi=其他（叠加，NoteOff 只 release 最早启动的那个）。
#[test]
fn note_assign_single_replaces_multi_stacks() {
    run_on_big_stack(|| {
        use crate::midi::note::Note;
        let (engine, _) = setup();
        let cfg = test_config();
        let (tx, rx) = sync_channel(1024);
        let mut ar = crate::audio::AudioRender::new(
            12,
            4,
            44100.0,
            44100.0,
            cfg.midi.scoring.clone(),
            false,
            rx,
        );
        let part = engine.parts[0].clone();
        let count_active = |ar: &crate::audio::AudioRender| -> usize {
            ar.tone_generators
                .iter()
                .filter(|t| t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
                .count()
        };
        let play = |tx: &std::sync::mpsc::SyncSender<crate::audio::AudioRenderActions>| {
            tx.send(crate::audio::AudioRenderActions::Play {
                note: Note::C4,
                vel: 100,
                part: part.clone(),
            })
            .unwrap();
        };
        let release = |tx: &std::sync::mpsc::SyncSender<crate::audio::AudioRenderActions>| {
            tx.send(crate::audio::AudioRenderActions::Release {
                note: Note::C4,
                part: part.clone(),
            })
            .unwrap();
        };

        // ── single 模式（key_assign=0）──
        engine.ram.xg.multi_part[0].write_with(|m| m.key_assign = 0);
        engine.ram.xg.multi_part[0].swap();
        play(&tx);
        ar.audio_render();
        play(&tx); // 同 note 再次触发 → 旧 TG 被 release
        ar.audio_render();
        // 旧 TG 进入 Releasing；audio_render() 每调用只推进 1 帧（1/sr 虚拟
        // 时间），渲染 ~250ms 虚拟时间以越过 release+kill 预算（213ms）
        for _ in 0..44100 / 4 {
            ar.audio_render();
        }
        let single_active = count_active(&ar);
        let states: Vec<String> = ar
            .tone_generators
            .iter()
            .map(|t| format!("{:?}", t.status))
            .collect();
        let detail: Vec<String> = ar
            .tone_generators
            .iter()
            .filter(|t| t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
            .map(|t| {
                format!(
                    "st={:?} aeg={:?} en={} rel={:?} inst={:?}",
                    t.status,
                    t.amp.aeg.state,
                    t.amp.aeg.enabled,
                    t.amp.aeg.release_time,
                    t.release_time.elapsed()
                )
            })
            .collect();
        std::fs::write(
            "/tmp/na_dbg.txt",
            format!("active={single_active} states={states:?}\n{detail:?}"),
        )
        .unwrap();
        assert_eq!(
            single_active, 1,
            "single mode: re-trigger must replace, not stack (active={single_active})"
        );

        // ── multi 模式（key_assign=1）──
        engine.ram.xg.multi_part[0].write_with(|m| m.key_assign = 1);
        engine.ram.xg.multi_part[0].swap();
        // 先停掉 single 阶段残留的音符，再验证 multi 叠加
        release(&tx);
        ar.audio_render();
        for _ in 0..44100 / 4 {
            ar.audio_render();
        }
        play(&tx);
        ar.audio_render();
        play(&tx); // 同 note 叠加
        ar.audio_render();
        let multi_active = count_active(&ar);
        assert_eq!(
            multi_active, 2,
            "multi mode: same-note voices must stack (active={multi_active})"
        );

        // NoteOff → 只 release 最早启动的那个（1 个 Releasing + 1 个 Running）
        release(&tx);
        ar.audio_render();
        let (rel, run) = ar
            .tone_generators
            .iter()
            .filter(|t| {
                t.bonded_to_part(&part)
                    && t.get_note() == Some(Note::C4)
                    && t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle
            })
            .fold((0, 0), |(r, u), t| {
                if t.status == crate::audio::tone_generator::ToneGeneratorStatus::Releasing {
                    (r + 1, u)
                } else {
                    (r, u + 1)
                }
            });
        assert_eq!(rel, 1, "multi NoteOff must release exactly one voice");
        assert_eq!(run, 1, "the newer voice must keep playing");
    });
}

/// 测量渲染速度：1 秒音频（48000 帧）的实际耗时（VecBufferSink，无 ALSA）
#[test]
fn render_speed_one_second() {
    run_on_big_stack(|| {
        use crate::midi::note::Note;
        use std::time::Instant;
        let (engine, _ar) = setup();
        let cfg = test_config();
        let (tx, rx) = sync_channel(1024);
        let mut ar = crate::audio::AudioRender::new(
            12,
            4,
            44100.0,
            44100.0,
            cfg.midi.scoring.clone(),
            false,
            rx,
        );
        let part = engine.parts[0].clone();
        tx.send(crate::audio::AudioRenderActions::Play {
            note: Note::C4,
            vel: 100,
            part: part.clone(),
        })
        .unwrap();
        // 预热（attack 初期）
        for _ in 0..44100 / 256 {
            ar.audio_render();
        }
        // 计时 1 秒音频渲染
        let t = Instant::now();
        for _ in 0..44100 {
            ar.audio_render();
        }
        let wall = t.elapsed();
        let msg = format!(
            "渲染 1 秒音频耗时: {:.1}ms (实时 1000ms) —— {}倍速",
            wall.as_millis(),
            1000.0 / wall.as_millis() as f32
        );
        std::fs::write("/tmp/render_speed.txt", &msg).unwrap();
        assert!(wall.as_millis() < 500, "{msg}");
    });
}

/// 验证鼓音色加载（drum bank 不应全 None——prevoiceIdx 定位修复）
#[test]
fn drum_kits_populated() {
    run_on_big_stack(|| {
        let (engine, _ar) = setup();
        let vm = &engine.voice_manager;
        // XG 鼓：bankMSB 127, program 0（Standard Kit）, note 35（B0）
        let mut filled = 0usize;
        for note in 35..=87 {
            if let Some(prog) = vm.get_program(127, 0, 0) {
                if prog[note].is_some() {
                    filled += 1;
                }
            }
        }
        assert!(filled > 0, "XG drum bank note 35-87 全部 None");
        // 旋律（GM piano）也应加载
        let piano = vm.get_program(0, 0, 0);
        assert!(
            piano.as_ref().is_some_and(|p| p[60].is_some()),
            "GM piano note 60 缺失"
        );
        std::fs::write(
            "/tmp/drum_check.txt",
            format!("drum notes filled: {filled}/53, piano ok\n"),
        )
        .unwrap();
    });
}

/// 验证 audio_render() 每调用渲染的帧数（sink buffer 累积）
#[test]
fn audio_render_frames_per_call() {
    run_on_big_stack(|| {
        use crate::midi::note::Note;
        let (engine, _ar) = setup();
        let cfg = test_config();
        let (tx, rx) = sync_channel(1024);
        let mut ar = crate::audio::AudioRender::new(
            12,
            4,
            22050.0,
            48000.0,
            cfg.midi.scoring.clone(),
            false,
            rx,
        );
        let part = engine.parts[0].clone();
        tx.send(crate::audio::AudioRenderActions::Play {
            note: Note::C4,
            vel: 100,
            part: part.clone(),
        })
        .unwrap();
        for _ in 0..1024 {
            ar.audio_render();
        }
        let buf = ar
            .sink
            .as_any_mut()
            .downcast_mut::<crate::audio::sink::VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let msg = format!(
            "1024 次 audio_render → sink {} 样本 = {} 帧（预期 1024 帧）",
            buf.len(),
            buf.len() / 2
        );
        std::fs::write("/tmp/render_frames.txt", &msg).unwrap();
        assert!(buf.len() / 2 == 1024, "{msg}");
    });
}

/// 渲染钢琴长音的每帧耗时（回归：效果链增益/pan 查表）
#[test]
fn piano_render_perf() {
    run_on_big_stack(|| {
        let (_engine, mut ar) = setup();
        let mut msg = String::new();
        for _round in 0..4 {
            let t0 = std::time::Instant::now();
            for _ in 0..1024 { ar.audio_render(); }
            msg += &format!("{}ms; ", t0.elapsed().as_millis());
        }
        std::fs::write("/tmp/piano_perf.txt", &msg).unwrap();
        assert!(std::time::Instant::now().elapsed().as_secs() < 60);
    });
}
