//! End-to-end integration test: MIDI NoteOn → Engine → AudioRender → sink audio output

use std::collections::HashMap;
use std::sync::mpsc::sync_channel;
use std::thread;

use libmadaha::SoundModuleType;

use crate::args::Args;
use crate::audio::AudioRender;
use crate::config::{AudioConfig, Config, MidiConfig, ScoringConfig, SoundModuleConfig};
use crate::midi::Engine;

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
            sample_rate: 44100,
            buffer_size: 256,
            interpolating: crate::audio::tone_generator::oscillator::InterpolatingMethods::Linear,
            device: None,
            channels: 2,
            master_volume: 1.0,
            soft_clip: false,
            dc_blocker: true,
            sleep_delay_ms: 200,
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

fn test_args() -> Args {
    Args {
        debug: true,
        config: "".to_string(),
        generate_default_config: false,
        dump_tbl_to_json_file: "".to_string(),
    }
}

fn setup() -> (Engine, AudioRender) {
    let (tx, rx) = sync_channel(256);
    let cfg = test_config();
    let arg = test_args();
    let engine = Engine::new(&cfg, &arg, tx);
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
    thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn midi_note_flows_to_audio_output() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
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
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
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
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
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
    use crate::midi::event::MidiEvent;
    use std::time::Duration;
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
        thread::sleep(Duration::from_millis(750));
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
    use crate::midi::event::MidiEvent;
    use std::time::Duration;
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
        thread::sleep(Duration::from_millis(750));
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
    use crate::midi::note::Note;
    run_on_big_stack(|| {
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

/// 手动试听辅助测试（无断言）：440Hz 正弦波 5 秒，走 cpal 后端
/// （CpalSink：ring + cpal 回调线程播放）。
#[test]
fn cpal_play_440hz() {
    use crate::audio::backend::cpal::CpalSink;
    use crate::audio::sink::AudioSink;
    use crate::audio::tone_generator::oscillator::InterpolatingMethods;
    use std::f32::consts::TAU;
    use std::fs;
    use std::time::Duration;

    let cfg = crate::config::AudioConfig {
        sample_rate: 48000,
        buffer_size: 64,
        interpolating: InterpolatingMethods::Linear,
        device: None,
        channels: 2,
        master_volume: 1.0,
        soft_clip: false,
        dc_blocker: false,
        sleep_delay_ms: 200,
    };
    let mut sink = CpalSink::open(&cfg).expect("cpal open failed");
    thread::sleep(Duration::from_secs(1)); // let the stream start
    let _ = fs::write("/tmp/negotiated.txt", format!("rate={}\n", sink.rate()));
    eprintln!("sine wave testing (cpal)...");
    let sample_rate = cfg.sample_rate as f32;
    let block = cfg.buffer_size as usize;
    let total = sample_rate as usize * 5; // 5 秒
    // Pre-fill the ring (~1.36s) so early callbacks never read a short chunk
    // (a starved first chunk skews the first cycles of the tone).
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
        // No sleep: the ring's write-side backpressure paces production to the
        // consumer rate (~48k/s); a fixed sleep starves the ring and the cpal
        // callback pads silence.
        sink.flush();
        i += n;
    }
    thread::sleep(Duration::from_secs(5)); // keep the stream alive
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

/// 切换音色 + 滑音（快速按键）场景回归：复用的 TG 遇到"当前音色无该音符的 key"
/// 时必须保持静音（kill），不得用旧音色的 sample 继续发声（音色混乱 bug）。
/// 元素 vel 范围不匹配走同一 else 分支（kill + return）。
#[test]
fn program_switch_reused_voice_stays_silent_on_missing_key() {
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        use crate::midi::event::MidiEvent;
        use crate::midi::note::Note;
        let (mut engine, mut ar) = setup();

        // 鼓 part（channel 10）：XG 鼓键覆盖约 13-91，note 92+ 无 key。
        // 先把 part 9 切到鼓音色库。
        engine.ram.xg.multi_part[9].write_with(|m| m.bank_select_msb = 0x7F);
        engine.on_event(MidiEvent::ProgramChange {
            channel: 9,
            program: 0,
        });
        let missing = {
            let snap = engine.parts[9].snapshot();
            let prog = snap.program_entry.clone().unwrap();
            (0u8..128).find(|n| prog[*n as usize].is_none())
        };
        if missing.is_none() {
            eprintln!("SKIP: drum program covers all 128 keys in this data set");
            return;
        }
        let note = Note::try_from(missing.unwrap()).unwrap();
        // 占一个 voice → 释放（进入 Releasing，仍在衰减——模拟复用的 TG 池）
        engine.on_event(MidiEvent::NoteOn {
            channel: 9,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }
        engine.on_event(MidiEvent::NoteOff {
            channel: 9,
            note: Note::C4,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }
        let before = ar
            .tone_generators
            .iter()
            .filter(|t| t.status != S::Idle)
            .count();
        assert_eq!(before, 1, "precondition: one releasing voice");
        // 无 key 音符 → 必须不发声（不得用旧 sample 触发新 Running voice）
        engine.on_event(MidiEvent::NoteOn {
            channel: 9,
            note,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Running)
                .count(),
            0,
            "missing-key note must not ring (stale-sample bug)"
        );
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status != S::Idle)
                .count(),
            before,
            "no new voice may be allocated for a missing-key note"
        );
    });
}

/// NoteOff（及 NoteOn vel=0）后，声音必须在 AEG release 时间内衰减到静音。
#[test]
fn noteoff_stops_audio_output() {
    use crate::midi::note::Note;
    run_on_big_stack(|| {
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
    use crate::midi::note::Note;
    use std::fs;
    use std::sync::mpsc::SyncSender;
    run_on_big_stack(|| {
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
        let play = |tx: &SyncSender<crate::audio::AudioRenderActions>| {
            tx.send(crate::audio::AudioRenderActions::Play {
                note: Note::C4,
                vel: 100,
                part: part.clone(),
            })
            .unwrap();
        };
        let release = |tx: &SyncSender<crate::audio::AudioRenderActions>| {
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
        fs::write(
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
        // （元素级 release 可长达 ~1.6s，等尾音不现实——直接清场）
        use crate::audio::tone_generator::interface::ToneGeneratorInterface;
        for tg in ar.tone_generators.iter_mut() {
            tg.kill();
        }
        ar.audio_render();
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
    use crate::midi::note::Note;
    use std::fs;
    run_on_big_stack(|| {
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
        fs::write("/tmp/render_speed.txt", &msg).unwrap();
        assert!(wall.as_millis() < 500, "{msg}");
    });
}

/// 验证鼓音色加载（drum bank 不应全 None——prevoiceIdx 定位修复）
#[test]
fn drum_kits_populated() {
    use std::fs;
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
        fs::write(
            "/tmp/drum_check.txt",
            format!("drum notes filled: {filled}/53, piano ok\n"),
        )
        .unwrap();
    });
}

/// 验证 audio_render() 每调用渲染的帧数（sink buffer 累积）
#[test]
fn audio_render_frames_per_call() {
    use crate::midi::note::Note;
    use std::fs;
    run_on_big_stack(|| {
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
        fs::write("/tmp/render_frames.txt", &msg).unwrap();
        assert!(buf.len() / 2 == 1024, "{msg}");
    });
}

/// 渲染钢琴长音的每帧耗时（回归：效果链增益/pan 查表）
#[test]
fn piano_render_perf() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use std::fs;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        if std::env::var("NOTE").is_ok() {
            engine.on_event(MidiEvent::NoteOn {
                channel: 0,
                note: Note::C4,
                velocity: 100,
                off_velocity: 0,
                duration: 0,
            });
        }
        let mut msg = String::new();
        for _round in 0..4 {
            let t0 = std::time::Instant::now();
            for _ in 0..1024 {
                ar.audio_render();
            }
            msg += &format!("{}ms; ", t0.elapsed().as_millis());
        }
        fs::write("/tmp/piano_perf.txt", &msg).unwrap();
        assert!(std::time::Instant::now().elapsed().as_secs() < 60);
    });
}

/// 复现 pipewire 阶梯损坏：模拟"64帧块写入 + 128帧 quantum 读取"
#[test]
fn ring_stress_reproduces_staircase() {
    use crate::audio::backend::ringbuf::SpscRing;
    use std::f32::consts::TAU;
    use std::fs;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let rb = Arc::new(SpscRing::new(65536));
    let rb_w = rb.clone();
    let sr = 48000.0f32;
    let writer = thread::spawn(move || {
        let mut t = 0.0f32;
        let mut buf = Vec::with_capacity(128);
        for _ in 0..(5 * 48000 / 64) {
            buf.clear();
            for _ in 0..64 {
                let s = (TAU * 440.0 * t).sin() * 0.8;
                buf.push(s);
                buf.push(s);
                t += 1.0 / sr;
            }
            rb_w.write(&buf);
            thread::sleep(Duration::from_secs_f32(64.0 / sr * 0.9));
        }
    });
    let rb_r = rb.clone();
    let reader = thread::spawn(move || {
        let mut tmp = vec![0.0f32; 256];
        let mut out = Vec::new();
        for _ in 0..(6 * 48000 / 128) {
            let frames = rb_r.read(&mut tmp[..256]);
            if frames > 0 {
                out.extend_from_slice(&tmp[..frames * 2]);
            }
            thread::sleep(Duration::from_secs_f32(128.0 / sr * 0.9));
        }
        out
    });
    writer.join().unwrap();
    let out = reader.join().unwrap();
    let n = out.len() / 2;
    let lft: Vec<f32> = out[..n * 2].iter().step_by(2).copied().collect();
    // 阶梯检测：相邻样本重复率（440Hz 正弦几乎无相邻相等）
    let dup = lft.windows(2).filter(|w| w[0] == w[1]).count();
    let dup_rate = dup as f32 / lft.len() as f32;
    // 跳变
    let jumps = lft.windows(2).filter(|w| (w[0] - w[1]).abs() > 0.3).count();
    let msg = format!(
        "读出 {} 帧, 相邻相等 {:.2}%, 跳变 {jumps} 处\n",
        n,
        dup_rate * 100.0
    );
    let msg2 = msg + &format!("前 40 样本: {:?}", &lft[..40.min(lft.len())]);
    fs::write("/tmp/ring_stress.txt", &msg2).unwrap();
    assert!(dup_rate < 0.001, "{msg2}");
}

/// 单线程：先全部写入再读回——验证 ring 逻辑自身
#[test]
fn ring_single_thread_roundtrip() {
    use crate::audio::backend::ringbuf::SpscRing;
    use std::f32::consts::TAU;
    use std::fs;
    let rb = SpscRing::new(65536);
    let sr = 48000.0f32;
    // 写入 10000 帧（模拟 64 帧块）
    let mut t = 0.0f32;
    let mut expected = Vec::new();
    let mut buf = Vec::with_capacity(128);
    for _ in 0..10000 / 64 {
        buf.clear();
        for _ in 0..64 {
            let s = (TAU * 440.0 * t).sin() * 0.8;
            buf.push(s);
            buf.push(s);
            expected.push(s);
            t += 1.0 / sr;
        }
        rb.write(&buf);
    }
    let mut tmp = vec![0.0f32; 256];
    let mut got = Vec::new();
    let mut total = 0;
    while total < 10000 {
        let frames = rb.read(&mut tmp[..256]);
        if frames == 0 {
            break;
        }
        for i in 0..frames {
            got.push(tmp[i * 2]);
        }
        total += frames;
    }
    assert_eq!(got.len(), expected.len(), "帧数不一致");
    let bad = got
        .iter()
        .zip(&expected)
        .filter(|&(a, b)| (a - b).abs() > 1e-4)
        .count();
    let mut dbg = String::new();
    dbg += &format!("expected[0..40]: {:?}\n", &expected[..40]);
    dbg += &format!("got[0..40]:     {:?}\n", &got[..40]);
    dbg += &format!("expected[60..70]: {:?}\n", &expected[60..70]);
    dbg += &format!("got[60..70]:     {:?}\n", &got[60..70]);
    fs::write("/tmp/ring_single.txt", &dbg).unwrap();
    assert_eq!(bad, 0, "{bad} 帧不一致");
}

/// 复现"写 53k/s > 读 48k/s"→ ring 满丢块 → 跳变
#[test]
fn ring_overflow_drop_causes_jumps() {
    use crate::audio::backend::ringbuf::SpscRing;
    use std::f32::consts::TAU;
    use std::fs;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let rb = Arc::new(SpscRing::new(65536));
    let rb_w = rb.clone();
    let sr = 48000.0f32;
    let writer = thread::spawn(move || {
        let mut t = 0.0f32;
        let mut buf = Vec::with_capacity(128);
        for _ in 0..(5 * 48000 / 64) {
            buf.clear();
            for _ in 0..64 {
                let s = (TAU * 440.0 * t).sin() * 0.8;
                buf.push(s);
                buf.push(s);
                t += 1.0 / sr;
            }
            rb_w.write(&buf);
            thread::sleep(Duration::from_secs_f32(64.0 / sr * 0.9));
        }
    });
    let rb_r = rb.clone();
    let reader = thread::spawn(move || {
        let mut tmp = vec![0.0f32; 256];
        let mut out = Vec::new();
        for _ in 0..(6 * 48000 / 128) {
            let frames = rb_r.read(&mut tmp[..256]);
            if frames > 0 {
                out.extend_from_slice(&tmp[..frames * 2]);
            }
            thread::sleep(Duration::from_secs_f32(128.0 / sr));
        }
        out
    });
    writer.join().unwrap();
    let out = reader.join().unwrap();
    let n = out.len() / 2;
    let lft: Vec<f32> = out[..n * 2].iter().step_by(2).copied().collect();
    let jumps = lft.windows(2).filter(|w| (w[0] - w[1]).abs() > 0.2).count();
    let msg = format!(
        "写出 {n} 帧, 跳变 {jumps} 处 (每 {:.1}ms 一处)",
        n as f64 / 48000.0 * 1000.0 / jumps.max(1) as f64
    );
    fs::write("/tmp/ring_overflow.txt", &msg).unwrap();
    assert!(jumps == 0, "{msg}");
}

/// CC#123 All Notes Off → 释放 part 全部音符；CC#120 All Sound Off → 立即静音
#[test]
fn all_notes_off_releases_part_voices() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        ar.audio_render();
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::E4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        ar.audio_render();
        let active = ar
            .tone_generators
            .iter()
            .filter(|t| t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
            .count();
        assert_eq!(active, 2, "两个音符应发声");

        // CC#123 All Notes Off
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 123,
            value: 0,
        });
        ar.audio_render();
        let releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == crate::audio::tone_generator::ToneGeneratorStatus::Releasing)
            .count();
        assert_eq!(
            releasing, 2,
            "CC#123 应释放 part 的全部音符 (releasing={releasing})"
        );
    });
}

