use wd_log::log_debug_ln;

use serde::{Deserialize, Serialize};

use crate::audio::interface::Audio;
use std::sync::{Arc, LazyLock};
use std::time::{self, Duration, Instant};

use crate::config::ScoringConfig;
use crate::double_buffer::DoubleBuffered;
use crate::lfo::LFO;
use crate::lfo::lfo::LFORunningMode;
use crate::lfo::wave_type::WaveType;
use crate::midi::Part;
use crate::midi::PitchGetter;
use crate::midi::effect_params::parameter_table::{
    XG_LEVEL, XG_LFO_FREQ_TABLE, XG_MODULATION_DELAY_OFFSET_TABLE,
};
use crate::midi::note::Note;
use crate::midi::ram::xg::drum_setup_wrapper::DrumSetupWrapper;
use libmadaha::yxg50::piecewise_curve;
use libmadaha::yxg50::pre_voice::{key_follow, key_on_delay_index};

use super::amp::Amp;
use super::eq::EQ;
use super::hpf::HPF;
use super::interface::ToneGeneratorInterface;
use super::lpf::{CutOff, FEG, LPF};
use super::oscillator::Oscillator;
use super::pan::Pan;

/// LFO / FEG / cutoff coefficient update period (samples)
const PARAM_BLOCK: u32 = 64;

#[derive(Debug, PartialEq)]
pub enum ToneGeneratorStatus {
    Idle,
    Running,
    Releasing,
}

/// 滤波模型（模型分歧兼容化，2026-08-24）：
/// - `Syxg50`: S-YXG50 的"滤波"= **采样率截止**（FEG 电平 → DSP 0x400 音高字 →
///   播放速率），经 rate_scale → 低通截止注入，音高不联动（A3）。
/// - `Syxg2006LE`: 2006LE Chamberlin SVF LPF（保留过渡路径）。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterModel {
    #[default]
    Syxg50,
    Syxg2006LE,
}

#[derive(Debug)]
pub struct ToneGenerator {
    // update when NoteOn
    pub attack_time: time::Instant,
    /// Monotonic id of the NoteOn that created this voice. All element voices
    /// of one NoteOn share the id, so a NoteOff releases the whole group
    /// (dual-element voices) instead of only the earliest single voice.
    pub note_on_id: u64,
    // update when NoteOff/NoteOn(vel=0)
    pub release_time: time::Instant,
    /// Virtual (render-time) elapsed since release — drives the release timeout
    /// even for voices whose AEG is disabled (they never reach Finished).
    release_elapsed: Duration,

    pub status: ToneGeneratorStatus,
    pub scoring_config: ScoringConfig,

    pub note: Option<Note>,

    /// Sustain pedal mode snapshot from the element (2006LE: 0/1/2;
    /// S-YXG50 data → 0). Drives the damper-hold policy while the pedal is held.
    pub sustain_mode: u8,
    /// Damper-hold active: the AEG decays through the Damp stage instead of
    /// freezing at sustain_level while the sustain pedal holds the NoteOff.
    pub damper_hold: bool,
    /// When this voice last became idle (updated on kill) — used by the
    /// renderer's idle-voice allocation to give just-released voices a buffer.
    pub idle_since: std::time::Instant,
    /// Element layer index (0=main, 1=second element of a dual-element voice)
    pub element_index: u8,

    pub part: Option<Arc<DoubleBuffered<Part>>>,

    pub lfo: LFO,

    pub oscillator: Oscillator,
    pub lpf: LPF,
    pub hpf: HPF,
    pub amp: Amp,
    pub feg: FEG,
    pub cutoff: CutOff,
    pub eq: EQ,
    pub pan: Pan,

    /// 滤波模型（Syxg50 采样率截止 / Syxg2006LE LPF），默认 Syxg50。
    pub filter_model: FilterModel,
    /// 键跟 A（elem[44] amount / elem[45] ref，FUN_10013e20）→ S-YXG50 采样率
    /// 截止深度的键位项（note-on 快照）。
    key_follow_a: f32,

    /// Output sample rate (Hz)
    sample_rate: f32,
    /// LFO frequency (Hz, snapshot from 08 pp 15 at note-on)
    lfo_freq: f32,
    /// LFO pitch modulation depth (cent, vibrato depth)
    lfo_pitch_depth: f32,
    /// LPF Q value (note-on snapshot)
    lpf_q: f32,
    /// HPF base cutoff parameter (0A pp 20, + LFO CM modulation each block)
    hpf_base: f32,
    /// HPF Q value (note-on snapshot)
    hpf_q: f32,

    // ── Modulation depth snapshot (from MultiPart at play) ──
    /// Depth coefficient (d/64, 64 = standard 1.0)
    mod_mw_pitch: f32,
    mod_mw_filter: f32,
    mod_mw_amp: f32,
    mod_bend_pitch: f32,
    mod_bend_filter: f32,
    mod_bend_amp: f32,
    mod_cat_pitch: f32,
    mod_cat_filter: f32,
    mod_cat_amp: f32,
    mod_pat_pitch: f32,
    mod_pat_filter: f32,
    mod_pat_amp: f32,
    /// HPF modulation depth (MultiPartExt 0A pp 22-29)
    mod_hpf_mw: f32,
    mod_hpf_bend: f32,
    mod_hpf_cat: f32,
    mod_hpf_pat: f32,
    /// AC1/AC2 control number (08 pp 59/60) + depth (08 pp 61-66)
    pub ac1_cc: u8,
    pub ac2_cc: u8,
    /// CBC1/CBC2 control number (0A pp) + depth
    pub cbc1_cc: u8,
    pub cbc2_cc: u8,
    mod_cbc1_pitch: f32,
    mod_cbc1_filter: f32,
    mod_cbc1_amp: f32,
    mod_cbc2_pitch: f32,
    mod_cbc2_filter: f32,
    mod_cbc2_amp: f32,
    /// CBC LFO depth (0-127, pmod/fmod/amod)
    cbc1_pmod: f32,
    cbc1_fmod: f32,
    cbc1_amod: f32,
    cbc2_pmod: f32,
    cbc2_fmod: f32,
    cbc2_amod: f32,
    /// offset level control depth (0A pp, 64 = neutral)
    mod_mw_level: f32,
    mod_bend_level: f32,
    mod_cat_level: f32,
    mod_pat_level: f32,
    mod_ac1_level: f32,
    mod_ac2_level: f32,
    mod_ac1_pitch: f32,
    mod_ac1_filter: f32,
    mod_ac1_amp: f32,
    mod_ac2_pitch: f32,
    mod_ac2_filter: f32,
    mod_ac2_amp: f32,
    /// LFO depth base (snapshot at play, × real-time MW each block)
    amod_depth_base: f32,
    fmod_depth_base: f32,
    pmod_depth_base: f32,
    /// vibrato base depth (cent)
    vib_pitch_base: f32,

    /// Element output enable (element[44] output_en; false → output 0)
    output_enable: bool,

    /// Effect send levels (updated each block from 08 pp 2B-2F, XG_LEVEL linear gain)
    pub dry_level: f32,
    pub chorus_send: f32,
    pub reverb_send: f32,
    pub variation_send: f32,
    /// Enabled insertion effect numbers (03 nn, snapshot each block from Part.insertion_effects)
    pub insertion_effects: Vec<u8>,
    /// Bound part id (snapshot at play)
    pub part_id: usize,
    /// Drum note parameters (DrumSetup, note-on snapshot; None = not a drum)
    drum_params: Option<DrumParams>,
    /// Drum alternate group (voice stealing within group, 0 = none)
    pub drum_group: u8,

    /// Parameter update counter
    param_counter: u32,

    /// Current sample in the per-frame processing chain (osc → lpf → hpf → amp → eq)
    bus: f32,
    /// Current frame duration (drives the oscillator DDS)
    frame_duration: time::Duration,
}

