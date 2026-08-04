use crate::audio::interface::Audio;
use std::sync::{Arc, RwLock};
use std::time::{self, Duration, Instant};

use crate::config::ScoringConfig;
use crate::lfo::LFO;
use crate::midi::note::Note;
use crate::midi::Part;

use super::interface::ToneGeneratorInterface;
use super::amp::Amp;
use super::eq::EQ;
use super::hpf::HPF;
use super::lpf::{CutOff, FEG, LPF};
use super::oscillator::Oscillator;
use super::pan::Pan;

/// LFO / FEG / cutoff 系数的更新周期 (samples)
const PARAM_BLOCK: u32 = 64;

#[derive(Debug, PartialEq)]
pub enum ToneGeneratorStatus {
    Idle,
    Running,
    Releasing,
}

#[derive(Debug)]
pub struct ToneGenerator {
    // update when NoteOn
    pub attack_time: time::Instant,
    // update when NoteOff/NoteOn(vel=0)
    pub release_time: time::Instant,

    pub status: ToneGeneratorStatus,
    pub scoring_config: ScoringConfig,

    pub note: Option<Note>,

    part: Option<Arc<RwLock<Part>>>,

    pub lfo: LFO,

    pub oscillator: Oscillator,
    pub lpf: LPF,
    pub hpf: HPF,
    pub amp: Amp,
    pub feg: FEG,
    pub cutoff: CutOff,
    pub eq: EQ,
    pub pan: Pan,

    /// 输出采样率 (Hz)
    sample_rate: f32,
    /// LFO 频率 (Hz, note-on 快照自 08 pp 15)
    lfo_freq: f32,
    /// LFO pitch 调制幅度 (cent, vibrato depth)
    lfo_pitch_depth: f32,
    /// LPF Q 值 (note-on 快照)
    lpf_q: f32,
    /// HPF base cutoff 参数 (0A pp 20, 每块 + LFO CM 调制)
    hpf_base: f32,
    /// HPF Q 值 (note-on 快照)
    hpf_q: f32,

    // ── 调制深度快照 (play 时自 MultiPart) ──
    /// 深度系数 (d/64, 64=标准 1.0)
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
    /// HPF 调制深度 (MultiPartExt 0A pp 22-29)
    mod_hpf_mw: f32,
    mod_hpf_bend: f32,
    mod_hpf_cat: f32,
    mod_hpf_pat: f32,
    /// AC1/AC2 控制号 (08 pp 59/60) + 深度 (08 pp 61-66)
    ac1_cc: u8,
    ac2_cc: u8,
    mod_ac1_pitch: f32,
    mod_ac1_filter: f32,
    mod_ac1_amp: f32,
    mod_ac2_pitch: f32,
    mod_ac2_filter: f32,
    mod_ac2_amp: f32,
    /// LFO 深度基数 (play 时快照, 每块 × 实时 MW)
    amod_depth_base: f32,
    fmod_depth_base: f32,
    pmod_depth_base: f32,
    /// vibrato 基深度 (cent)
    vib_pitch_base: f32,

    /// Element 输出使能 (element[44] output_en; false → 输出 0)
    output_enable: bool,

    /// 参数更新计数器
    param_counter: u32,
}

impl ToneGenerator {
    pub fn new(source_sample_rate: f32, target_sample_rate: f32, scoring: ScoringConfig) -> Self {
        Self {
            attack_time: Instant::now(),
            release_time: Instant::now(),
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
            ac1_cc: 0x11,
            ac2_cc: 0x12,
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
            param_counter: 0,
        }
    }

    pub fn bonded_to_part(&self, part: &Arc<RwLock<Part>>) -> bool {
        self.part
            .as_ref()
            .map_or(false, |p| Arc::ptr_eq(p, part))
    }

    pub fn bonded_to_channel(&self, channel: u8) -> bool {
        self.part.as_ref().map_or(false, |p| {
            p.read().map_or(false, |f| {
                f.ram.read().map_or(false, |c| c.rcv_channel == channel)
            })
        })
    }

    pub fn get_note(&self) -> Option<Note> {
        self.note
    }