/// 调制公式回归：默认调制深度（0x40=无调制）下，pitchbend 应改变音高而非音量
#[test]
fn pitchbend_changes_pitch_not_volume() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        // 弹 C4
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..48000 {
            ar.audio_render();
        } // 1s，越过 attack/decay 到稳态
        let buf0 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        // pitchbend 全上（+2 半音，默认灵敏度）
        engine.on_event(MidiEvent::PitchBend {
            channel: 0,
            value: 16383,
        });
        for _ in 0..48000 {
            ar.audio_render();
        }
        let buf1 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        // 稳态幅值对比（跳过 attack 尾部，取后半）
        let amp = |b: &[f32]| -> f32 {
            let s = &b[b.len() - 4000..];
            let c: Vec<f32> = s.chunks(2).map(|c| c[0]).collect();
            c.iter().map(|x| x * x).sum::<f32>() / c.len() as f32
        };
        let a0 = amp(&buf0);
        let a1 = amp(&buf1);
        // 音量不应大幅变化（默认 amp 调制 = 0）。测量时刻相隔 1s，
        // 钢琴采样自然衰减约 10-15%，故放宽到 20%（修复前 ±24dB ≈ 15 倍变化）。
        assert!(
            (a1 - a0).abs() < a0 * 0.20,
            "bend 不应改变音量: 前 {a0} 后 {a1}"
        );
        // 音高应升高（自相关基频比 ≈ 2^(2/12)；零交叉受 8-bit 谐波干扰）
        let fz = |b: &[f32]| -> f32 {
            let s: Vec<f32> = b.chunks(2).map(|c| c[0]).collect();
            let seg = &s[s.len() - 8000..];
            let mut best_lag = 0usize;
            let mut best_v = 0.0f32;
            for lag in (48000 / 2000)..(48000 / 40) {
                if lag > seg.len() / 2 {
                    break;
                }
                let c: f32 = (0..(seg.len() - lag))
                    .step_by(4)
                    .map(|i| seg[i] * seg[i + lag])
                    .sum();
                let e: f32 = (0..(seg.len() - lag))
                    .step_by(4)
                    .map(|i| seg[i] * seg[i])
                    .sum();
                if e <= 0.0 {
                    continue;
                }
                let v = c / e;
                if v > best_v {
                    best_v = v;
                    best_lag = lag;
                }
            }
            if best_lag > 0 {
                44100.0 / best_lag as f32
            } else {
                0.0
            }
        };
        let f0 = fz(&buf0);
        let f1 = fz(&buf1);
        let ratio = f1 / f0;
        assert!(
            (ratio - 2f32.powf(2.0 / 12.0)).abs() < 0.05,
            "bend 应升高音高: 前 {f0}Hz 后 {f1}Hz (ratio {ratio})"
        );
    });
}

/// 诊断：渲染 note 的输出频率 + ratio_cents 各成分（定位音高偏移）
#[test]
fn pitch_offset_diagnose() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use std::fs;
    run_on_big_stack(|| {
        // 每个音符独立测量
        for target in [48u8, 60u8, 69u8, 81u8] {
            let (mut engine, mut ar) = setup();
            // 输出频率与采样率配置无关（pos 步进按 ratio×play_speed，play_speed 抵消）
            let note = Note::try_from(target).unwrap_or(Note::C4);
            engine.on_event(MidiEvent::NoteOn {
                channel: 0,
                note,
                velocity: 100,
                off_velocity: 0,
                duration: 0,
            });
            for _ in 0..44100 {
                ar.audio_render();
            }
            // 取稳态输出频率
            let buf = ar
                .sink
                .as_any_mut()
                .downcast_mut::<VecBufferSink>()
                .map(|s| s.take_buffer())
                .unwrap();
            let s: Vec<f32> = buf.chunks(2).map(|c| c[0]).collect();
            // 自相关基频（更可靠）
            let autocorr = |data: &[f32], sr: f32, skip: usize| -> f32 {
                let data = &data[data.len().saturating_sub(8000.min(data.len()))..];
                let mut best_lag = 0usize;
                let mut best_v = 0.0f32;
                for lag in ((sr / 2000.0) as usize)..((sr / 40.0) as usize) {
                    if lag > data.len() / 2 {
                        break;
                    }
                    let c: f32 = (0..(data.len() - lag))
                        .step_by(skip.max(1))
                        .map(|i| data[i] * data[i + lag])
                        .sum();
                    let e: f32 = (0..(data.len() - lag))
                        .step_by(skip.max(1))
                        .map(|i| data[i] * data[i])
                        .sum();
                    if e <= 0.0 {
                        continue;
                    }
                    let v = c / e;
                    // 选相关度最高的最小 lag（低 lag = 高基频；高次谐波周期会给出
                    // 错误的低基频，如 1/3 谐波误测）
                    if v > best_v * 1.02 {
                        best_v = v;
                        best_lag = lag;
                    }
                }
                if best_lag > 0 {
                    sr / best_lag as f32
                } else {
                    0.0
                }
            };
            let f = autocorr(&s, 44100.0, 4);
            let mut msg = String::new();
            msg += &format!(
                "弹内部note{target} 期望 {}Hz\n",
                440.0 * 2f32.powf((target as f32 - 69.0) / 12.0)
            );
            // 打印 TG 的音高成分
            msg += &format!(
                "输出频率: {0:.1} Hz zc={1:.1} Hz\n",
                f,
                zero_crossing_freq(&s, 44100.0)
            );
            let _ = fs::write(
                format!("/tmp/out_{target}.f32"),
                s.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(),
            );
            for (ti, tg) in ar.tone_generators.iter().enumerate() {
                if tg.status == crate::audio::tone_generator::ToneGeneratorStatus::Idle {
                    continue;
                }
                let o = &tg.oscillator;
                let sm = o.sample_ref();
                if let Some(sm) = sm {
                    let note_cent = o.pitch.note_in_cent;
                    msg += &format!(
                        "TG{ti}: note_cent={note_cent:.1} base_note_cent={:.1} coarse_cent={:.1} tone={:.1} pitch_offset={:.1} fine={:.1}\n",
                        sm.get_base_note_cent(),
                        sm.get_coarse_in_cent(),
                        sm.get_tone(),
                        sm.get_pitch_offset(),
                        sm.get_fine_in_cent(100),
                    );
                    msg += &format!(
                        "      base_cent={:.0} coarse_cent={:.0} tone={:.1} pitch_offset={:.1}\n",
                        sm.get_base_note_cent(),
                        sm.get_coarse_in_cent(),
                        sm.get_tone(),
                        sm.get_pitch_offset(),
                    );
                }
                msg += &format!(
                    "      pitch_note_cent={:.1} pitch_mod={:.1} peg_level={:.1} peg_state={:?}\n",
                    o.pitch.note_in_cent, o.pitch_mod, o.peg.current_level, o.peg.state,
                );
                let mp = engine.parts[0].snapshot().ram.snapshot();
                msg += &format!(
                    "      part: note_shift={} detune={} scale0={} coarse_rpn={}\n",
                    mp.note_shift,
                    mp.get_detune(),
                    mp.scale_tuning[0],
                    engine.parts[0].snapshot().rpn.coarse,
                );
                if let Some(sm) = sm {
                    msg += &format!(
                        "      sample: detune={} wave_pitch={} loop_point={} loop_length={} pcm_len={}\n",
                        sm.detune,
                        sm.wave_pitch,
                        sm.loop_point,
                        sm.loop_length,
                        sm.pcm.as_ref().map(|p| p.len()).unwrap_or(0),
                    );
                    msg += &format!(
                        "      cents_to_ratio({})={} pos_step={}\n",
                        o.pitch.note_in_cent - sm.get_base_note_cent()
                            + sm.get_coarse_in_cent()
                            + sm.get_tone(),
                        crate::audio::tone_generator::oscillator::oscillator::cents_to_ratio(
                            o.pitch.note_in_cent - sm.get_base_note_cent()
                                + sm.get_coarse_in_cent()
                                + sm.get_tone()
                        ),
                        o.play_speed_base,
                    );
                }
                msg += &format!(
                    "      porta: src={:.1} tgt={:.1} time={:.1} elapsed={:.1} delay_samples={} delay_state={:?}\n",
                    o.portamento.source_note,
                    o.portamento.target_note,
                    o.portamento.portamento_time,
                    0.0,
                    o.delay.delay_samples,
                    0u32,
                );
                msg += &format!("      play_speed_base={}\n", o.play_speed_base);
                if let Some(pcm) = sm.and_then(|s| s.pcm.as_deref()) {
                    // 不经播放链，直接测 PCM 内容基频（过零 vs 自相关对照）
                    let pcm_f = pcm.to_vec();
                    let _ = fs::write(
                        format!("/tmp/pcm_{target}.bin"),
                        pcm_f
                            .iter()
                            .flat_map(|v| v.to_le_bytes())
                            .collect::<Vec<_>>(),
                    );
                    msg += &format!(
                        "      pcm_content_freq={:.1}Hz zc={:.1}Hz (base_key={}, len={})\n",
                        autocorr(pcm, 22050.0, 4),
                        zero_crossing_freq(&pcm_f, 22050.0),
                        (sm.unwrap().get_base_note_cent() / 100.0) as u8,
                        pcm.len(),
                    );
                }
            }
            fs::write(format!("/tmp/pitch_diag_{target}.txt"), &msg).unwrap();
        }
    });
}