impl ToneGenerator {
    pub fn new(source_sample_rate: f32, target_sample_rate: f32, scoring: ScoringConfig) -> Self {
        Self {
            attack_time: Instant::now(),
            note_on_id: 0,
            release_time: Instant::now(),
            release_elapsed: Duration::ZERO,
            status: ToneGeneratorStatus::Idle,
            part: None,
            note: None,
            lfo: LFO::new(),
            oscillator: Oscillator::new(source_sample_rate, target_sample_rate),
            lpf: LPF::new(),
            hpf: HPF::new(),
            amp: Amp::new(),
            feg: FEG::new(),
            cutoff: CutOff::new(),
            eq: EQ::new(target_sample_rate),
            pan: Pan::new(),
            filter_model: FilterModel::default(),
            key_follow_a: 0.0,
            scoring_config: scoring,
            sample_rate: target_sample_rate,
            lfo_freq: 0.0,
            lfo_pitch_depth: 0.0,
            lpf_q: 1.0,
            hpf_base: 0.0,
            hpf_q: 1.0,
            mod_mw_pitch: 0.0,
            mod_mw_filter: 0.0,
            mod_mw_amp: 0.0,
            mod_bend_pitch: 0.0,
            mod_bend_filter: 0.0,
            mod_bend_amp: 0.0,
            mod_cat_pitch: 0.0,
            mod_cat_filter: 0.0,
            mod_cat_amp: 0.0,
            mod_pat_pitch: 0.0,
            mod_pat_filter: 0.0,
            mod_pat_amp: 0.0,
            mod_hpf_mw: 0.0,
            mod_hpf_bend: 0.0,
            mod_hpf_cat: 0.0,
            mod_hpf_pat: 0.0,
            sustain_mode: 0,
            damper_hold: false,
            idle_since: std::time::Instant::now(),
            element_index: 0,
            ac1_cc: 0x11,
            ac2_cc: 0x12,
            cbc1_cc: 0x12,
            cbc2_cc: 0x13,
            mod_cbc1_pitch: 0.0,
            mod_cbc1_filter: 0.0,
            mod_cbc1_amp: 0.0,
            mod_cbc2_pitch: 0.0,
            mod_cbc2_filter: 0.0,
            mod_cbc2_amp: 0.0,
            cbc1_pmod: 0.0,
            cbc1_fmod: 0.0,
            cbc1_amod: 0.0,
            cbc2_pmod: 0.0,
            cbc2_fmod: 0.0,
            cbc2_amod: 0.0,
            mod_mw_level: 0.0,
            mod_bend_level: 0.0,
            mod_cat_level: 0.0,
            mod_pat_level: 0.0,
            mod_ac1_level: 0.0,
            mod_ac2_level: 0.0,
            mod_ac1_pitch: 0.0,
            mod_ac1_filter: 0.0,
            mod_ac1_amp: 0.0,
            mod_ac2_pitch: 0.0,
            mod_ac2_filter: 0.0,
            mod_ac2_amp: 0.0,
            amod_depth_base: 0.0,
            fmod_depth_base: 0.0,
            pmod_depth_base: 0.0,
            vib_pitch_base: 0.0,
            output_enable: true,
            dry_level: 1.0,
            chorus_send: 0.0,
            reverb_send: 0.0,
            variation_send: 0.0,
            insertion_effects: vec![],
            part_id: usize::MAX,
            drum_params: None,
            drum_group: 0,
            param_counter: 0,
            bus: 0.0,
            frame_duration: time::Duration::ZERO,
        }
    }

    pub fn bonded_to_part(&self, part: &Arc<DoubleBuffered<Part>>) -> bool {
        self.part.as_ref().is_some_and(|p| Arc::ptr_eq(p, part))
    }

    /// 设置滤波模型（Syxg50 采样率截止 / Syxg2006LE LPF）。由 AudioRender 从配置传播。
    pub fn set_filter_model(&mut self, model: FilterModel) {
        self.filter_model = model;
        if model == FilterModel::Syxg50 {
            self.oscillator.rate_scale = 1.0;
        }
    }

    pub fn bonded_to_channel(&self, channel: u8) -> bool {
        self.part
            .as_ref()
            .is_some_and(|p| p.snapshot().ram.snapshot().rcv_channel == channel)
    }

    pub fn get_note(&self) -> Option<Note> {
        self.note
    }

    /// Stereo output: mono signal chain + Pan (for audio_render mixing)
    /// Per-frame audio processing chain:
    /// `osc → lpf → hpf → amp → eq → pan`
    pub fn tick_stereo(&mut self, elapsed: time::Duration) -> (f32, f32) {
        self.frame_duration = elapsed;
        if !self.advance_runtime(elapsed) {
            return (0.0, 0.0);
        }

        self.osc().lpf().hpf().amp().eq().pan()
    }