    /// 立体声输出: 单声道信号链 + Pan (供 audio_render 混音)
    pub fn tick_stereo(&mut self, elapsed: time::Duration) -> (f32, f32) {
        let mono = <Self as Audio>::tick(self, elapsed);
        self.pan.apply(mono)
    }

    pub fn play(&mut self, note: Note, vel: u8, part: Arc<RwLock<Part>>) {
        self.part = Some(part);
        self.note = Some(note);
        self.oscillator.velocity = vel;
        self.oscillator.pitch.play(note);

        // 绑定采样 (S-YXG50 数据 → 2006LE 程序参数)
        if let Some(p) = self.part.as_ref().and_then(|p| p.read().ok()) {
            if let Some(program) = &p.program_entry {
                if let Some(key) = program[note as usize] {
                    // 按力度取主层采样
                    if let Some(sample) = key.samples[vel as usize][0] {
                        self.oscillator.setup(sample, note as u8, vel, self.sample_rate);
                        // LPF 参数: VCE base + Part 相对偏移
                        let mp = p.ram.read().ok();
                        let (cutoff_off, reso_off, vib_rate, vib_depth, vib_delay, feg_depth, eg_a, eg_d, eg_r, lfo_fmod) = match &mp {
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
                                m.mw.lfo_fmod_depth,                // 08 pp 21 (LFO→cutoff)
                            ),
                            None => (0.0, 0.0, 0.0, 0.0, 0, 0.0, 0x40, 0x40, 0x40, 0),
                        };

                        self.cutoff.base = sample.filter_cutoff as f32;
                        self.cutoff.part_offset = cutoff_off;
                        self.cutoff.feg_depth = CutOff::feg_depth_param(feg_depth.clamp(0.0, 127.0) as u8);
                        self.cutoff.lfo_depth = CutOff::lfo_depth_param(lfo_fmod);

                        let q = LPF::resonance_param_to_q(
                            (sample.filter_resonance as f32 + reso_off).clamp(0.0, 127.0) as u8,
                        );
                        self.lpf_q = q;
                        self.lpf.set_params(
                            self.cutoff.compute_hz(0.0, 0.0),
                            q,
                            self.sample_rate,
                        );
                        self.lpf.reset();

                        // FEG
                        self.feg.setup(eg_a, eg_d, eg_r, feg_depth.clamp(0.0, 127.0) as u8);

                        // Amp: velocity (get_velocity) + part volume + AEG 时间
                        if let Some(m) = &mp {
                            self.amp.setup(vel, m, eg_a, eg_d, eg_r);
                            self.amp.expression = 1.0;
                            // LFO AM 深度 (MW LFO AMOD 08 pp 22, 默认 0 = 无影响)
                            self.amp.lfo_depth = m.mw.lfo_amod_depth as f32 / 127.0;
                            self.lfo.amp.depth = 1.0;
                            self.lfo.amp.offset = 0.0;

                            // Pan (08 pp 0E, 0=random); 鼓 per-note pan 待鼓路径接入
                            self.pan.set(m.pan);

                            // Pitch EG (08 pp 69-6C, XG ±12 半音; 仅非默认值覆盖)
                            self.oscillator.peg.apply_xg_eg(
                                m.pitch_eg_init_level,
                                m.pitch_eg_attack_time,
                                m.pitch_eg_release_level,
                                m.pitch_eg_release_time,
                                self.sample_rate,
                            );

                            // 调制深度快照 (08 pp 1D-28, 4D-58): 深度系数 d/64, 64=标准 1.0
                            let d = |v: u8| v as f32 / 64.0;
                            self.mod_mw_pitch = d(m.mw.pitch_control);
                            self.mod_mw_filter = d(m.mw.filter_control);
                            self.mod_mw_amp = d(m.mw.amplitude_control);
                            self.mod_bend_pitch = d(m.bend.pitch_control);
                            self.mod_bend_filter = d(m.bend.filter_control);
                            self.mod_bend_amp = d(m.bend.amplitude_control);
                            self.mod_cat_pitch = d(m.cat.pitch_control);
                            self.mod_cat_filter = d(m.cat.filter_control);
                            self.mod_cat_amp = d(m.cat.amplitude_control);
                            self.mod_pat_pitch = d(m.pat.pitch_control);
                            self.mod_pat_filter = d(m.pat.filter_control);
                            self.mod_pat_amp = d(m.pat.amplitude_control);
                            // LFO 深度基数 (08 pp 20-22), 每块 × 实时 MW
                            self.pmod_depth_base = m.mw.lfo_pmod_depth as f32 / 127.0;
                            self.fmod_depth_base = self.cutoff.lfo_depth; // lfo_fmod_depth 已换算
                            self.amod_depth_base = m.mw.lfo_amod_depth as f32 / 127.0;
                            self.vib_pitch_base = self.lfo_pitch_depth; // vibrato depth 已换算

                            // AC1/AC2 (08 pp 59-66): 控制号 + 深度
                            self.ac1_cc = m.ac[0].controller_number;
                            self.ac2_cc = m.ac[1].controller_number;
                            self.mod_ac1_pitch = d(m.ac[0].pitch_control);
                            self.mod_ac1_filter = d(m.ac[0].filter_control);
                            self.mod_ac1_amp = d(m.ac[0].amplitude_control);
                            self.mod_ac2_pitch = d(m.ac[1].pitch_control);
                            self.mod_ac2_filter = d(m.ac[1].filter_control);
                            self.mod_ac2_amp = d(m.ac[1].amplitude_control);

                            // 音高: note_shift (08 pp 0D) + detune (08 pp 0F/10)
                            // + scale_tuning (0A pp 26-31) + sample detune (element[19])
                            let note_shift = (m.note_shift as i32 - 64) * 100;
                            let detune = (m.detune_msb as i32 - 64) * 100
                                + (m.detune_lsb as i32 - 64);
                            let scale = m.scale_tuning
                                [self.note.map_or(0, |n| n as u8 % 12) as usize] as f32;
                            let scale_cent = ((scale - 64.0) / 64.0 * 100.0) as i32;
                            let sample_detune = sample.detune as i32 - 64;
                            let mut pitch_extra =
                                (note_shift + detune + scale_cent + sample_detune) as f32;
                            // 采样层音高微调 (element[70] wave_pitch, 64=中性)
                            pitch_extra += (sample.wave_pitch as i32 - 64) as f32;
                            self.oscillator.pitch.note_in_cent += pitch_extra;
                            self.oscillator.portamento.target_note += pitch_extra;

                            // DSP 使能位 (element[40..44], [71]):
                            // eg_filt_en / eg_amp_en / lfo_en / eg_pitch_en / output_en / eg_enable
                            let eg_total = sample.eg_enable != 0;
                            self.feg.enabled = eg_total && sample.eg_filt_en != 0;
                            self.amp.aeg.enabled = eg_total && sample.eg_amp_en != 0;
                            self.oscillator.peg.enabled = eg_total && sample.eg_pitch_en != 0;
                            self.lfo.enable = sample.lfo_en != 0;
                            self.output_enable = sample.output_en != 0;

                            // AEG 速率覆写 (element[54]/[56]/[57], 非0 生效)
                            // aeg_d2 = Decay2 → AEG 无第二衰减段, 近似映射到 sustain 电平
                            if sample.aeg_d1 != 0 {
                                self.amp.aeg.decay_time = eg_time_ms(sample.aeg_d1);
                            }
                            if sample.aeg_d2 != 0 {
                                self.amp.aeg.sustain_level = 1.0 - sample.aeg_d2 as f32 / 127.0;
                            }
                            if sample.aeg_rel != 0 {
                                self.amp.aeg.release_time = eg_time_ms(sample.aeg_rel);
                            }

                            // EQ (08 pp 72-7F; MID 段 Spec NOT USED, 此处仍实现)
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
                        // 2006LE: HPF 与 LPF 共享 LFO CM 调制 (CLFOUnit::SetupParameter)
                        if let Ok(mex) = p.ram_ext.read() {
                            self.hpf_base = mex.hpf_cutoff_freq as f32;
                            self.hpf_q = HPF::resonance_param_to_q(mex.hpf_resonance);
                            self.hpf.set_params(
                                HPF::cutoff_param_to_hz(mex.hpf_cutoff_freq),
                                self.hpf_q,
                                self.sample_rate,
                            );
                            self.hpf.reset();

                            // HPF 调制深度 (0A pp 22-29)
                            let d = |v: u8| v as f32 / 64.0;
                            self.mod_hpf_mw = d(mex.mw_hpf_control_depth);
                            self.mod_hpf_bend = d(mex.bend_hpf_control_depth);
                            self.mod_hpf_cat = d(mex.cat_hpf_control_depth);
                            self.mod_hpf_pat = d(mex.pat_hpf_control_depth);
                        }

                        // LFO: 波形 + 频率 + 调制深度
                        if let Ok(wt) = crate::lfo::wave_type::WaveType::try_from(
                            self.oscillator.lfo_wave as u8,
                        ) {
                            self.lfo.wave_type = wt;
                        }
                        self.lfo_freq = vib_to_hz(vib_rate as u8);
                        self.lfo.enable = true;
                        // LFO 各输出保持原始波形 (深度在 CutOff/后续调制侧乘)
                        self.lfo.pitch.depth = 1.0;
                        self.lfo.lpf.depth = 1.0;
                        self.lfo.pitch.offset = 0.0;
                        self.lfo.lpf.offset = 0.0;
                        // LFO pitch 调制幅度 (vibrato depth → ±depth/127×100 cent)
                        self.lfo_pitch_depth = vib_depth / 127.0 * 100.0;
                        // LFO 起振延迟 (08 pp 17 → XG Table #2, 0-50ms)
                        self.oscillator.delay.delay_samples =
                            (crate::midi::effect_params::parameter_table::XG_MODULATION_DELAY_OFFSET_TABLE
                                [vib_delay.min(127) as usize]
                                / 1000.0
                                * self.sample_rate) as u32;
                        self.oscillator.delay.fade_samples = 0;
                        self.oscillator.delay.fade_step = 1.0;
                        self.param_counter = 0;
                    }
                }
            }
        }

        self.attack_time = Instant::now();
        self.status = ToneGeneratorStatus::Running;
    }