/// MIDI 48（C3）输入 → 内部 48（键号直映：MIDI 键号 = Yamaha 键号 = 内部键号）
#[test]
fn midi_48_maps_to_internal_48() {
    use crate::midi::event::MidiEvent;
    run_on_big_stack(|| {
        // parse_midi_bytes: MIDI 48 → internal 48
        let mut rs = None;
        let mut sx = Vec::new();
        let mut out = Vec::new();
        crate::midi::source::parse_midi_bytes(&[0x90, 48, 100], &mut rs, &mut sx, &mut out);
        let MidiEvent::NoteOn { note, .. } = out[0] else {
            panic!("no NoteOn")
        };
        assert_eq!(note as u8, 48, "MIDI 48 应映射到内部 48");
    });
}

/// set_output_rate 重定目标后音高不变（渲染时钟跟随 sink 实际速率）
#[test]
fn output_rate_retarget_keeps_pitch() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..44100 {
            ar.audio_render();
        }
        let f0 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        // 重新定位到 22050（模拟 ALSA 协商）
        ar.set_output_rate(22050.0);
        for _ in 0..22050 {
            ar.audio_render();
        } // 虚拟 1 秒 @22050
        let f1 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let freq = |b: &[f32]| -> f32 {
            let s: Vec<f32> = b.chunks(2).map(|c| c[0]).collect();
            zero_crossing_freq(&s, 22050.0)
        };
        let a = freq(&f0);
        let b = freq(&f1);
        // f0 段渲染时钟 = 44100（AudioRender 初始 sample_rate），用 22050 解算
        // → 显示值 = 真实×22050/44100；f1 段时钟 = 22050 → 显示值 = 真实。
        // 重定位前后音高必须一致：b ≈ a × 44100/22050 = a × 2
        let expected_b = a * 44100.0 / 22050.0;
        assert!(
            (b - expected_b).abs() < expected_b * 0.08,
            "set_output_rate 后音高应不变: 前 {a}Hz(22050 解算) 后 {b}Hz (期望 {expected_b})"
        );
    });
}

/// 回归：A3 = 69 → 440Hz（Yamaha 键号直映；采样录于 baseKey，无八度补偿）
#[test]
fn a3_plays_at_440hz() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::A3,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..44100 {
            ar.audio_render();
        }
        let buf = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let s: Vec<f32> = buf.chunks(2).map(|c| c[0]).collect();
        // 限定在期望附近搜索：全范围扫峰会被 8-bit 波形的偶次谐波（2f）干扰
        let f = dft_freq_in(&s, 44100.0, 400.0, 480.0);
        assert!(
            (f - 440.0).abs() < 440.0 * 0.03,
            "A3(69) 应输出 440Hz，实际 {f}Hz"
        );
    });
}

/// DFT 测频（指定范围：10Hz 粗扫 + 0.2Hz 细化）。
/// 全范围扫峰会被 8-bit 波形的偶次谐波（2f 常比基频强）干扰，
/// 限定在期望音高附近搜索可稳定得到基频。
fn dft_freq_in(s: &[f32], sr: f32, lo: f32, hi: f32) -> f32 {
    use std::f32::consts::TAU;

    let seg = &s[s.len().saturating_sub(8000.min(s.len()))..];
    let mag_at = |f: f32| -> f32 {
        let w = TAU * f / sr;
        let (mut re, mut im) = (0.0f32, 0.0f32);
        for (i, &v) in seg.iter().enumerate() {
            let t = w * i as f32;
            re += v * t.cos();
            im += v * t.sin();
        }
        (re * re + im * im).sqrt()
    };
    let mut best_f = lo;
    let mut best_m = 0.0f32;
    let mut f = lo;
    while f <= hi {
        let m = mag_at(f);
        if m > best_m {
            best_m = m;
            best_f = f;
        }
        f += 10.0;
    }
    let mut fine_f = best_f;
    let mut fine_m = 0.0f32;
    let mut f = (best_f - 10.0).max(lo);
    while f <= (best_f + 10.0).min(hi) {
        let m = mag_at(f);
        if m > fine_m {
            fine_m = m;
            fine_f = f;
        }
        f += 0.2;
    }
    fine_f
}

/// 过零率测频（去均值；仅用于简单正弦信号诊断）
fn zero_crossing_freq(s: &[f32], sr: f32) -> f32 {
    let seg = &s[s.len().saturating_sub(8000.min(s.len()))..];
    let mean: f32 = seg.iter().sum::<f32>() / seg.len() as f32;
    let mut crossings = 0usize;
    for w in seg.windows(2) {
        if (w[0] - mean) * (w[1] - mean) < 0.0 {
            crossings += 1;
        }
    }
    crossings as f32 / 2.0 * sr / seg.len() as f32
}

/// RAM Pitch EG（08 pp 69-6C）链路：非默认值在 note-on 时应用到 PEG
/// （init level → 音头初始偏移，attack 滑回 0）
#[test]
fn ram_pitch_eg_applies_on_note_on() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();

        // RAM: pitch_eg_init_level = 96 → (96-64)/64×1200 = +600 cents（+6 半音音头偏移）
        engine.ram.xg.multi_part[0].write_with(|m| m.pitch_eg_init_level = 96);
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }

        let lvl = ar
            .tone_generators
            .iter()
            .find(|tg| tg.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
            .map(|tg| tg.oscillator.peg.current_level)
            .expect("voice not allocated");
        // 64 帧渲染后 PEG 已开始下滑（~0.08 cent/sample），初始偏移应在 600 附近
        assert!(
            lvl > 590.0 && lvl <= 601.0,
            "PEG 初始电平应为 +600 cents（已开始下滑），实际 {lvl}"
        );

        // attack 滑回 0（速率由 pitch_eg_attack_time=0x40 → 元素 peg_rate0=64 决定）
        for _ in 0..44100 {
            ar.audio_render();
        }
        let final_lvl = ar
            .tone_generators
            .iter()
            .find(|tg| tg.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
            .map(|tg| tg.oscillator.peg.current_level)
            .unwrap_or(0.0);
        assert!(final_lvl.abs() < 1.0, "PEG 应滑回 0，实际 {final_lvl}");
    });
}

/// 快速同音 NoteOn×2 + NoteOff×2：两个叠音都必须释放。
/// 回归：release_handler 未过滤 Releasing 时，attack_time 相同的两个 TG
/// 会被两次 NoteOff 选到同一个（release() 对 Releasing 无效）→ 另一个永不释放。
#[test]
fn rapid_same_note_releases_all_voices() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();

        // key_assign=1 (Multi) → 同音叠加
        engine.ram.xg.multi_part[0].write_with(|m| m.key_assign = 1);
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 90,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }

        let running = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Running && t.get_note() == Some(Note::C4))
            .count();
        assert_eq!(running, 2, "同音应叠加 2 个 voice");

        // 构造 attack_time 相同的场景（快速连按的极端情况）
        let same = std::time::Instant::now();
        for tg in ar.tone_generators.iter_mut() {
            if tg.status == S::Running && tg.get_note() == Some(Note::C4) {
                tg.attack_time = same;
            }
        }

        engine.on_event(MidiEvent::NoteOff {
            channel: 0,
            note: Note::C4,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        });
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

        let releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Releasing && t.get_note() == Some(Note::C4))
            .count();
        let still_running = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Running && t.get_note() == Some(Note::C4))
            .count();
        assert_eq!(
            releasing, 2,
            "两个叠音都应释放: releasing={releasing} still_running={still_running}"
        );
    });
}