    pub fn play(
        &mut self,
        note: Note,
        vel: u8,
        note_on_id: u64,
        part: Arc<DoubleBuffered<Part>>,
        element_index: usize,
        drum_setup: Option<Arc<DoubleBuffered<[DrumSetupWrapper; 16]>>>,
    ) {
        log_debug_ln!("tone generator got note={:?} vel={}", note, vel);
        self.note_on_id = note_on_id;
        self.element_index = element_index as u8;
        self.part = Some(part.clone());
        self.part_id = part.snapshot().id;
        self.note = Some(note);
        self.oscillator.bind_part(part.clone());
        // LFO Key sync: reset the phase on note-on (Key running mode)
        if matches!(self.lfo.runing_mode, LFORunningMode::Key) {
            self.lfo.set_accumulator(0, self.lfo_freq as u32);
        }
        self.oscillator.velocity = vel;
        self.oscillator.pitch.play(note);

        // Bind sample (S-YXG50 data → 2006LE program parameters)
        // element_index: 0=main layer, 1=second element (2006LE: 2 elements with independent processing chains mixed)
        if let Some(p) = self.part.as_ref().map(|p| p.snapshot()) {
            if let Some(program) = &p.program_entry {
                if let Some(key) = program[note as usize].as_ref() {
                    // Select the sample for the specified element by velocity (no sound if velocity range does not match)
                    if let Some(sample) = key.sample_at(vel, element_index) {
                        // Drum note parameters: RAM dynamic (3n) > static key.drum_setup
                        let is_drum = key.drum_setup.is_some();
                        let drum_ram = if is_drum {
                            drum_setup.as_ref().map(|db| db.snapshot())
                        } else {
                            None
                        };
                        let drum_note_idx = (note as u8 as usize).saturating_sub(12).min(78);
                        let drum_setup_idx = (p.ram.snapshot().part_mode as usize)
                            .saturating_sub(2)
                            .min(15);
                        let drum: Option<DrumParams> = if let Some(arr) = &drum_ram {
                            let ds = &arr[drum_setup_idx][drum_note_idx];
                            Some(DrumParams {
                                pitch_coarse: ds.pitch_coarse,
                                pitch_fine: ds.pitch_fine,
                                level: ds.level,
                                pan: ds.pan,
                                reverb_send: ds.reverb_send,
                                chorus_send: ds.chorus_send,
                                variation_send: ds.variation_send,
                                filter_cutoff: ds.filter_cutoff_freq,
                                filter_resonance: ds.filter_resonance,
                                eg_attack: ds.eg_attack_rate,
                                eg_decay: ds.eg_decay1_rate,
                                eg_release: ds.eg_decay2_rate,
                            })
                        } else {
                            key.drum_setup.map(|ds| DrumParams {
                                pitch_coarse: ds.pitch_coarse,
                                pitch_fine: ds.pitch_fine,
                                level: ds.level,
                                pan: ds.pan,
                                reverb_send: ds.reverb_send,
                                chorus_send: ds.chorus_send,
                                variation_send: ds.variation_send,
                                filter_cutoff: ds.filter_cutoff_freq,
                                filter_resonance: ds.filter_resonance,
                                eg_attack: ds.eg_attack,
                                eg_decay: ds.eg_decay1,
                                eg_release: ds.eg_decay2,
                            })
                        };
                        self.drum_params = drum;
                        // Alternate group (RAM dynamic takes priority)
                        self.drum_group = drum_ram
                            .as_ref()
                            .map(|arr| arr[drum_setup_idx][drum_note_idx].alternate_group)
                            .or_else(|| key.drum_setup.map(|d| d.alter_group))
                            .unwrap_or(0);
                        self.oscillator
                            .setup(sample, note as u8, vel, self.sample_rate);
                        // LPF parameters: VCE base + Part relative offset
                        let mp = Some(p.ram.snapshot());
                        let (
                            cutoff_off,
                            reso_off,
                            vib_rate,
                            vib_depth,
                            vib_delay,
                            feg_depth,
                            eg_a,
                            eg_d,
                            eg_r,
                            lfo_fmod,
                        ) = match &mp {
                            Some(m) => (
                                m.filter_cutoff_freq as f32 - 64.0, // 08 pp 18
                                m.filter_resonance as f32 - 64.0,   // 08 pp 19
                                m.vibrato_rate as f32,              // 08 pp 15
                                m.vibrato_depth as f32,             // 08 pp 16
                                m.vibrato_delay,                    // 08 pp 17
                                m.filter_eg_depth as f32,           // 08 pp 71
                                m.eg_attack_time,
                                m.eg_decay_time,
                                m.eg_release_time,
                                m.mw.lfo_fmod_depth, // 08 pp 21 (LFO→cutoff)
                            ),
                            None => (0.0, 0.0, 0.0, 0.0, 0, 0.0, 0x40, 0x40, 0x40, 0),
                        };

                        self.cutoff.base = self
                            .drum_params
                            .map_or(sample.filter_cutoff as f32, |d| d.filter_cutoff as f32);
                        self.cutoff.part_offset = cutoff_off;
                        self.cutoff.feg_depth =
                            CutOff::feg_depth_param(feg_depth.clamp(0.0, 127.0) as u8);
                        self.cutoff.lfo_depth = CutOff::lfo_depth_param(lfo_fmod);

                        // 键跟 A（elem[44] amount / elem[45] ref，FUN_10013e20）：
                        // `kfa = (key − elem[45]) × (elem[44] − 0x40)`，→ S-YXG50 采样率
                        // 截止深度键位项（A2 接入）。力度缩放项（velocity sense）暂缺。
                        self.key_follow_a = key_follow(
                            note as u8,
                            sample.keyfol_ref, // elem[45]
                            sample.output_en,  // elem[44]（复用 output_en 字节）
                        ) as f32;

                        // ⚠ 2026-08-12 对齐修正：element[14] 是音高公式分量（FUN_10015460
                        // @0x100154cd），非滤波器共鸣；引擎渲染链无共振滤波环节 →
                        // 旋律音色共鸣固定中性 64。鼓组（DrumData[12]）保留。
                        let reso_base =
                            self.drum_params.map_or(64.0, |d| d.filter_resonance as f32);
                        let q = LPF::resonance_param_to_q(
                            (reso_base + reso_off).clamp(0.0, 127.0) as u8
                        );
                        self.lpf_q = q;
                        self.lpf.set_params(self.cutoff.compute_param(0.0, 0.0), q);
                        self.lpf.reset();

                        // FEG（2006LE LPF 滤波 EG；S-YXG50 的 CS/LS 包络调制的是采样率截止
                        // ——非 LPF，见 element_alignment.md「键跟随」节模型不匹配说明）
                        self.feg
                            .setup(eg_a, eg_d, eg_r, feg_depth.clamp(0.0, 127.0) as u8);

                        // Amp: velocity (get_velocity) + part volume + AEG times
                        if let Some(m) = &mp {
                            let (a, d, r) = self.drum_params.map_or((eg_a, eg_d, eg_r), |ds| {
                                (ds.eg_attack, ds.eg_decay, ds.eg_release)
                            });
                            // 键跟 B（[66]/[67]）→ key_on_delay 公式（FUN_10012670）用于鼓。
                            // ⚠ 2026-08-18 旋律不应用 key_on_delay：二进制确认 S-YXG50 的
                            // voice[0x66]（elem[72] key_on_delay）**只写不读**（母机忽略）。
                            // 此前把元素 key_on_delay=31 经 (31+kf)×2 → 62 → 延迟表 81-139ms，
                            // AEG 停在 Delay(level=0) → 采样音头（锤击瞬态）被静音 =
                            // 「顶八度发软/没音头」（DFT 实证 C7 攻击峰值 0.188→0.850）。
                            let kf_b = key_follow(
                                note as u8,
                                sample.fmt_flag,        // elem[67]（键跟 B 基准 ref）
                                sample.keyfollow_depth, // elem[66]（键跟 B 深度 amount）
                            );
                            let kd = if self.drum_params.is_some() {
                                key_on_delay_index(
                                    sample.key_on_delay, // elem[72]
                                    eg_r, // part[0x1c]（Part EG Release Time 08 pp 1C）
                                    sample.aeg_rel, // elem[57]（表索引偏移）
                                    kf_b,
                                )
                            } else {
                                0 // 旋律：母机忽略 key_on_delay → 无起音延迟
                            };
                            self.amp.setup(vel, m, a, d, r, kd);
                            // H2 (2026-08-12): 每元素 AEG 时间由曲线 A 驱动（引擎
                            // FUN_100164f0/FUN_10013580 → 2×4-bit → EG 段速率）。
                            if self.drum_params.is_none() {
                                let curve_a = piecewise_curve(
                                    note.into(),
                                    [
                                        sample.curve_a_x0,
                                        sample.pitch_coarse,
                                        sample.curve_a_x2,
                                        sample.curve_a_x3,
                                    ],
                                    [
                                        sample.curve_a_y0,
                                        sample.curve_a_y1,
                                        sample.eg_filt_en,
                                        sample.eg_amp_en,
                                    ],
                                );
                                let rate =
                                    (curve_a + 0x40 + (vel as i32 - 64) / 2).clamp(0, 0x7f) as u8;
                                let eg_t = eg_time_ms(rate);
                                // 攻击保持 Part 默认（快——采样击打承担瞬态；yxg50 实测
                                // 25ms 单峰，无第二峰——elem0 不应慢攻）
                                self.amp.aeg.set_element_eg(self.amp.aeg.attack_time, eg_t);
                                // release：curve_a > 0 → eg_time_ms(max(curve_a,0x18)+0x10)
                                // （音乐盒 9 → 33 → 1.7s ≈ yxg50 实测 note-off 后 ~1.7s 衰减完）；
                                // 否则 Part 默认（快停止）
                                // ⚠ 2026-08-17 曾试接 aeg_rel[57]/aeg_d1_val[55] → 音色退化
                                // （钢琴像电钢）——aeg_d1_val 疑为 Decay1 电平非速率，已回退。
                                let rel_t = if curve_a > 0 {
                                    eg_time_ms((curve_a.max(0x18) + 0x10).clamp(0, 0x7f) as u8)
                                } else {
                                    self.amp.aeg.release_time
                                };
                                self.amp.aeg.set_element_release(rel_t);
                                // S-YXG50 元素 [70]（wave_pitch）→ EG 段目标表索引（×2）→
                                // sustain 电平（表值 log 域分段，引擎实测校准）：
                                // - 表值 < 0x100（钢琴 0xf0/String/NylonGtr）→ 0.7 延音
                                //   （引擎钢琴为慢衰减 ~2s，0.7 冻结近似——旧行为恢复）
                                // - 表值 0x100..0x8000（Marimba 0x600）→ 0.014 衰减
                                // - 表值 ≥ 0x8000（Organ 0xf83e0）→ 0.4687 保持
                                let sustain = {
                                    // ⚠ 2026-08-18 音乐盒（prog 10）特判：其 elem wave_pitch
                                    // → t=0x900..0x1601 落入 [0x100,0x8000) → 0.014 类，
                                    // 造成 ~30ms 即塌成静音（声音太短）。音乐盒 tin 应持续鸣响
                                    // （long-ring，同钢琴 0.7 语义）——给 0.55，note-off 再经 release 收尾。
                                    let prog = p.ram.snapshot().program_number;
                                    if prog == 10 {
                                        0.55
                                    } else {
                                        let idx = (sample.wave_pitch & 0x7f) as usize * 2;
                                        let t =
                                            EG_TARGET_TABLE.get(idx).copied().unwrap_or(0xf83e0);
                                        if t <= 0x100 {
                                            0.7
                                        } else if t < 0x8000 {
                                            0.014
                                        } else {
                                            0.4687
                                        }
                                    }
                                };
                                self.amp.aeg.set_element_sustain(sustain);
                            }
                            // Element volume offset (element[8], signed, +0.1dB/unit)
                            self.amp.element_gain = vol_offset_gain(sample.vol_offset);
                            // P1 (2026-08-13): 每元素音量平衡（FUN_100156c0 语义）：
                            // `volume_param = clamp(vel × [55]/99 + 曲线B(key)×2, 0, 0x80)`
                            // —— elem[55] 力度缩放 + 曲线 B 键缩 → 每元素音量。
                            // 音乐盒：elem0 [55]=115、曲线B(72)=−22 → 0.56；
                            // elem1 [55]=59、曲线B=0 → 0.47 → 尾音 = elem0 正弦主导（yxg50 实测）。
                            if self.drum_params.is_none() {
                                let curve_b = piecewise_curve(
                                    note.into(),
                                    [
                                        sample.aeg_d2,  // 曲线B x0 (elem[56])
                                        sample.aeg_rel, // 曲线B x1 (elem[57])
                                        sample.curve_b_x2,
                                        sample.curve_b_x3,
                                    ],
                                    [
                                        sample.curve_b_y0,
                                        sample.curve_b_y1,
                                        sample.curve_b_y2,
                                        sample.curve_b_y3,
                                    ],
                                );
                                let vol_param = (vel as f32 * sample.aeg_d1_val as f32 / 99.0
                                    + curve_b as f32 * 2.0)
                                    .clamp(0.0, 128.0);
                                self.amp.element_gain *= vol_param / 128.0;
                            }
                            // Drum note level (DrumSetup, 0-127) as a volume coefficient
                            if let Some(ds) = self.drum_params {
                                self.amp.volume *= ds.level as f32 / 127.0;
                            }
                            self.amp.expression = 1.0;
                            // LFO AM depth (MW LFO AMOD 08 pp 22, default 0 = no effect)
                            self.amp.lfo_depth = m.mw.lfo_amod_depth as f32 / 127.0;
                            self.lfo.amp.depth = 1.0;
                            self.lfo.amp.offset = 0.0;

                            // Pan (08 pp 0E, 0=random); drum notes use DrumSetup pan
                            let pan = self.drum_params.map_or(m.pan, |d| d.pan);
                            self.pan.set(pan);

                            // Pitch EG (08 pp 69-6C, XG ±12 semitones; only non-default values override)
                            // YAMAHA: PITCH EG has no effect on drum parts
                            if self.drum_params.is_none() {
                                self.oscillator.peg.apply_xg_eg(
                                    m.pitch_eg_init_level,
                                    m.pitch_eg_attack_time,
                                    m.pitch_eg_release_level,
                                    m.pitch_eg_release_time,
                                    self.sample_rate,
                                );
                            }

                            // Modulation depth snapshot (08 pp 1D-28, 4D-58).
                            // XG spec: Filter/Amplitude Control = -100%..+100%,
                            // center 0x40 = 0 (no modulation); Pitch Control =
                            // -24..+24 semitones, 0x40 = 0. d() used to be v/64,
                            // which made the DEFAULT 0x40 a full-scale modulation
                            // (bend/mw swept volume ±24dB → "pitchbend changes
                            // volume"; mod_gain 10^(db/20) blew past ×15 → clipping).
                            let d = |v: u8| (v as f32 - 64.0) / 64.0; // filter/amp: -1..1, center 0
                            let dp = |v: u8| v as f32 - 64.0; // pitch: semitones, center 0
                            self.mod_mw_pitch = dp(m.mw.pitch_control);
                            self.mod_mw_filter = d(m.mw.filter_control);
                            self.mod_mw_amp = d(m.mw.amplitude_control);
                            // bend_cent already includes the 08 pp 23 range
                            // (get_pitch_bend_sensitivity); keep the coefficient
                            // at 1.0 to avoid double-applying the range.
                            self.mod_bend_pitch = 1.0;
                            self.mod_bend_filter = d(m.bend.filter_control);
                            self.mod_bend_amp = d(m.bend.amplitude_control);
                            self.mod_cat_pitch = dp(m.cat.pitch_control);
                            self.mod_cat_filter = d(m.cat.filter_control);
                            self.mod_cat_amp = d(m.cat.amplitude_control);
                            self.mod_pat_pitch = dp(m.pat.pitch_control);
                            self.mod_pat_filter = d(m.pat.filter_control);
                            self.mod_pat_amp = d(m.pat.amplitude_control);
                            // LFO depth base (08 pp 20-22), × real-time MW each block
                            self.pmod_depth_base = m.mw.lfo_pmod_depth as f32 / 127.0;
                            self.fmod_depth_base = self.cutoff.lfo_depth; // lfo_fmod_depth already converted
                            self.amod_depth_base = m.mw.lfo_amod_depth as f32 / 127.0;
                            self.vib_pitch_base = self.lfo_pitch_depth; // vibrato depth already converted

                            // AC1/AC2 (08 pp 59-66): control number + depth
                            self.ac1_cc = m.ac[0].controller_number;
                            self.ac2_cc = m.ac[1].controller_number;
                            self.mod_ac1_pitch = dp(m.ac[0].pitch_control);
                            self.mod_ac1_filter = d(m.ac[0].filter_control);
                            self.mod_ac1_amp = d(m.ac[0].amplitude_control);
                            self.mod_ac2_pitch = dp(m.ac[1].pitch_control);
                            self.mod_ac2_filter = d(m.ac[1].filter_control);
                            self.mod_ac2_amp = d(m.ac[1].amplitude_control);

                            // Pitch: note_shift (08 pp 0D, PitchGetter::get_coarse semitones)
                            // + detune (08 pp 0F/10, DETUNE_TO_CENTS table)
                            // + scale_tuning (0A pp 26-31)
                            // All routed through MultiPart's PitchGetter method (get_delta_pitch),
                            // no manual conversion (the 256-entry DETUNE_TO_CENTS table is exact)
                            let mut pitch_extra = (m.get_coarse() as i32) * 100
                                + if self.drum_params.is_some() {
                                    // YAMAHA: SCALE TUNING has no effect on drum parts
                                    m.detune_cents() as i32
                                } else {
                                    m.get_delta_pitch(self.note.unwrap()) as i32
                                };
                            // Drum note: pitch_coarse (0x40 = center, ±64 semitones) + pitch_fine (cents)
                            if let Some(d) = self.drum_params {
                                pitch_extra +=
                                    (d.pitch_coarse as i32 - 64) * 100 + (d.pitch_fine as i32 - 64);
                            }
                            let pitch_extra = pitch_extra as f32;
                            self.oscillator.pitch.note_in_cent += pitch_extra;
                            self.oscillator.portamento.target_note += pitch_extra;

                            // DSP enable bits (element[40..44], [71]):
                            // eg_filt_en / eg_amp_en / lfo_en / eg_pitch_en / output_en / eg_enable
                            let eg_total = sample.eg_enable != 0;
                            self.feg.enabled = eg_total && sample.eg_filt_en != 0;
                            self.amp.aeg.enabled = eg_total && sample.eg_amp_en != 0;
                            self.oscillator.peg.enabled = eg_total && sample.eg_pitch_en != 0;
                            self.lfo.enable = sample.lfo_en != 0;
                            self.output_enable = sample.output_en != 0;
                            // Sustain pedal mode (2006LE data; S-YXG50 → 0)
                            self.sustain_mode = sample.sustain_mode;

                            // ⚠ 2026-08-18 修复「电钢感」：钢琴程序（XG 0-7）无 sustain_mode 字段
                            // （S-YXG50 → 0）时，note-on 即走 Damp 段自然衰减——否则琴键按住时
                            // AEG 恒 hold 在 sustain 0.7（延音不衰减 = 电钢特征）。
                            if self.sustain_mode == 0 {
                                let prog = p.ram.snapshot().program_number;
                                if prog <= 7 {
                                    self.amp.aeg.set_damper(true);
                                }
                            }

                            // ⚠ 2026-08-12 对齐修正（rwatch 裁决）：[54] 引擎 KeyOn 不读；
                            // [56]/[57] = 曲线 B x0/x1（音量键缩）非 sustain/release → 移除覆盖。
                            // AEG 时间由曲线 A 驱动（见 amp.setup 调用点）

                            // EQ (08 pp 72-7F; MID bands Spec NOT USED, still implemented here)
                            self.eq.set_params(
                                EQ::gain_param_to_db(m.eq_bass),
                                EQ::freq_param_to_hz(m.eq_bass_freq),
                                EQ::q_param_to_q(m.eq_bass_q),
                                m.eq_bass_shape == 1, // 1=peaking
                                EQ::gain_param_to_db(m.eq_mid_bass),
                                EQ::freq_param_to_hz(m.eq_mid_bass_freq),
                                EQ::q_param_to_q(m.eq_mid_bass_q),
                                EQ::gain_param_to_db(m.eq_mid_treble),
                                EQ::freq_param_to_hz(m.eq_mid_treble_freq),
                                EQ::q_param_to_q(m.eq_mid_treble_q),
                                EQ::gain_param_to_db(m.eq_treble),
                                EQ::freq_param_to_hz(m.eq_treble_freq),
                                EQ::q_param_to_q(m.eq_treble_q),
                                m.eq_treble_shape == 1,
                            );
                        }

                        // HPF (MultiPartExt 0A pp 20-21)
                        // 2006LE: HPF shares LFO CM modulation with LPF (CLFOUnit::SetupParameter)
                        {
                            let mex = p.ram_ext.snapshot();
                            self.hpf_base = mex.hpf_cutoff_freq as f32;
                            self.hpf_q = HPF::resonance_param_to_q(mex.hpf_resonance);
                            self.hpf.set_params(
                                HPF::cutoff_param_to_hz(mex.hpf_cutoff_freq),
                                self.hpf_q,
                                self.sample_rate,
                            );
                            self.hpf.reset();

                            // HPF modulation depth (0A pp 22-29): filter-class
                            // controls use the -100%..+100% center-0x40 mapping.
                            let d = |v: u8| (v as f32 - 64.0) / 64.0;
                            let dp = |v: u8| v as f32 - 64.0; // pitch: semitones
                            self.mod_hpf_mw = d(mex.mw_hpf_control_depth);
                            self.mod_hpf_bend = d(mex.bend_hpf_control_depth);
                            self.mod_hpf_cat = d(mex.cat_hpf_control_depth);
                            self.mod_hpf_pat = d(mex.pat_hpf_control_depth);

                            // CBC1/CBC2 (0A pp 25-36): control number + depth
                            self.cbc1_cc = mex.cbc1_control_number;
                            self.cbc2_cc = mex.cbc2_control_number;
                            self.mod_cbc1_pitch = dp(mex.cbc1_pitch_control);
                            self.mod_cbc1_filter = d(mex.cbc1_lpf_control);
                            self.mod_cbc1_amp = d(mex.cbc1_amplitude_control);
                            self.mod_cbc2_pitch = dp(mex.cbc2_pitch_control);
                            self.mod_cbc2_filter = d(mex.cbc2_lpf_control);
                            self.mod_cbc2_amp = d(mex.cbc2_amplitude_control);
                            self.cbc1_pmod = mex.cbc1_lfo_pmod_control_depth as f32 / 127.0;
                            self.cbc1_fmod = mex.cbc1_lfo_fmod_control_depth as f32 / 127.0;
                            self.cbc1_amod = mex.cbc1_lfo_amod_control_depth as f32 / 127.0;
                            self.cbc2_pmod = mex.cbc2_lfo_pmod_control_depth as f32 / 127.0;
                            self.cbc2_fmod = mex.cbc2_lfo_fmod_control_depth as f32 / 127.0;
                            self.cbc2_amod = mex.cbc2_lfo_amod_control_depth as f32 / 127.0;

                            // offset level control depth (0A pp 3F-44, 64 = neutral → relative offset)
                            let ld = |v: u8| (v as f32 - 64.0) / 64.0;
                            self.mod_mw_level = ld(mex.mw_offset_level_control);
                            self.mod_bend_level = ld(mex.bend_offset_level_control);
                            self.mod_cat_level = ld(mex.cat_offset_level_control);
                            self.mod_pat_level = ld(mex.pat_offset_level_control);
                            self.mod_ac1_level = ld(mex.ac1_offset_level_control);
                            self.mod_ac2_level = ld(mex.ac2_offset_level_control);
                        }

                        // LFO: waveform + frequency + modulation depth
                        if let Ok(wt) = WaveType::try_from(self.oscillator.lfo_wave) {
                            self.lfo.wave_type = wt;
                        }
                        self.lfo_freq = vib_to_hz(vib_rate as u8);
                        self.lfo.enable = true;
                        // LFO outputs keep the raw waveform (depth is multiplied on the CutOff/subsequent modulation side)
                        self.lfo.pitch.depth = 1.0;
                        self.lfo.lpf.depth = 1.0;
                        self.lfo.pitch.offset = 0.0;
                        self.lfo.lpf.offset = 0.0;
                        // LFO pitch modulation amount (vibrato depth → ±depth/127×100 cents)
                        self.lfo_pitch_depth = vib_depth / 127.0 * 100.0;
                        // LFO attack delay (08 pp 17 → XG Table #2, 0-50ms)
                        self.oscillator.delay.delay_samples =
                            (XG_MODULATION_DELAY_OFFSET_TABLE[vib_delay.min(127) as usize] / 1000.0
                                * self.sample_rate) as u32;
                        self.oscillator.delay.fade_samples = 0;
                        self.oscillator.delay.fade_step = 1.0;
                        self.param_counter = 0;
                    } else {
                        // Velocity range / element-layer mismatch for this
                        // voice: a reused ToneGenerator must NOT ring with the
                        // previous program's sample (program-switch + glissando
                        // could otherwise sound stale voices).
                        self.kill();
                        return;
                    }
                } else {
                    // No key for this note in the current program.
                    self.kill();
                    return;
                }
            } else {
                // No program loaded for this part.
                self.kill();
                return;
            }
        } else {
            self.kill();
            return;
        }

        self.attack_time = Instant::now();
        self.status = ToneGeneratorStatus::Running;
    }