    pub fn scoring(&self) -> u128 {
        let args = &self.scoring_config;
        let mut score = self.attack_time.elapsed().as_millis() * args.time_weight as u128;
        // TODO: Should check AEG stage
        score = match self.status {
            ToneGeneratorStatus::Running => score * args.protect_attack as u128 / 1000,
            ToneGeneratorStatus::Releasing => score * args.penalty_release as u128 / 1000,
            _ => score,
        };

        // if sustain(CC#64) hold
        self.part
            .as_ref()
            .map_or(false, |p| p.read().map_or(false, |f| f.controller.sustain))
            .then(|| score = score * args.protect_sustain_pedal as u128 / 1000);

        // note protect
        score = if self.oscillator.is_drum() {
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

impl ToneGeneratorInterface for ToneGenerator {
    fn reset(&mut self) {}

    fn kill(&mut self) {
        self.status = ToneGeneratorStatus::Idle;
        self.part = None;
        self.note = None;
    }

    fn release(&mut self) {
        if self.status == ToneGeneratorStatus::Running {
            self.release_time = Instant::now();
            self.status = ToneGeneratorStatus::Releasing;
        }
    }
}

impl Audio for ToneGenerator {
    fn tick(&mut self, elapsed: time::Duration) -> f32 {
        match self.status {
            ToneGeneratorStatus::Idle => 0.0,
            ToneGeneratorStatus::Running | ToneGeneratorStatus::Releasing => {
                // 每 PARAM_BLOCK 个采样更新一次 LFO / FEG / cutoff / Amp 参数
                self.param_counter += 1;
                if self.param_counter >= PARAM_BLOCK {
                    self.param_counter = 0;
                    let block_elapsed =
                        Duration::from_secs_f32(PARAM_BLOCK as f32 / self.sample_rate);

                    // LFO 推进 → 波形输出
                    self.lfo.update_accumulator(
                        self.lfo_freq,
                        PARAM_BLOCK,
                        self.sample_rate as u32,
                    );
                    self.lfo.make_wave();

                    // FEG 推进
                    let feg_level = self.feg.tick(block_elapsed);

                    // CutOff = base + part 偏移 + FEG×深度 + LFO×深度
                    let hz = self
                        .cutoff
                        .compute_hz(feg_level, self.lfo.lpf.output);
                    self.lpf.set_params(hz, self.lpf_q, self.sample_rate);

                    // HPF: 与 LPF 共享 LFO CM 调制 (2006LE CLFOUnit)
                    let hpf_param =
                        self.hpf_base + self.lfo.lpf.output * self.cutoff.lfo_depth;
                    self.hpf.set_params(
                        HPF::cutoff_param_to_hz(hpf_param.round().clamp(0.0, 127.0) as u8),
                        self.hpf_q,
                        self.sample_rate,
                    );

                    // LFO pitch 调制 → oscillator delay 输入
                    let lfo_pitch = self.lfo.pitch.output * self.lfo_pitch_depth;
                    self.oscillator.set_lfo(lfo_pitch);

                    // Amp: Expression (CC#11) 每块更新 + 实时调制源 (MW/Bend/CAT/PAT)
                    if let Some(p) = self.part.as_ref().and_then(|p| p.read().ok()) {
                        self.amp.update(p.controller.expression);

                        // 实时调制值 (归一化)
                        let mw = p.controller.modulation as f32 / 127.0;
                        let bend_cent = p.get_pitchbend();
                        let bend_norm = (p.pitchbend as f32 - 8192.0) / 8192.0;
                        let cat = p.cat_value as f32 / 127.0;
                        let pat = p
                            .pat_values
                            .get(self.note.map_or(0, |n| n as u8) as usize)
                            .map_or(0.0, |&v| v as f32 / 127.0);

                        // LFO 深度受 MW 控制 (08 pp 20-22)
                        self.amp.lfo_depth = self.amod_depth_base * mw;
                        self.cutoff.lfo_depth = self.fmod_depth_base * mw;
                        self.lfo_pitch_depth =
                            self.vib_pitch_base + self.pmod_depth_base * mw * 100.0;

                        // 直接调制: pitch (cent) / filter (param) / amp (dB)
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
                        self.amp.mod_gain_db = a_mw + a_bend + a_cat + a_pat;

                        // HPF 调制 (0A pp 22-29)
                        self.hpf.mod_offset = mw * self.mod_hpf_mw * 24.0
                            + bend_norm * self.mod_hpf_bend * 24.0
                            + cat * self.mod_hpf_cat * 24.0
                            + pat * self.mod_hpf_pat * 24.0;

                        // AC1/AC2 (08 pp 59-66): 控制号 → 实时 CC 值
                        let ac1 = p.controller.cc_values[self.ac1_cc as usize] as f32 / 127.0;
                        let ac2 = p.controller.cc_values[self.ac2_cc as usize] as f32 / 127.0;
                        self.oscillator.pitch_mod += ac1 * self.mod_ac1_pitch * 100.0
                            + ac2 * self.mod_ac2_pitch * 100.0;
                        self.cutoff.mod_offset += ac1 * self.mod_ac1_filter * 24.0
                            + ac2 * self.mod_ac2_filter * 24.0;
                        self.amp.mod_gain_db += ac1 * self.mod_ac1_amp * 24.0
                            + ac2 * self.mod_ac2_amp * 24.0;
                    }
                }

                // Oscillator → LPF → HPF → Amp → EQ → (Pan 在 tick_stereo)
                let osc_out = self.oscillator.tick(elapsed);
                let lpf_out = self.lpf.tick(osc_out);
                let hpf_out = self.hpf.tick(lpf_out);
                let amp_out = self.amp.tick(
                    hpf_out,
                    Duration::from_secs_f32(1.0 / self.sample_rate),
                    self.lfo.amp.output,
                );
                if !self.output_enable {
                    return 0.0;
                }
                self.eq.tick(amp_out)
            }
        }
    }
}

/// element[54]/[56]/[57] AEG 速率 → 时间 (指数近似, 值越大越快)
fn eg_time_ms(v: u8) -> std::time::Duration {
    let ms = 2000.0 * 2f32.powf(-(v as f32) / 8.0);
    std::time::Duration::from_secs_f32(ms / 1000.0)
}

/// 08 pp 15 Vibrato Rate (0-127) → LFO 频率 (Hz)
/// XG Spec Table #1 (0.00 - 39.7 Hz) 查表
fn vib_to_hz(param: u8) -> f32 {
    crate::midi::effect_params::parameter_table::XG_LFO_FREQ_TABLE[(param & 0x7F) as usize]
}