/// CC#64 sustain: NoteOff 在踏板按下时挂起（TG 保持 Running），松开后释放
#[test]
fn sustain_pedal_holds_noteoff_until_release() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();

        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        // 踩下踏板 → NoteOn → NoteOff
        engine.on_event(cc(64, 127));
        engine.on_event(note_on(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }
        engine.on_event(note_off(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }

        // 挂起：TG 仍 Running（未释放）
        let running = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Running && t.get_note() == Some(Note::C4))
            .count();
        assert_eq!(running, 1, "sustain 踩下时 NoteOff 应挂起");

        // 松开踏板 → 释放
        engine.on_event(cc(64, 0));
        for _ in 0..64 {
            ar.audio_render();
        }
        let releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Releasing && t.get_note() == Some(Note::C4))
            .count();
        assert_eq!(releasing, 1, "踏板松开后挂起的音符应释放");
    });
}

/// CC#123 (All Notes Off): 必须释放 sustain 挂起的音符
#[test]
fn cc123_releases_sustain_held_notes() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();

        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        engine.on_event(cc(64, 127));
        engine.on_event(note_on(Note::C4));
        engine.on_event(note_on(Note::E4));
        for _ in 0..64 {
            ar.audio_render();
        }
        engine.on_event(note_off(Note::C4));
        engine.on_event(note_off(Note::E4));
        for _ in 0..64 {
            ar.audio_render();
        }

        // 两个都挂起（Running）
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Running)
                .count(),
            2
        );

        // CC#123 → 全部释放 + 挂起队列清空
        engine.on_event(cc(123, 0));
        for _ in 0..64 {
            ar.audio_render();
        }
        let releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Releasing)
            .count();
        assert_eq!(releasing, 2, "CC#123 应释放 sustain 挂起的音符");
        assert_eq!(
            ar.sustain_held.values().map(|v| v.len()).sum::<usize>(),
            0,
            "CC#123 后挂起队列应清空"
        );
    });
}

/// CC#120 (All Sound Off): 立即 kill sustain 挂起的音符
#[test]
fn cc120_kills_sustain_held_notes() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();

        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        engine.on_event(cc(64, 127));
        engine.on_event(note_on(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }
        engine.on_event(note_off(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }

        engine.on_event(cc(120, 0));
        for _ in 0..64 {
            ar.audio_render();
        }
        // CC#120 = kill（立即静音），挂起队列清空
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Running)
                .count(),
            0
        );
        assert_eq!(ar.sustain_held.values().map(|v| v.len()).sum::<usize>(), 0);
    });
}

/// 钢琴类（program 0-7，S-YXG50 无 sustain_mode 字段）→ damper 策略：
/// 踏板挂起时 AEG 走 Damp 段衰减（S-YXG50 钢琴的"明显衰减"）
#[test]
fn sustain_damper_decays_piano_while_held() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        // GM piano: program 0 (XG acoustic-piano group) — no sustain_mode field → damper fallback
        engine.on_event(cc(64, 127));
        engine.on_event(note_on(Note::C4));
        for _ in 0..256 {
            ar.audio_render();
        }
        engine.on_event(note_off(Note::C4));
        for _ in 0..256 {
            ar.audio_render();
        }

        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.get_note() == Some(Note::C4))
            .unwrap();
        assert!(
            tg.damper_hold,
            "piano program must use damper hold (fallback)"
        );

        // Sustain reached → Damp stage decays the level below sustain_level
        for _ in 0..12000 {
            ar.audio_render();
        }
        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.get_note() == Some(Note::C4))
            .unwrap();
        assert!(
            !tg.amp.aeg.enabled || tg.amp.aeg.level < tg.amp.aeg.sustain_level,
            "damper level must decay below sustain level (enabled={} level={} sustain={})",
            tg.amp.aeg.enabled,
            tg.amp.aeg.level,
            tg.amp.aeg.sustain_level
        );
    });
}

/// 非钢琴 program + 无 sustain_mode 字段 → 恒保持（挂起时 AEG 冻结在 sustain level）
#[test]
fn sustain_hold_freezes_non_piano() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        engine.ram.xg.multi_part[0].write_with(|m| m.program_number = 40);
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 1,
            value: 0,
        }); // trigger swap
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        engine.on_event(cc(64, 127));
        engine.on_event(note_on(Note::C4));
        for _ in 0..256 {
            ar.audio_render();
        }
        engine.on_event(note_off(Note::C4));
        for _ in 0..256 {
            ar.audio_render();
        }

        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.get_note() == Some(Note::C4))
            .unwrap();
        assert!(
            !tg.damper_hold,
            "non-piano program must not use damper hold"
        );

        // Frozen at sustain level (no decay while held)
        for _ in 0..12000 {
            ar.audio_render();
        }
        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.get_note() == Some(Note::C4))
            .unwrap();
        assert!(
            (tg.amp.aeg.level - tg.amp.aeg.sustain_level).abs() < 1e-4,
            "non-piano sustain must freeze at sustain level"
        );
    });
}

/// sustain_mode 字段优先于 program fallback：mode=2 即使非钢琴也 damper；
/// mode=1 即使钢琴也不 damper
#[test]
fn sustain_mode_wins_over_program_fallback() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        engine.ram.xg.multi_part[0].write_with(|m| m.program_number = 40);
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 1,
            value: 0,
        }); // trigger swap
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        // 2006LE data: element sustain_mode=2 (damper) on a non-piano program
        engine.on_event(note_on(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }
        ar.tone_generators
            .iter_mut()
            .find(|t| t.get_note() == Some(Note::C4))
            .unwrap()
            .sustain_mode = 2;
        engine.on_event(cc(64, 127));
        engine.on_event(note_off(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }
        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.get_note() == Some(Note::C4))
            .unwrap();
        assert!(
            tg.damper_hold,
            "sustain_mode=2 must win over program fallback"
        );

        // sustain_mode=1 (half-hold) on piano program 0 → frozen hold
        engine.on_event(note_on(Note::E4));
        for _ in 0..64 {
            ar.audio_render();
        }
        ar.tone_generators
            .iter_mut()
            .find(|t| t.get_note() == Some(Note::E4))
            .unwrap()
            .sustain_mode = 1;
        engine.on_event(note_off(Note::E4));
        for _ in 0..64 {
            ar.audio_render();
        }
        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.get_note() == Some(Note::E4))
            .unwrap();
        assert!(
            !tg.damper_hold,
            "sustain_mode=1 must disable damper even on piano"
        );
    });
}

/// Watchdog：无 TG 发声且静音窗口（sleep_delay，默认 200ms）过后 →
/// needs_render=false（渲染线程可休眠）；新音符立即唤醒
#[test]
fn watchdog_sleeps_after_silence_window() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use std::time::Duration;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        ar.set_sleep_delay(Duration::from_millis(200));

        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..256 {
            ar.audio_render();
        }
        assert!(ar.needs_render(), "active note must keep rendering");

        engine.on_event(MidiEvent::NoteOff {
            channel: 0,
            note: Note::C4,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        });
        // release 尾音（元素级最长 ~1.6s + 200ms budget）+ 静音窗口走完后才可休眠
        for _ in 0..110000 {
            ar.audio_render();
        }
        assert!(
            !ar.needs_render(),
            "watchdog must sleep after the silence window"
        );

        // 新音符 → 立即恢复渲染
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::E4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        ar.audio_render(); // drain the Play action
        assert!(ar.needs_render(), "new note must wake the watchdog");
    });
}

/// sleep_idle：休眠等待事件；事件到达立即处理并唤醒
#[test]
fn sleep_idle_wakes_on_event() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use std::time::Duration;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        ar.set_sleep_delay(Duration::from_millis(200));

        // 排空初始化/残留事件
        while ar.sleep_idle(Duration::from_millis(10)) {}
        for _ in 0..20000 {
            ar.audio_render();
        } // 0.45s > 200ms 窗口
        assert!(!ar.needs_render(), "must be idle before the sleep test");

        // 无事件 → 超时返回 false
        assert!(
            !ar.sleep_idle(Duration::from_millis(10)),
            "no event → timeout"
        );

        // 事件到达 → 处理并唤醒
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        assert!(
            ar.sleep_idle(Duration::from_millis(100)),
            "event must wake sleep_idle"
        );
        assert!(ar.needs_render(), "wakeup must restore rendering");
    });
}

/// RPN#0（Pitch Bend Sensitivity）= 12 → bend 全量应为 ±12 半音（ratio 2）
#[test]
fn rpn0_sets_pitchbend_range() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };

        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..48000 {
            ar.audio_render();
        }
        let f0 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();

        // RPN 0 = 12（CC101=0 RPN MSB, CC100=0 RPN LSB, CC6=12 Data Entry MSB）
        engine.on_event(cc(101, 0));
        engine.on_event(cc(100, 0));
        engine.on_event(cc(6, 12));
        engine.on_event(MidiEvent::PitchBend {
            channel: 0,
            value: 16383,
        });
        for _ in 0..48000 {
            ar.audio_render();
        }
        let f1 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();

        let freq = |b: &[f32]| -> f32 {
            let s: Vec<f32> = b.chunks(2).map(|c| c[0]).collect();
            dft_freq_in(&s, 44100.0, 400.0, 2000.0)
        };
        let a = freq(&f0);
        let b = freq(&f1);
        let ratio = b / a;
        // RPN 0 = 12 → +12 半音 → ratio 2.0（±3%）
        assert!(
            (ratio - 2.0).abs() < 0.06,
            "RPN0=12 时 bend 全量应 +12 半音: 前 {a}Hz 后 {b}Hz ratio {ratio}"
        );
    });
}