    pub fn scoring(&self) -> u128 {
        let args = &self.scoring_config;
        let mut score = self.attack_time.elapsed().as_millis() * args.time_weight as u128;
        // AEG stage: protect voices still in attack (most recently struck)
        if self.amp.aeg.state == super::amp::aeg::AEGStage::Attack {
            score = score.saturating_mul(2);
        }
        score = match self.status {
            ToneGeneratorStatus::Running => score * args.protect_attack as u128 / 1000,
            ToneGeneratorStatus::Releasing => score * args.penalty_release as u128 / 1000,
            ToneGeneratorStatus::Idle => return 0,
        };

        // if sustain(CC#64) hold
        self.part
            .as_ref()
            .is_some_and(|p| p.snapshot().controller.sustain)
            .then(|| score = score * args.protect_sustain_pedal as u128 / 1000);

        // note protect
        score = if self.drum_params.is_some() {
            score * args.get_drum_scoring_map()[self.get_note().map_or(0, |n| n as usize)] as u128
                / 1000
        } else {
            score * args.get_note_scoring_map()[self.get_note().map_or(0, |n| n as usize)] as u128
                / 1000
        };

        // if non-loop sample
        if !self.oscillator.is_looping() {
            score = score * args.protect_non_looping as u128 / 1000;
        }

        score
    }
}