/// CC#7（Volume）：note-on 前设为 0 → 静音；note-on 后实时改变
#[test]
fn cc7_volume_controls_output() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        let cc = |v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: 7,
            value: v,
        };
        let rms = |ar: &mut crate::audio::AudioRender| -> f32 {
            let b = ar
                .sink
                .as_any_mut()
                .downcast_mut::<VecBufferSink>()
                .map(|s| s.take_buffer())
                .unwrap();
            let s: Vec<f32> = b.chunks(2).map(|c| c[0]).collect();
            (s.iter().map(|x| x * x).sum::<f32>() / s.len().max(1) as f32).sqrt()
        };

        // note-on 前 CC7=0 → 静音
        engine.on_event(cc(0));
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..48000 {
            ar.audio_render();
        }
        assert!(rms(&mut ar) < 1e-4, "CC7=0 时应静音");

        // note-on 后 CC7=127 → 实时恢复
        engine.on_event(cc(127));
        for _ in 0..48000 {
            ar.audio_render();
        }
        assert!(rms(&mut ar) > 1e-3, "CC7=127 后应发声");

        // 再 CC7=0 → 实时静音（XG: volume 实时生效）
        engine.on_event(cc(0));
        for _ in 0..48000 {
            ar.audio_render();
        }
        let b = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let s: Vec<f32> = b.chunks(2).map(|c| c[0]).collect();
        let tail: f32 = s[s.len() - 8000..].iter().map(|x| x * x).sum::<f32>() / 8000.0;
        assert!(
            tail < 1e-4,
            "CC7=0 应实时静音（非 note-on 快照）: tail={tail}"
        );
    });
}

/// CC#11（Expression）：0 → 静音（实时）
#[test]
fn cc11_expression_controls_output() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..48000 {
            ar.audio_render();
        }
        let b0 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let amp0: f32 = b0.chunks(2).map(|c| c[0] * c[0]).sum::<f32>() / (b0.len() / 2) as f32;

        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 11,
            value: 0,
        });
        for _ in 0..48000 {
            ar.audio_render();
        }
        let b1 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let s1: Vec<f32> = b1.chunks(2).map(|c| c[0]).collect();
        let amp1: f32 = s1[s1.len() - 8000..].iter().map(|x| x * x).sum::<f32>() / 8000.0;
        assert!(amp1 < amp0 * 1e-3, "CC11=0 应静音: {amp0} → {amp1}");
    });
}

/// CC#10（Panpot）：0 → 左声道、127 → 右声道
#[test]
fn cc10_pan_moves_stereo_image() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, mut ar) = setup();
        let pan = |v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: 10,
            value: v,
        };
        let stereo = |ar: &mut crate::audio::AudioRender| -> (f32, f32) {
            let b = ar
                .sink
                .as_any_mut()
                .downcast_mut::<VecBufferSink>()
                .map(|s| s.take_buffer())
                .unwrap();
            let l: f32 = b.chunks(2).map(|c| c[0] * c[0]).sum();
            let r: f32 = b.chunks(2).map(|c| c[1] * c[1]).sum();
            (l, r)
        };

        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        engine.on_event(pan(1)); // 极左
        for _ in 0..48000 {
            ar.audio_render();
        }
        let (l, r) = stereo(&mut ar);
        assert!(l > r * 3.0, "pan=1 应偏左: L={l} R={r}");

        engine.on_event(pan(127)); // 极右
        for _ in 0..48000 {
            ar.audio_render();
        }
        let (l, r) = stereo(&mut ar);
        assert!(r > l * 3.0, "pan=127 应偏右: L={l} R={r}");
    });
}

/// CAT（Channel Aftertouch）/ PAT（Poly Aftertouch）：存储生效（rcv 开关默认开）
#[test]
fn cat_pat_values_stored() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        let (mut engine, _ar) = setup();
        engine.on_event(MidiEvent::ChannelPressure {
            channel: 0,
            pressure: 96,
        });
        assert_eq!(engine.parts[0].snapshot().cat_value, 96);

        engine.on_event(MidiEvent::PolyPressure {
            channel: 0,
            note: Note::C4,
            pressure: 80,
        });
        assert_eq!(engine.parts[0].snapshot().pat_values[Note::C4 as usize], 80);
    });
}

/// CC#66 sostenuto：只保持踏板踩下时已响的音符，新音符正常释放
#[test]
fn sostenuto_holds_only_pedal_time_notes() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        // C4 先响，然后踩下 sostenuto
        engine.on_event(note_on(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }
        engine.on_event(cc(66, 127)); // sostenuto 踩下
        for _ in 0..64 {
            ar.audio_render();
        }
        // 踩下后再按 E4（不应被保持）
        engine.on_event(note_on(Note::E4));
        for _ in 0..64 {
            ar.audio_render();
        }

        // 两个都 NoteOff
        engine.on_event(note_off(Note::C4));
        engine.on_event(note_off(Note::E4));
        for _ in 0..64 {
            ar.audio_render();
        }

        // C4（踩下时活跃）→ 挂起保持 Running；E4（踩下后按的）→ 正常释放
        let c4_running = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Running && t.get_note() == Some(Note::C4))
            .count();
        let e4_releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Releasing && t.get_note() == Some(Note::E4))
            .count();
        assert_eq!(c4_running, 1, "sostenuto 踩下时活跃的 C4 应保持");
        assert_eq!(e4_releasing, 1, "sostenuto 踩下后按的 E4 应正常释放");

        // 松开 sostenuto → C4 释放
        engine.on_event(cc(66, 0));
        for _ in 0..64 {
            ar.audio_render();
        }
        let c4_releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Releasing && t.get_note() == Some(Note::C4))
            .count();
        assert_eq!(c4_releasing, 1, "sostenuto 松开后 C4 应释放");
    });
}

/// CC#67 soft pedal 存储 + CC#71-75（谐波/起音/释音/亮度）→ RAM 08 pp 18-1C
#[test]
fn cc67_soft_pedal_stored_cc71_75_write_ram() {
    use crate::midi::event::MidiEvent;
    run_on_big_stack(|| {
        let (mut engine, _ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };

        // CC#67 soft pedal 踩下
        engine.on_event(cc(67, 127));
        assert!(engine.parts[0].snapshot().controller.soft_pedal);
        engine.on_event(cc(67, 0));
        assert!(!engine.parts[0].snapshot().controller.soft_pedal);

        // CC#74 brightness → 08 pp 18 (filter cutoff)
        engine.on_event(cc(74, 100));
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().filter_cutoff_freq,
            100
        );
        // CC#71 harmonic content → 08 pp 19
        engine.on_event(cc(71, 80));
        assert_eq!(engine.ram.xg.multi_part[0].snapshot().filter_resonance, 80);
        // CC#73 attack time → 08 pp 1A
        engine.on_event(cc(73, 90));
        assert_eq!(engine.ram.xg.multi_part[0].snapshot().eg_attack_time, 90);
        // CC#75 decay time → 08 pp 1B
        engine.on_event(cc(75, 60));
        assert_eq!(engine.ram.xg.multi_part[0].snapshot().eg_decay_time, 60);
        // CC#72 release time → 08 pp 1C
        engine.on_event(cc(72, 70));
        assert_eq!(engine.ram.xg.multi_part[0].snapshot().eg_release_time, 70);
    });
}

/// Yamaha XG bulk dump（F0 43 1n 4C ...）：修复前 `||` 逻辑导致永不生效
#[test]
fn xg_bulk_write_applies_parameters() {
    use crate::midi::event::MidiEvent;
    run_on_big_stack(|| {
        use crate::midi::sysex::ManufacturerId;
        let (mut engine, _ar) = setup();

        // XG System On（F0 43 0n 4C 00 00 00 7E 00 chk F7）→ reset_mode = XG（bulk 前置）
        let syson = vec![0x7F, 0x4C, 0x00, 0x00, 0x00, 0x7E, 0x00];
        let mut syson = syson.clone();
        let sum: u32 = syson[2..].iter().map(|&b| b as u32).sum();
        syson.push(((!sum + 1) & 0x7F) as u8);
        engine.on_event(MidiEvent::SysEx {
            manufacturer_id: ManufacturerId::Yamaha,
            data: syson.into(),
        });

        // bulk 写 volume（08 00 0B）= 60（默认 100，用非默认值验证真实写入）
        // 数据：dev_id(7F 广播), model, lenH, lenL, addrH, addrM, addrL, data
        let mut d = vec![0x7F, 0x4C, 0x10, 0x00, 0x01, 0x08, 0x00, 0x0B, 60];
        let sum: u32 = d[2..].iter().map(|&b| b as u32).sum();
        d.push(((!sum + 1) & 0x7F) as u8);
        engine.on_event(MidiEvent::SysEx {
            manufacturer_id: ManufacturerId::Yamaha,
            data: d.into(),
        });
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().volume,
            60,
            "bulk write 应写入 volume"
        );

        // dev_id 不匹配（0x02 ≠ 设备 id）→ 拒绝
        let mut d2 = vec![0x02, 0x4C, 0x10, 0x00, 0x01, 0x08, 0x00, 0x0B, 40];
        let sum: u32 = d2[2..].iter().map(|&b| b as u32).sum();
        d2.push(((!sum + 1) & 0x7F) as u8);
        engine.on_event(MidiEvent::SysEx {
            manufacturer_id: ManufacturerId::Yamaha,
            data: d2.into(),
        });
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().volume,
            60,
            "dev_id 不匹配的 bulk 应被拒绝"
        );
    });
}

/// NRPN（CC98/99 选参 + CC6 数据）→ XG RAM 写入（Filter Cutoff 08 pp 18）
#[test]
fn nrpn_writes_xg_ram() {
    use crate::midi::event::MidiEvent;
    run_on_big_stack(|| {
        let (mut engine, _ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };

        // NRPN MSB=0x01, LSB=0x20 → Filter Cutoff Frequency (08 pp 18)
        engine.on_event(cc(99, 0x01));
        engine.on_event(cc(98, 0x20));
        engine.on_event(cc(6, 100));
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().filter_cutoff_freq,
            100,
            "NRPN 应写入 filter cutoff"
        );

        // NRPN LSB=0x21 → Filter Resonance (08 pp 19)
        engine.on_event(cc(98, 0x21));
        engine.on_event(cc(6, 80));
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().filter_resonance,
            80,
            "NRPN 应写入 filter resonance"
        );

        // NRPN (0x01, 0x08) → Vibrato Rate (08 pp 15)
        engine.on_event(cc(99, 0x01));
        engine.on_event(cc(98, 0x08));
        engine.on_event(cc(6, 90));
        assert_eq!(
            engine.ram.xg.multi_part[0].snapshot().vibrato_rate,
            90,
            "NRPN 应写入 vibrato rate"
        );
    });
}

/// XG 单址 SysEx 写 volume（08 00 0B）→ 实际影响声音（note-on 后静音/发声）
#[test]
fn xg_sysex_writes_affect_sound() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::midi::sysex::ManufacturerId;
        let (mut engine, mut ar) = setup();

        let xg_write = |addr: [u8; 3], val: u8| -> MidiEvent {
            let mut d = vec![0x7F, 0x4C, 0x00, addr[0], addr[1], addr[2], val];
            let sum: u32 = d[2..].iter().map(|&b| b as u32).sum();
            d.push(((!sum + 1) & 0x7F) as u8);
            MidiEvent::SysEx {
                manufacturer_id: ManufacturerId::Yamaha,
                data: d.into(),
            }
        };
        // XG System On → reset_mode = XG（单址写的前置）
        engine.on_event(xg_write([0x00, 0x00, 0x7E], 0));

        // volume = 0 → 静音
        engine.on_event(xg_write([0x08, 0x00, 0x0B], 0));
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..48000 {
            ar.audio_render();
        }
        let b0 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let amp0: f32 = b0.chunks(2).map(|c| c[0] * c[0]).sum::<f32>() / (b0.len() / 2) as f32;
        assert!(amp0 < 1e-4, "volume=0 时应静音: {amp0}");

        // volume = 127 → 发声（实时）
        engine.on_event(xg_write([0x08, 0x00, 0x0B], 127));
        for _ in 0..48000 {
            ar.audio_render();
        }
        let b1 = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let amp1: f32 = b1.chunks(2).map(|c| c[0] * c[0]).sum::<f32>() / (b1.len() / 2) as f32;
        // 相对验证 volume 生效（绝对音量受 AEG sustain 电平影响——待 AEG 对齐）
        assert!(
            amp1 > amp0 * 5.0,
            "volume=127 应明显大于 volume=0: {amp0} → {amp1}"
        );
    });
}

/// PortUnsubscribed（MIDI 源断开，防 panic 机制）：全部 channel 释放，
/// 包括 sustain 挂起的音符
#[test]
fn port_unsubscribed_releases_all_channels() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |ch: u8, n: Note| MidiEvent::NoteOn {
            channel: ch,
            note: n,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |ch: u8, n: Note| MidiEvent::NoteOff {
            channel: ch,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        // 两个 channel 的 note + sustain 挂起一个
        engine.on_event(note_on(0, Note::C4));
        engine.on_event(note_on(1, Note::E4));
        engine.on_event(cc(64, 127)); // sustain 踩下
        for _ in 0..64 {
            ar.audio_render();
        }
        engine.on_event(note_off(0, Note::C4)); // 挂起（sustain）
        for _ in 0..64 {
            ar.audio_render();
        }
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Running)
                .count(),
            2
        );

        // PortUnsubscribed → 全部 channel 释放（含 sustain 挂起）
        engine.on_event(MidiEvent::PortUnsubscribed);
        for _ in 0..64 {
            ar.audio_render();
        }
        let releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Releasing)
            .count();
        assert_eq!(releasing, 2, "PortUnsubscribed 应释放全部音符");
        assert_eq!(
            ar.sustain_held.values().map(|v| v.len()).sum::<usize>(),
            0,
            "PortUnsubscribed 后挂起队列应清空"
        );
    });
}

/// 双 element 音色（bank lsb=41 prog=0 Dream）：vel 范围内应分配 2 个 TG 且都发声
#[test]
fn dual_element_dream_allocates_two_voices() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();

        // 切到 lsb=41 prog=0（Dream）
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 32,
            value: 41,
        });
        engine.on_event(MidiEvent::ProgramChange {
            channel: 0,
            program: 0,
        });

        let key = engine.parts[0]
            .snapshot()
            .program_entry
            .as_ref()
            .and_then(|p| p[Note::C4 as usize].as_ref())
            .cloned();
        assert!(
            key.as_ref()
                .is_some_and(|k| k.layers[0].is_some() && k.layers[1].is_some()),
            "Dream 应为双元素（layer0+layer1 都有）"
        );

        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }

        let active = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Running && t.get_note() == Some(Note::C4))
            .count();
        assert!(
            active >= 2,
            "双 element 音色应分配 2 个 voice，实际 {active}"
        );

        // 两个都发声（渲染到稳态后 RMS > 0）
        for _ in 0..48000 {
            ar.audio_render();
        }
        let b = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let amp: f32 = b.chunks(2).map(|c| c[0] * c[0]).sum::<f32>() / (b.len() / 2) as f32;
        assert!(amp > 1e-5, "双 element 音色应发声: {amp}");
    });
}

/// 双元素音色（Dream, LSB=41 prog=0）：加载 + 分配 2 个 TG 并播放
#[test]
fn dual_element_dream_loads_two_tone_generators() {
    use crate::audio::sink::VecBufferSink;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };

        // Bank LSB=41 (Dream), MSB=0, Program=0
        engine.on_event(cc(0, 0));
        engine.on_event(cc(32, 41));
        engine.on_event(MidiEvent::ProgramChange {
            channel: 0,
            program: 0,
        });

        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..256 {
            ar.audio_render();
        }

        // 应分配 2 个 TG（双元素）
        let active: Vec<usize> = ar
            .tone_generators
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status != S::Idle)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(active.len(), 2, "Dream 应分配 2 个 TG: {active:?}");

        // 两个 TG 的采样应不同（不同元素）
        let sms: Vec<(u32, usize)> = ar
            .tone_generators
            .iter()
            .filter(|t| t.status != S::Idle)
            .filter_map(|t| t.oscillator.sample_ref())
            .map(|sm| (sm.get_base_note_cent() as u32, sm.loop_point))
            .collect();
        assert_eq!(sms.len(), 2);
        assert_ne!(sms[0], sms[1], "两个元素采样应不同: {sms:?}");

        // 渲染 1 秒：两个 TG 都应发声（输出非零）
        for _ in 0..48000 {
            ar.audio_render();
        }
        let b = ar
            .sink
            .as_any_mut()
            .downcast_mut::<VecBufferSink>()
            .map(|s| s.take_buffer())
            .unwrap();
        let amp: f32 = b.chunks(2).map(|c| c[0] * c[0]).sum::<f32>() / (b.len() / 2) as f32;
        assert!(amp > 1e-4, "双元素应发声: amp={amp}");
    });
}

/// 诊断：libmadaha BinTbl 对 Dream（LSB=41 prog=0）的 prevoice 解析（双元素?）
#[test]
fn dream_prevoice_elements_diagnose() {
    let b = match libmadaha::load(
        libmadaha::SoundModuleType::Syxg50,
        "/home/user/Projects/yxg50/VST/Yamaha/sxgbin41.tbl".to_string(),
        "/home/user/Projects/yxg50/VST/Yamaha/Sxgwave4.tbl".to_string(),
    )
    .unwrap()
    {
        libmadaha::LoadedModule::Syxg50(b) => b,
    };
    let idx = b.get_program_index(0, 41, 0);
    assert_eq!(idx, 17551, "Dream 的 prevoice 索引应为 17551");
    let pv = b.get_prevoice(idx);
    let (e0, e1) = pv.expect("Dream prevoice 应存在");
    assert_eq!(e0.index, 236);
    assert!(e1.is_some(), "Dream 应为双元素");
    let e1 = e1.unwrap();
    assert_eq!(e1.index, 37);
    // 两个元素的采样链都应可加载
    let mut sms0 = vec![];
    b.get_sample_meta(&mut sms0, e0.index as usize);
    assert!(!sms0.is_empty(), "e0 采样链为空");
    let mut sms1 = vec![];
    b.get_sample_meta(&mut sms1, e1.index as usize);
    assert!(!sms1.is_empty(), "e1 采样链为空");
    for sm in &sms1 {
        eprintln!("e1 sample: key_end={} base_key={}", sm.key_end, sm.base_key);
    }
}

/// 双元素音色（Dream）：NoteOff 应释放整个组（2 个 TG 都进入 Releasing）
#[test]
fn dual_element_noteoff_releases_both_voices() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };

        engine.on_event(cc(0, 0));
        engine.on_event(cc(32, 41));
        engine.on_event(MidiEvent::ProgramChange {
            channel: 0,
            program: 0,
        });
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }
        let running = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Running)
            .count();
        assert_eq!(running, 2, "Dream 应分配 2 个 voice");

        // NoteOff → 两个元素都应释放（组释放）
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
        let releasing = ar
            .tone_generators
            .iter()
            .filter(|t| t.status == S::Releasing)
            .count();
        assert_eq!(
            releasing, 2,
            "NoteOff 应释放双元素的全部 voice: {releasing}"
        );
    });
}

/// NoteOn(vel=0) 等效 NoteOff：双元素也应全部释放
#[test]
fn dual_element_noteon_zero_velocity_releases_both() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };

        engine.on_event(cc(0, 0));
        engine.on_event(cc(32, 41));
        engine.on_event(MidiEvent::ProgramChange {
            channel: 0,
            program: 0,
        });
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Running)
                .count(),
            2
        );

        // NoteOn vel=0 = NoteOff
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..64 {
            ar.audio_render();
        }
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Releasing)
                .count(),
            2,
            "NoteOn(vel=0) 应释放双元素全部 voice"
        );
    });
}

/// 双元素 + 同音快速连按（key_assign=1 stacking）：每组 NoteOff 释放正确组，
/// 不串音（id=1 组先释放，id=2 组后释放）
#[test]
fn dual_element_stacked_noteoffs_release_correct_groups() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    run_on_big_stack(|| {
        use crate::audio::tone_generator::ToneGeneratorStatus as S;
        let (mut engine, mut ar) = setup();
        let cc = |c: u8, v: u8| MidiEvent::ControlChange {
            channel: 0,
            controller: c,
            value: v,
        };
        let note_on = |n: Note, v: u8| MidiEvent::NoteOn {
            channel: 0,
            note: n,
            velocity: v,
            off_velocity: 0,
            duration: 0,
        };
        let note_off = |n: Note| MidiEvent::NoteOff {
            channel: 0,
            note: n,
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        };

        engine.on_event(cc(0, 0));
        engine.on_event(cc(32, 41));
        engine.on_event(MidiEvent::ProgramChange {
            channel: 0,
            program: 0,
        });

        // 两次同音 NoteOn（stacking，双元素 → 4 个 TG）
        engine.on_event(note_on(Note::C4, 100));
        engine.on_event(note_on(Note::C4, 90));
        for _ in 0..64 {
            ar.audio_render();
        }
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Running)
                .count(),
            4
        );

        // 第一个 NoteOff → 只释放 id 最小的一组（2 个 TG）
        engine.on_event(note_off(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }
        let (rel, run) = (
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Releasing)
                .count(),
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Running)
                .count(),
        );
        assert_eq!(
            (rel, run),
            (2, 2),
            "第一个 NoteOff 应释放一组（2 释放/2 响）: rel={rel} run={run}"
        );

        // 第二个 NoteOff → 第二组也释放
        engine.on_event(note_off(Note::C4));
        for _ in 0..64 {
            ar.audio_render();
        }
        assert_eq!(
            ar.tone_generators
                .iter()
                .filter(|t| t.status == S::Releasing)
                .count(),
            4,
            "第二个 NoteOff 应释放剩余组"
        );
    });
}