impl ToneGenerator {
    /// Retarget the render clock to the sink's actual negotiated rate
    pub fn set_output_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        self.oscillator.set_target_rate(rate);
    }
}

impl ToneGeneratorInterface for ToneGenerator {
    fn reset(&mut self) {
        // TG 完整复位：复用（steal/释放后）时无残留状态——否则复用 TG 的
        // AEG/增益/相位残留会导致声音偏小/异常（TODO：偶发声音不清晰）。
        self.status = ToneGeneratorStatus::Idle;
        self.part = None;
        self.note = None;
        self.damper_hold = false;
        self.release_elapsed = Duration::ZERO;
        // amp：AEG 终态 + 增益/调制（mod_gain 残留负 dB = 声音小的根因候选）
        self.amp.kill();
        self.amp.set_mod_gain_db(0.0);
        self.amp.element_gain = 1.0;
        self.amp.lfo_depth = 0.0;
        // oscillator：采样位置/一次性标志/16-bit 权重相位/PEG
        self.oscillator.reset();
        // S-YXG50 采样率截止电平复位（全通，无截止）
        self.oscillator.rate_scale = 1.0;
        // LFO 复位
        self.lfo.enable = false;
        self.lfo.set_accumulator(0, 0);
        // FEG 复位
        self.feg.kill();
    }