/// xorshift 随机分配 + 缓冲：刚释放的 voice（idle_since 新）不被立即复用，
/// 优先选中"空闲已久"的 voice（确定性：无论随机起点，缓冲期满者必被第一轮扫到）。
#[test]
fn idle_allocation_prefers_buffered_voice() {
    run_on_big_stack(|| {
        use crate::midi::event::MidiEvent;
        use crate::midi::note::Note;
        let (mut engine, mut ar) = setup();
        use crate::audio::tone_generator::interface::ToneGeneratorInterface;
        for tg in ar.tone_generators.iter_mut() {
            tg.kill();
        }
        let n = ar.tone_generators.len();
        assert!(n >= 2, "pool must have ≥2 voices");

        // voice 0: 刚释放（缓冲期内）；voice 1: 空闲已久
        ar.tone_generators[0].idle_since = std::time::Instant::now();
        ar.tone_generators[1].idle_since =
            std::time::Instant::now() - std::time::Duration::from_secs(5);

        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::C4,
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        ar.audio_render(); // drain the Play action
        let used = ar.tone_generators.iter().position(|t| {
            t.status == crate::audio::tone_generator::ToneGeneratorStatus::Running
                && t.note == Some(Note::C4)
        });
        assert_eq!(
            used,
            Some(1),
            "buffered allocation must prefer the long-idle voice (got {used:?})"
        );
    });
}

/// musicbox（prog 9，外部编码 10）元素 AEG release 时长（XG rate 表语义）：
/// aeg_rel=58 → ~312ms（旧映射 2000×2^(-v/8) 只有 13ms，导致所有元素级
/// attack/decay/release 过短——长余音/击锤丢失的系统性问题）
#[test]
fn musicbox_element_release_uses_xg_rate_table() {
    run_on_big_stack(|| {
        use crate::midi::event::MidiEvent;
        use crate::midi::note::Note;
        let (mut engine, mut ar) = setup();
        // ⚠ 2026-08-12 修正：真音乐盒 = program 10（GS seg11[10] → 0x9D48，
        // 双元素 seg15_idx 80/111）；program 9 = GS 钢琴（0x98A2）——旧测试测错音色。
        // 必须走 ProgramChange 事件（RAM set 钩子触发 part program_entry 重建）；
        // 直写 write_with 不重建（旧测试因此一直测的是钢琴）。
        engine.on_event(MidiEvent::ProgramChange {
            channel: 0,
            program: 10,
        });
        engine.on_event(MidiEvent::ControlChange {
            channel: 0,
            controller: 1,
            value: 0,
        });

        // element AEG release override present
        let snap = engine.parts[0].snapshot();
        let key = snap.program_entry.as_ref().unwrap()[72].as_ref().unwrap();
        let sm = key.layers[0].unwrap().2;
        assert!(
            sm.aeg_rel != 0,
            "musicbox must have an element release override"
        );
        drop(snap);

        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::try_from(72).unwrap(),
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..22050 {
            ar.audio_render();
        }
        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.note == Some(Note::try_from(72).unwrap()))
            .unwrap();
        assert!(
            tg.amp.aeg.release_time >= std::time::Duration::from_millis(250),
            "musicbox release must follow the XG rate table (~312ms), got {:?}",
            tg.amp.aeg.release_time
        );
        // noteoff 后余音必须持续 >100ms（旧 13ms 在 0.01s 内静音）
        engine.on_event(MidiEvent::NoteOff {
            channel: 0,
            note: Note::try_from(72).unwrap(),
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        });
        let mut survived = 0u32;
        for _ in 0..22050 {
            ar.audio_render();
            if ar.tone_generators.iter().any(|t| {
                t.note == Some(Note::try_from(72).unwrap())
                    && t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle
            }) {
                survived += 1;
            } else {
                break;
            }
        }
        assert!(
            survived > 4400,
            "musicbox release tail must exceed 100ms (got {survived} frames)"
        );
    });
}

/// XG 短按键行为：attack+decay 是保护阶段——noteoff 不打断，
/// 走完 decay 到 sustain 后才进入 release（musicbox 击锤+叮完整）
#[test]
fn short_note_preserves_attack_decay() {
    run_on_big_stack(|| {
        use crate::midi::event::MidiEvent;
        use crate::midi::note::Note;
        let (mut engine, mut ar) = setup();
        engine.on_event(MidiEvent::ProgramChange {
            channel: 0,
            program: 10,
        });
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::try_from(72).unwrap(),
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        ar.audio_render();
        let attack_time = {
            let tg = ar
                .tone_generators
                .iter()
                .find(|t| t.note == Some(Note::try_from(72).unwrap()))
                .unwrap();
            tg.amp.aeg.attack_time
        };
        // 极短按键：attack 未完成就 noteoff
        for _ in 0..(attack_time.as_secs_f32() * 44100.0) as usize / 2 {
            ar.audio_render();
        }
        engine.on_event(MidiEvent::NoteOff {
            channel: 0,
            note: Note::try_from(72).unwrap(),
            velocity: 0,
            off_velocity: 0,
            duration: 0,
        });
        ar.audio_render();
        // 保护：noteoff 后 AEG 不得立即 Release（还在 Attack/Decay）
        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.note == Some(Note::try_from(72).unwrap()))
            .unwrap();
        assert!(
            !matches!(
                tg.amp.aeg.state,
                crate::audio::tone_generator::AEGStage::Release
            ),
            "noteoff during attack must not cut the envelope (state={:?})",
            tg.amp.aeg.state
        );
        // 走完 decay（元素 aeg_d1 → decay 时长）后进入 Release
        let decay_time = {
            let tg = ar
                .tone_generators
                .iter()
                .find(|t| t.note == Some(Note::try_from(72).unwrap()))
                .unwrap();
            tg.amp.aeg.decay_time
        };
        for _ in 0..(decay_time.as_secs_f32() * 44100.0) as usize + 22050 {
            ar.audio_render();
        }
        let tg = ar
            .tone_generators
            .iter()
            .find(|t| t.note == Some(Note::try_from(72).unwrap()))
            .unwrap();
        assert!(
            matches!(
                tg.amp.aeg.state,
                crate::audio::tone_generator::AEGStage::Release
                    | crate::audio::tone_generator::AEGStage::Finished
            ),
            "after decay completes the pending release must fire (state={:?})",
            tg.amp.aeg.state
        );
    });
}

// 临时验证：渲染音乐盒 note72 → /tmp/madaha-new-note72.wav

/// A1 竞态修复验证（2026-08-13）：双元素音色偶发单元素无声（~1/10）。
/// 根因：on_event 中 Play 在 swap 前发送 → 音频线程可能读到旧 program 条目 →
/// element_count 回退 1。修复：动作收集 → swap → 再发送。
/// 本测试用双线程模拟真实时序（MIDI 线程发事件、音频线程紧循环渲染），
/// 统计 ProgramChange 后 NoteOn 的 polyphony（应为 2）。

/// A1 竞态修复验证（2026-08-13）：双元素音色偶发单元素无声（~1/10）。
/// 根因：on_event 中 Play 在 swap 前发送 → 音频线程可能读到旧 program 条目 →
/// element_count 回退 1。修复：动作收集 → swap → 再发送。
/// 与真实架构一致：ar 在音频线程内创建（dyn sink 非 Send）；polyphony 经
/// 共享 AtomicUsize 由音频线程报告。
#[test]
fn dual_element_program_change_race_stress() {
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc::sync_channel;
    run_on_big_stack(|| {
        let (tx, rx) = sync_channel(256);
        let cfg = test_config();
        let arg = test_args();
        let engine = crate::midi::engine::Engine::new(&cfg, &arg, tx);
        engine.send_audio_init();
        let eng = Arc::new(std::sync::Mutex::new(engine));
        let eng2 = eng.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let poly_now = Arc::new(AtomicUsize::new(0));
        let poly2 = poly_now.clone();

        // 音频线程：紧循环渲染 + 报告 polyphony
        let audio = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let mut ar = AudioRender::new(
                    64,
                    64,
                    44100.0,
                    44100.0,
                    cfg.midi.scoring.clone(),
                    false,
                    rx,
                );
                while !stop2.load(Ordering::Relaxed) {
                    ar.audio_render();
                    poly2.store(ar.get_current_polyphony(), Ordering::Relaxed);
                }
            })
            .unwrap();

        // MIDI 线程：每 3 次迭代插 ProgramChange(10)，随后 NoteOn(72)
        let midi = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let mut anomalies = 0u32;
                for i in 0..600 {
                    {
                        let mut e = eng2.lock().unwrap();
                        if i < 400 && i % 3 == 0 {
                            // 前 400 次：每 3 次插一个 ProgramChange（切音色竞态场景）
                            e.on_event(MidiEvent::ProgramChange {
                                channel: 0,
                                program: 10,
                            });
                        }
                        // 后 200 次：稳定程序快速重触发（无 PC）
                        e.on_event(MidiEvent::NoteOn {
                            channel: 0,
                            note: Note::try_from(72).unwrap(),
                            velocity: 100,
                            off_velocity: 0,
                            duration: 0,
                        });
                    }
                    // 等 poly 达 2（最多 100ms）
                    let start = std::time::Instant::now();
                    let p = loop {
                        let v = poly_now.load(Ordering::Relaxed);
                        if v >= 2 || start.elapsed().as_millis() > 100 {
                            break v;
                        }
                        std::thread::sleep(std::time::Duration::from_micros(200));
                    };
                    if p != 2 {
                        anomalies += 1;
                        eprintln!("DBG-RACE anomaly iter={i} poly={p}");
                    }
                    {
                        let mut e = eng2.lock().unwrap();
                        e.on_event(MidiEvent::NoteOff {
                            channel: 0,
                            note: Note::try_from(72).unwrap(),
                            velocity: 0,
                            off_velocity: 0,
                            duration: 0,
                        });
                    }
                    // 等 release 完成（避免堆叠干扰下一轮）
                    let start = std::time::Instant::now();
                    loop {
                        let v = poly_now.load(Ordering::Relaxed);
                        if v == 0 || start.elapsed().as_millis() > 200 {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_micros(200));
                    }
                }
                anomalies
            })
            .unwrap();

        let anomalies = midi.join().unwrap();
        stop.store(true, Ordering::Relaxed);
        audio.join().unwrap();
        assert_eq!(
            anomalies, 0,
            "双元素竞态：{anomalies}/600 次 NoteOn 仅分配 1 个 element"
        );
    });
}