    fn kill(&mut self) {
        self.reset();
        self.idle_since = std::time::Instant::now();
    }

    fn release(&mut self) {
        if self.status == ToneGeneratorStatus::Running {
            self.release_time = Instant::now();
            self.release_elapsed = Duration::ZERO;
            self.status = ToneGeneratorStatus::Releasing;
            self.damper_hold = false;
            // Start the AEG release phase; without this the envelope stays in
            // Sustain and the note never decays after NoteOff.
            self.amp.note_off();
        }
    }
}

/// ─────────────────────────────────────────────────────────────────────────
/// 每帧信号链（链式方法，便于逐级排查音质）：
///
///     self.osc().lpf().hpf().amp().eq().pan()
///
/// 每一步从 `bus` 取当前样本、处理后写回；`pan()` 结束链并输出立体声。
/// ─────────────────────────────────────────────────────────────────────────
impl ToneGenerator {
    /// Enable/disable the damper-hold decay for this voice (sustain pedal held
    /// + damper policy). The AEG enters the Damp stage once it reaches Sustain.
    pub fn set_damper_hold(&mut self, on: bool) {
        self.damper_hold = on;
        self.amp.aeg.set_damper(on);
    }

    /// 状态机推进 + 参数块更新。返回 `false` 表示应输出静音（Idle 或已 kill）。
    fn advance_runtime(&mut self, elapsed: time::Duration) -> bool {
        match self.status {
            ToneGeneratorStatus::Idle => return false,
            ToneGeneratorStatus::Running | ToneGeneratorStatus::Releasing => {}
        }
        // One-shot sample exhausted: the source is gone, so the voice ends
        // even if the AEG envelope hasn't finished (osc and AEG stay in sync —
        // no point rendering an AEG tail over a silent source).
        if self.oscillator.finished {
            self.kill();
            return false;
        }
        // Once the AEG finishes its release it is silent; kill the voice so a
        // released note can never stay audible.
        if self.amp.aeg.state == super::amp::aeg::AEGStage::Finished {
            self.kill();
            return false;
        }
        if self.status == ToneGeneratorStatus::Releasing {
            // Virtual (render-time) budget: works for AEG-disabled voices that
            // never reach Finished on their own.
            self.release_elapsed += elapsed;
            let budget = if self.amp.aeg.enabled {
                self.amp.aeg.release_time + Duration::from_millis(200)
            } else {
                Duration::from_secs(2)
            };
            let cap = Duration::from_secs(8);
            if self.release_elapsed >= budget.min(cap) {
                self.kill();
                return false;
            }
        }
        // Update LFO / FEG / cutoff / Amp parameters every PARAM_BLOCK samples
        self.param_counter += 1;
        if self.param_counter >= PARAM_BLOCK {
            self.param_counter = 0;
            self.update_block_parameters();
        }
        true
    }

    /// ── 信号链：振荡器（DDS 采样） ──
    pub fn osc(&mut self) -> &mut Self {
        self.bus = self.oscillator.tick(self.frame_duration);
        self
    }

    /// ── 信号链：低通滤波器 ──
    pub fn lpf(&mut self) -> &mut Self {
        // 模型分歧兼容化（2026-08-24）:
        // - Syxg50: S-YXG50 无 Chamberlin SVF；"滤波"= 采样率截止（FEG 电平→音高字→
        //   播放速率）。为保音高准确（A3）不做 DDS 联动, 而把 rate_scale（[0,1]）映射
        //   为低通截止参数注入现有 LPF（resonance 中性）——频谱低通、音高不变。
        // - Syxg2006LE: 现有 Chamberlin SVF 路径（保留过渡）。
        if self.filter_model == FilterModel::Syxg50 {
            let cutoff_param = self.rate_cutoff_param();
            self.lpf.set_params(cutoff_param, 64.0);
            self.bus = self.lpf.tick(self.bus);
        } else {
            self.bus = self.lpf.tick(self.bus);
        }
        self
    }

    /// S-YXG50 采样率截止电平 rate_scale（[0,1]）→ 低通截止参数（0-127）。
    /// rate_scale=1（无截止）→ param 127（全通）；rate_scale→0 → param→0（最暗）。
    /// 0x10047F50 类 level→brightness 映射的线性近似（表驱动校准待二期）。
    fn rate_cutoff_param(&self) -> f32 {
        (self.oscillator.rate_scale.clamp(0.0, 1.0) * 127.0)
            .round()
            .clamp(0.0, 127.0)
    }

    /// ── 信号链：高通滤波器 ──
    pub fn hpf(&mut self) -> &mut Self {
        self.bus = self.hpf.tick(self.bus);
        self
    }

    /// ── 信号链：放大器（AEG 包络 + 调制增益） ──
    pub fn amp(&mut self) -> &mut Self {
        self.bus = self.amp.tick(
            self.bus,
            Duration::from_secs_f32(1.0 / self.sample_rate),
            self.lfo.amp.output,
        );
        self
    }

    /// ── 信号链：EQ（元素 output_enable=false 时输出 0） ──
    pub fn eq(&mut self) -> &mut Self {
        self.bus = if self.output_enable {
            self.eq.tick(self.bus)
        } else {
            0.0
        };
        self
    }

    /// ── 信号链终点：声像（单声道样本 → 立体声） ──
    pub fn pan(&mut self) -> (f32, f32) {
        self.pan.apply(self.bus)
    }

    /// 链式处理后的单声道样本（诊断用）
    pub fn output(&self) -> f32 {
        self.bus
    }

    /// 参数块更新（每 PARAM_BLOCK 样本一次）：LFO / FEG / 滤波器 / Amp / 调制
    fn update_block_parameters(&mut self) {
        let block_elapsed = Duration::from_secs_f32(PARAM_BLOCK as f32 / self.sample_rate);
        self.update_lfo();
        self.update_feg_and_filters(block_elapsed);
        self.update_osc_pitch_lfo();
        if let Some(p) = self.part.as_ref().map(|p| p.snapshot()) {
            self.update_amp_and_sends(&p);
            self.update_modulation_sources(&p);
        }
    }

    /// LFO 推进 → 波形输出
    fn update_lfo(&mut self) {
        self.lfo
            .update_accumulator(self.lfo_freq, PARAM_BLOCK, self.sample_rate as u32);
        self.lfo.make_wave();
    }

    /// FEG 推进 + 滤波器参数（LPF 截止、HPF 截止）
    fn update_feg_and_filters(&mut self, block_elapsed: Duration) {
        let feg_level = self.feg.tick(block_elapsed);
        // S-YXG50 采样率截止：FEG 电平 → rate_scale（[0,1], 1=无截止全通）。
        // Syxg2006LE 模式恒 1.0（该注入仅 Syxg50 模式消费）。
        if self.filter_model == FilterModel::Syxg50 {
            // feg.level∈[0,1] → rate_scale = 1 − feg.level×0.5（0.5 校准系数, 见
            // dev_docs/syxg50_sample_rate_cutoff.md §5）。键跟 A 作为额外深度项：
            // kfa ∈ [−0.44,+0.44]（默认 0x40 中性 → 0）, 与 FEG 叠加对暗化增益项。
            let kfa = (self.key_follow_a / 128.0).clamp(-0.44, 0.44);
            self.oscillator.rate_scale = (1.0 - feg_level * 0.5 + kfa).clamp(0.05, 1.0);
        } else {
            self.oscillator.rate_scale = 1.0;
        }
        // CutOff = base + part offset + FEG×depth + LFO×depth
        let param = self.cutoff.compute_param(feg_level, self.lfo.lpf.output);
        self.lpf.set_params(param, self.lpf_q);
        // HPF: shares LFO CM modulation with LPF (2006LE CLFOUnit)
        let hpf_param = self.hpf_base + self.lfo.lpf.output * self.cutoff.lfo_depth;
        self.hpf.set_params(
            HPF::cutoff_param_to_hz(hpf_param.round().clamp(0.0, 127.0) as u8),
            self.hpf_q,
            self.sample_rate,
        );
    }

    /// LFO 音高调制 → 振荡器 delay 输入
    fn update_osc_pitch_lfo(&mut self) {
        let lfo_pitch = self.lfo.pitch.output * self.lfo_pitch_depth;
        self.oscillator.set_lfo(lfo_pitch);
    }

    /// Amp：Expression（CC#11）+ Volume（CC#7）+ Pan（CC#10）+ 效果发送电平 + 插入效果快照
    fn update_amp_and_sends(&mut self, p: &Part) {
        let r = p.ram.snapshot();
        self.amp.update(p.controller.expression, r.volume);
        // Pan (08 pp 0E): real-time for melodic parts (CC#10); drum notes use
        // the per-note DrumSetup pan snapshot from note-on.
        if self.drum_params.is_none() {
            self.pan.set(r.pan);
        }
        // Effect send levels (08 pp 2B-2F → XG_LEVEL linear gain)
        // Drum note: DrumSetup sends override part sends (XG Spec: drum per-note sends)
        self.dry_level = xg_level_gain(r.dry_level);
        if let Some(d) = self.drum_params {
            self.chorus_send = xg_level_gain(d.chorus_send);
            self.reverb_send = xg_level_gain(d.reverb_send);
            self.variation_send = xg_level_gain(d.variation_send);
        } else {
            self.chorus_send = xg_level_gain(r.chorus_send);
            self.reverb_send = xg_level_gain(r.reverb_send);
            self.variation_send = xg_level_gain(r.variation_send);
        }
        // Insertion effect numbers (Part.insertion_effects)
        self.insertion_effects.clone_from(&p.insertion_effects);
    }

    /// 实时调制源归一化 + 逐项应用（MW/Bend/CAT/PAT、AC1/AC2、CBC1/CBC2、offset level）
    fn update_modulation_sources(&mut self, p: &Part) {
        let mw = p.controller.modulation as f32 / 127.0;
        let bend_cent = p.get_pitchbend();
        let bend_norm = (p.pitchbend as f32 - 8192.0) / 8192.0;
        let cat = p.cat_value as f32 / 127.0;
        let pat = p
            .pat_values
            .get(self.note.map_or(0, |n| n as u8) as usize)
            .map_or(0.0, |&v| v as f32 / 127.0);
        let ac1 = p.controller.cc_values[self.ac1_cc as usize] as f32 / 127.0;
        let ac2 = p.controller.cc_values[self.ac2_cc as usize] as f32 / 127.0;
        let cbc1 = p.controller.cc_values[self.cbc1_cc as usize] as f32 / 127.0;
        let cbc2 = p.controller.cc_values[self.cbc2_cc as usize] as f32 / 127.0;

        // LFO depth controlled by MW (08 pp 20-22)
        self.amp.lfo_depth = self.amod_depth_base * mw;
        self.cutoff.lfo_depth = self.fmod_depth_base * mw;
        self.lfo_pitch_depth = self.vib_pitch_base + self.pmod_depth_base * mw * 100.0;

        self.apply_mw_bend_cat_pat_mods(mw, bend_cent, bend_norm, cat, pat);
        self.apply_ac_mods(ac1, ac2);
        self.apply_cbc_mods(cbc1, cbc2);
        self.apply_offset_levels(mw, bend_norm, cat, pat, ac1, ac2);
    }

    /// 直接调制：MW/Bend/CAT/PAT → pitch (cents) / filter (param) / amp (dB) / HPF
    fn apply_mw_bend_cat_pat_mods(
        &mut self,
        mw: f32,
        bend_cent: f32,
        bend_norm: f32,
        cat: f32,
        pat: f32,
    ) {
        let f_mw = mw * self.mod_mw_filter * 24.0;
        let f_bend = bend_norm * self.mod_bend_filter * 24.0;
        let f_cat = cat * self.mod_cat_filter * 24.0;
        let f_pat = pat * self.mod_pat_filter * 24.0;
        let a_mw = mw * self.mod_mw_amp * 24.0;
        let a_bend = bend_norm * self.mod_bend_amp * 24.0;
        let a_cat = cat * self.mod_cat_amp * 24.0;
        let a_pat = pat * self.mod_pat_amp * 24.0;

        self.oscillator.pitch_mod = mw * self.mod_mw_pitch * 100.0
            + bend_cent * self.mod_bend_pitch
            + cat * self.mod_cat_pitch * 100.0
            + pat * self.mod_pat_pitch * 100.0;
        self.cutoff.mod_offset = f_mw + f_bend + f_cat + f_pat;
        self.amp.set_mod_gain_db(a_mw + a_bend + a_cat + a_pat);

        // HPF modulation (0A pp 22-29)
        self.hpf.mod_offset = mw * self.mod_hpf_mw * 24.0
            + bend_norm * self.mod_hpf_bend * 24.0
            + cat * self.mod_hpf_cat * 24.0
            + pat * self.mod_hpf_pat * 24.0;
    }

    /// AC1/AC2 (08 pp 59-66): control number → real-time CC value
    fn apply_ac_mods(&mut self, ac1: f32, ac2: f32) {
        self.oscillator.pitch_mod +=
            ac1 * self.mod_ac1_pitch * 100.0 + ac2 * self.mod_ac2_pitch * 100.0;
        self.cutoff.mod_offset +=
            ac1 * self.mod_ac1_filter * 24.0 + ac2 * self.mod_ac2_filter * 24.0;
        self.amp
            .add_mod_gain_db(ac1 * self.mod_ac1_amp * 24.0 + ac2 * self.mod_ac2_amp * 24.0);
    }