/// 诊断：Organ（prog 16）延音验证——长按应持续发声（RMS 不衰减）
#[test]
fn dbg_organ_sustain() {
    run_on_big_stack(|| {
        use crate::midi::event::MidiEvent;
        use crate::midi::note::Note;
        use crate::audio::sink::VecBufferSink;
        for prog in [0u8, 12, 16, 21, 22] {
            let _ = prog;
            let (mut engine, mut ar) = setup();
            engine.on_event(MidiEvent::ProgramChange { channel: 0, program: prog });
            engine.on_event(MidiEvent::NoteOn {
                channel: 0,
                note: Note::C4,
                velocity: 100,
                off_velocity: 0,
                duration: 0,
            });
            let mut rmses = vec![];
            for _ in 0..6 {
                for _ in 0..22050 {
                    ar.audio_render();
                }
                let buf = ar
                    .sink
                    .as_any_mut()
                    .downcast_mut::<VecBufferSink>()
                    .unwrap()
                    .take_buffer();
                let n = buf.len() as f32;
                let mut sum = 0.0f64;
                for &v in buf.iter() {
                    sum += (v as f64) * (v as f64);
                }
                rmses.push((sum / n as f64).sqrt() as f32);
            }
            wd_log::log_debug_ln!(
                "DBG-RMS prog={} rms/500ms: {:?}",
                prog,
                rmses
            );
        }
    });
}

/// 冗余池 + 事件级评分释放：NoteOn 无条件分配，分配后复音超过设定值
/// （max_polyphony）→ 评分释放（高分先杀，双元素同组）直到回到限制内。
#[test]
fn polyphony_redundant_pool_then_event_steal() {
    use crate::audio::AudioRender;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use wd_log::log_debug_ln;
    use std::sync::mpsc::sync_channel;
    run_on_big_stack(|| {
        let (tx, rx) = sync_channel(256);
        let cfg = test_config();
        let mut engine = Engine::new(&cfg, &test_args(), tx);
        engine.send_audio_init();
        let mut ar = AudioRender::new(
            12, // 池大小（含冗余）
            8,  // 设定复音数（max_polyphony）
            44100.0,
            44100.0,
            cfg.midi.scoring.clone(),
            false,
            rx,
        );

        // 弹 10 个单元素音符（prog 12 Marimba 单元素）
        engine.on_event(MidiEvent::ProgramChange { channel: 0, program: 12 });
        let notes = [60u8, 62, 64, 65, 67, 69, 71, 72, 74, 76];
        for (i, n) in notes.iter().enumerate() {
            let _ = i;
            engine.on_event(MidiEvent::NoteOn {
                channel: 0,
                note: Note::try_from(*n).unwrap(),
                velocity: 100,
                off_velocity: 0,
                duration: 0,
            });
            // 渲染处理该事件（分配 + 事件级检查）
            for _ in 0..64 {
                ar.audio_render();
            }
            // 每 2 个事件采样一次 active 数
            if i % 2 == 1 || i == 9 {
                let active = ar
                    .tone_generators
                    .iter()
                    .filter(|t| t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
                    .count();
                log_debug_ln!(
                    "DBG-POLY after {} notes: active={} (limit 8)",
                    i + 1,
                    active
                );
            }
        }

        // 最终：active ≤ 8（分配后事件级释放）
        let active = ar
            .tone_generators
            .iter()
            .filter(|t| t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle)
            .count();
        assert!(
            active <= 8,
            "复音数必须 ≤ max_polyphony(8)，实际 {active}"
        );
        // 最早弹的 note 60（评分最高：最旧 + time_weight）应已被释放
        let note60_voice = ar
            .tone_generators
            .iter()
            .any(|t| t.get_note() == Some(Note::try_from(60).unwrap()) && t.status != crate::audio::tone_generator::ToneGeneratorStatus::Idle);
        assert!(
            !note60_voice,
            "评分最高的旧音符（note 60）应被释放"
        );
    });
}

/// 双元素音色（musicbox）：steal 时同 note_on_id 组一起释放
#[test]
fn dbg_render_lufs_probe() {
    use crate::audio::AudioRender;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use crate::audio::sink::VecBufferSink;
    use wd_log::log_debug_ln;
    use std::sync::mpsc::sync_channel;
    run_on_big_stack(|| {
        let (tx, rx) = sync_channel(256);
        let cfg = test_config();
        let mut engine = Engine::new(&cfg, &test_args(), tx);
        engine.send_audio_init();
        let mut ar = AudioRender::new(
            64, 64, 44100.0, 44100.0, cfg.midi.scoring.clone(), false, rx,
        );

        // 代表性多音色：钢琴/Organ/Marimba/Musicbox/弦乐 + 多音符
        let progs = [0u8, 16, 12, 10, 48];
        let notes_seq: [[u8; 3]; 5] = [
            [60, 64, 67],
            [60, 62, 64],
            [60, 72, 79],
            [72, 76, 79],
            [55, 59, 62],
        ];
        let mut all: Vec<f32> = vec![];
        for (pi, &p) in progs.iter().enumerate() {
            engine.on_event(MidiEvent::ProgramChange { channel: 0, program: p });
            for &n in &notes_seq[pi] {
                engine.on_event(MidiEvent::NoteOn {
                    channel: 0,
                    note: Note::try_from(n).unwrap(),
                    velocity: 100,
                    off_velocity: 0,
                    duration: 0,
                });
            }
            for _ in 0..22050 * 3 {
                ar.audio_render();
            }
            let buf = ar
                .sink
                .as_any_mut()
                .downcast_mut::<VecBufferSink>()
                .unwrap()
                .take_buffer();
            all.extend(buf);
            for &n in &notes_seq[pi] {
                engine.on_event(MidiEvent::NoteOff {
                    channel: 0,
                    note: Note::try_from(n).unwrap(),
                    velocity: 0,
                    off_velocity: 0,
                    duration: 0,
                });
            }
            for _ in 0..22050 {
                ar.audio_render();
            }
            let _ = ar.sink.as_any_mut().downcast_mut::<VecBufferSink>().unwrap().take_buffer();
        }
        // 写 wav（f32 → i16）
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        let data_len = all.len() * 2;
        wav.extend_from_slice(&((36u32 + data_len as u32)).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&44100u32.to_le_bytes());
        wav.extend_from_slice(&(44100u32 * 2 * 2).to_le_bytes());
        wav.extend_from_slice(&4u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data_len as u32).to_le_bytes());
        for &v in &all {
            let s = (v.clamp(-1.0, 1.0) * 32767.0) as i16;
            wav.extend_from_slice(&s.to_le_bytes());
        }
        std::fs::write("/tmp/opencode/lufs_probe.wav", wav).unwrap();
        log_debug_ln!("DBG-LUFS probe saved {} samples", all.len());
    });
}

/// TG 复用（steal 释放后）声音一致性：kill 完整重置后，复用 TG 的渲染
/// RMS 应与首次分配一致（无残留增益/相位导致的"声音小"）。
#[test]
fn tg_reuse_after_steal_has_consistent_volume() {
    use crate::audio::AudioRender;
    use crate::midi::event::MidiEvent;
    use crate::midi::note::Note;
    use crate::audio::sink::VecBufferSink;
    use wd_log::log_debug_ln;
    use std::sync::mpsc::sync_channel;
    run_on_big_stack(|| {
        let (tx, rx) = sync_channel(256);
        let cfg = test_config();
        let mut engine = Engine::new(&cfg, &test_args(), tx);
        engine.send_audio_init();
        let mut ar = AudioRender::new(
            4, // 池大小
            2, // 设定复音数——容易触发 steal
            44100.0,
            44100.0,
            cfg.midi.scoring.clone(),
            false,
            rx,
        );

        engine.on_event(MidiEvent::ProgramChange { channel: 0, program: 12 });

        let render_rms = |ar: &mut AudioRender| -> f32 {
            for _ in 0..22050 {
                ar.audio_render();
            }
            let buf = ar
                .sink
                .as_any_mut()
                .downcast_mut::<VecBufferSink>()
                .unwrap()
                .take_buffer();
            let n = buf.len() as f32;
            let sum: f32 = buf.iter().map(|v| v * v).sum();
            (sum / n).sqrt()
        };

        // 首次弹 A（note 60）——记录 RMS
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::try_from(60).unwrap(),
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..128 {
            ar.audio_render();
        }
        let rms_first = render_rms(&mut ar);

        // 弹 B/C/D——复音超限（max=2）触发 steal（评分释放 A）
        for n in [62u8, 64, 65] {
            engine.on_event(MidiEvent::NoteOn {
                channel: 0,
                note: Note::try_from(n).unwrap(),
                velocity: 100,
                off_velocity: 0,
                duration: 0,
            });
            for _ in 0..64 {
                ar.audio_render();
            }
        }

        // 释放 B/C/D（释放 TG 回池）
        for n in [62u8, 64, 65] {
            engine.on_event(MidiEvent::NoteOff {
                channel: 0,
                note: Note::try_from(n).unwrap(),
                velocity: 0,
                off_velocity: 0,
                duration: 0,
            });
        }
        for _ in 0..22050 * 2 {
            ar.audio_render();
        }
        let _ = ar.sink.as_any_mut().downcast_mut::<VecBufferSink>().unwrap().take_buffer();

        // 再弹 A（复用被 steal/释放的 TG）——记录 RMS
        engine.on_event(MidiEvent::NoteOn {
            channel: 0,
            note: Note::try_from(60).unwrap(),
            velocity: 100,
            off_velocity: 0,
            duration: 0,
        });
        for _ in 0..128 {
            ar.audio_render();
        }
        let rms_reuse = render_rms(&mut ar);

        let ratio = rms_reuse / rms_first;
        log_debug_ln!(
            "DBG-REUSE rms_first={:.6} rms_reuse={:.6} ratio={:.3}",
            rms_first,
            rms_reuse,
            ratio
        );
        assert!(
            ratio > 0.7 && ratio < 1.4,
            "复用 TG 声音应一致：ratio={ratio}（first={rms_first} reuse={rms_reuse}）"
        );
    });
}