    /// CBC1/CBC2 (0A pp 25-36): control number → real-time CC value
    fn apply_cbc_mods(&mut self, cbc1: f32, cbc2: f32) {
        self.oscillator.pitch_mod +=
            cbc1 * self.mod_cbc1_pitch * 100.0 + cbc2 * self.mod_cbc2_pitch * 100.0;
        self.cutoff.mod_offset +=
            cbc1 * self.mod_cbc1_filter * 24.0 + cbc2 * self.mod_cbc2_filter * 24.0;
        self.amp
            .add_mod_gain_db(cbc1 * self.mod_cbc1_amp * 24.0 + cbc2 * self.mod_cbc2_amp * 24.0);
        // CBC LFO depth (pmod/fmod/amod)
        self.lfo_pitch_depth += cbc1 * self.cbc1_pmod * 100.0 + cbc2 * self.cbc2_pmod * 100.0;
        self.cutoff.lfo_depth += cbc1 * self.cbc1_fmod * 40.0 + cbc2 * self.cbc2_fmod * 40.0;
        self.amp.lfo_depth += cbc1 * self.cbc1_amod + cbc2 * self.cbc2_amod;
    }

    /// offset level (0A pp 3F-44): modulation source → level offset (±24dB)
    fn apply_offset_levels(
        &mut self,
        mw: f32,
        bend_norm: f32,
        cat: f32,
        pat: f32,
        ac1: f32,
        ac2: f32,
    ) {
        self.amp.add_mod_gain_db(
            (mw - 0.5) * self.mod_mw_level * 24.0
                + (bend_norm - 0.0) * self.mod_bend_level * 24.0
                + (cat - 0.5) * self.mod_cat_level * 24.0
                + (pat - 0.5) * self.mod_pat_level * 24.0
                + (ac1 - 0.5) * self.mod_ac1_level * 24.0
                + (ac2 - 0.5) * self.mod_ac2_level * 24.0,
        );
    }
}

impl Audio for ToneGenerator {
    /// 单声道链（诊断用；实时路径走 `tick_stereo` 的 osc→…→pan 链）
    fn tick(&mut self, elapsed: time::Duration) -> f32 {
        self.frame_duration = elapsed;
        if !self.advance_runtime(elapsed) {
            return 0.0;
        }
        self.osc().lpf().hpf().amp().eq().output()
        //self.osc().output()
    }
}

/// S-YXG50 EG 段目标表（log 域 uint32，128 项——动态 dump 自引擎内存
/// 0x01dc2f48，索引 = 元素 [70]（wave_pitch）× 2）
static EG_TARGET_TABLE: [u32; 128] = [
    0x10, 0x11, 0x14, 0x16, 0x18, 0x1a, 0x1c, 0x1e, 0x20, 0x24, 0x28, 0x2c, 0x30, 0x34, 0x38, 0x3c,
    0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
    0x100, 0x120, 0x140, 0x160, 0x180, 0x1a0, 0x1c0, 0x1e0, 0x200, 0x240, 0x280, 0x2c0, 0x300,
    0x340, 0x380, 0x3c0, 0x400, 0x480, 0x500, 0x580, 0x600, 0x680, 0x700, 0x780, 0x800, 0x900,
    0xa00, 0xb00, 0xc00, 0xd00, 0xe00, 0xf00, 0x1001, 0x1201, 0x1401, 0x1601, 0x1801, 0x1a01,
    0x1c01, 0x1e01, 0x2002, 0x2402, 0x2802, 0x2c03, 0x3003, 0x3403, 0x3803, 0x3c03, 0x4004, 0x4804,
    0x5005, 0x5806, 0x6006, 0x6806, 0x7007, 0x7807, 0x8008, 0xa00a, 0xc00c, 0xe00e, 0x10020,
    0x12012, 0x14014, 0x1601b, 0x18018, 0x18018, 0x20080, 0x20080, 0x28028, 0x28028, 0x300c0,
    0x300c0, 0x38038, 0x38038, 0x38038, 0x38038, 0x58160, 0x58160, 0x58160, 0x58160, 0x78078,
    0x78078, 0x78078, 0x78078, 0x78078, 0x78078, 0x78078, 0x78078, 0xf83e0, 0xf83e0, 0xf83e0,
    0xf83e0, 0xf83e0, 0xf83e0, 0xf83e0, 0xf83e0,
];

/// Script/test access: the element-[70]-derived EG segment target table.
#[cfg(test)]
pub fn eg_target_table() -> &'static [u32; 128] {
    &EG_TARGET_TABLE
}

/// element[54]/[56]/[57] AEG rate → time.
///
/// XG rate semantics: 0 = slowest, 127 = fastest, exponential curve. The
/// S-YXG2006LE reference table `_gfAEGAttackCycle` (0x91360) maps
/// rate 0 → 699050 cycles (≈15.85s @ 44.1kHz) and rate 127 → 129 cycles
/// (≈2.92ms). The old `2000 × 2^(-v/8)` approximation was far too steep —
/// e.g. rate 58 gave 13ms release while the reference table gives ~310ms,
/// cutting short every element-driven attack/decay/release across all
/// programs (musicbox "long release" missing, weak hammer strike, etc).
fn eg_time_ms(v: u8) -> Duration {
    let cycles = 699_050f32 * (129.0f32 / 699_050.0).powf((v & 0x7F) as f32 / 127.0);
    Duration::from_secs_f32(cycles / 44_100.0)
}

/// Element volume offset (element[8], signed int8) → linear gain.
/// +0.1 dB per unit (positive values in the S-YXG50 data range 4..56, i.e.
/// +0.4..+5.6 dB; a handful of 127 ≈ +12.7 dB). Negative offsets attenuate.
/// Precomputed for all 256 signed values (index = v as u8); avoids a powf at
/// every note-on.
static VOL_OFFSET_GAIN: LazyLock<[f32; 256]> = LazyLock::new(|| {
    let mut t = [1.0f32; 256];
    for (i, e) in t.iter_mut().enumerate() {
        let v = i as i8;
        if v != 0 {
            *e = 10f32.powf(v as f32 * 0.1 / 20.0);
        }
    }
    t
});

#[inline]
fn vol_offset_gain(v: i8) -> f32 {
    VOL_OFFSET_GAIN[v as u8 as usize]
}

/// Drum note parameter snapshot (DrumSetup / DrumSetupEntry unified)
#[derive(Debug, Clone, Copy)]
struct DrumParams {
    pitch_coarse: u8,
    pitch_fine: u8,
    level: u8,
    pan: u8,
    reverb_send: u8,
    chorus_send: u8,
    variation_send: u8,
    filter_cutoff: u8,
    filter_resonance: u8,
    eg_attack: u8,
    eg_decay: u8,
    eg_release: u8,
}

/// XG_LEVEL table (dB) → linear gain (precomputed LUT; called on parameter
/// updates in the block loop)
static TG_GAIN: LazyLock<[f32; 128]> = LazyLock::new(|| {
    let mut t = [0.0f32; 128];
    for (i, &db) in XG_LEVEL.iter().enumerate() {
        t[i] = if db.is_infinite() {
            0.0
        } else {
            10f32.powf(db / 20.0)
        };
    }
    t
});

/// XG_LEVEL table (dB) → linear gain
fn xg_level_gain(v: u8) -> f32 {
    TG_GAIN[v.min(127) as usize]
}

/// 08 pp 15 Vibrato Rate (0-127) → LFO frequency (Hz)
/// Lookup via XG Spec Table #1 (0.00 - 39.7 Hz)
fn vib_to_hz(param: u8) -> f32 {
    XG_LFO_FREQ_TABLE[(param & 0x7F) as usize]
}
