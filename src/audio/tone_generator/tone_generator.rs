use crate::audio::interface::Audio;
use std::sync::Arc;
use std::time::{self, Duration, Instant};

use crate::config::ScoringConfig;
use crate::lfo::LFO;
use crate::midi::note::Note;
use crate::double_buffer::DoubleBuffered;
use crate::midi::PitchGetter;
use crate::midi::Part;

use super::interface::ToneGeneratorInterface;
use super::amp::Amp;
use super::eq::EQ;
use super::hpf::HPF;
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

#[derive(Debug)]
pub struct ToneGenerator {
    // update when NoteOn
    pub attack_time: time::Instant,
    // update when NoteOff/NoteOn(vel=0)
    pub release_time: time::Instant,

    pub status: ToneGeneratorStatus,
    pub scoring_config: ScoringConfig,

    pub note: Option<Note>,

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
        }
    }

    pub fn bonded_to_part(&self, part: &Arc<DoubleBuffered<Part>>) -> bool {
        self.part
            .as_ref()
            .map_or(false, |p| Arc::ptr_eq(p, part))
    }

    pub fn bonded_to_channel(&self, channel: u8) -> bool {
        self.part
            .as_ref()
            .map_or(false, |p| p.snapshot().ram.snapshot().rcv_channel == channel)
    }

    pub fn get_note(&self) -> Option<Note> {
        self.note
    }

    /// Stereo output: mono signal chain + Pan (for audio_render mixing)
    pub fn tick_stereo(&mut self, elapsed: time::Duration) -> (f32, f32) {
        let mono = <Self as Audio>::tick(self, elapsed);
        self.pan.apply(mono)
    }

    pub fn play(
        &mut self,
        note: Note,
        vel: u8,
        part: Arc<DoubleBuffered<Part>>,
        element_index: usize,
        drum_setup: Option<Arc<DoubleBuffered<[crate::midi::ram::xg::drum_setup_wrapper::DrumSetupWrapper; 16]>>>,
    ) {
        self.part = Some(part.clone());
        self.part_id = part.snapshot().id;
        self.note = Some(note);
        self.oscillator.bind_part(part.clone());
        // LFO Key sync: reset the phase on note-on (Key running mode)
        if matches!(self.lfo.runing_mode, crate::lfo::lfo::LFORunningMode::Key) {
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
                        self.oscillator.setup(sample, note as u8, vel, self.sample_rate);
                        // LPF parameters: VCE base + Part relative offset
                        let mp = Some(p.ram.snapshot());
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

                        self.cutoff.base = self
                            .drum_params
                            .map_or(sample.filter_cutoff as f32, |d| d.filter_cutoff as f32);
                        self.cutoff.part_offset = cutoff_off;
                        self.cutoff.feg_depth = CutOff::feg_depth_param(feg_depth.clamp(0.0, 127.0) as u8);
                        self.cutoff.lfo_depth = CutOff::lfo_depth_param(lfo_fmod);

                        let reso_base = self
                            .drum_params
                            .map_or(sample.filter_resonance as f32, |d| d.filter_resonance as f32);
                        let q = LPF::resonance_param_to_q(
                            (reso_base + reso_off).clamp(0.0, 127.0) as u8,
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

                        // Amp: velocity (get_velocity) + part volume + AEG times
                        if let Some(m) = &mp {
                            let (a, d, r) = self.drum_params.map_or(
                                (eg_a, eg_d, eg_r),
                                |ds| (ds.eg_attack, ds.eg_decay, ds.eg_release),
                            );
                            self.amp.setup(vel, m, a, d, r);
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

                            // Modulation depth snapshot (08 pp 1D-28, 4D-58): depth coefficient d/64, 64 = standard 1.0
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
                            // LFO depth base (08 pp 20-22), × real-time MW each block
                            self.pmod_depth_base = m.mw.lfo_pmod_depth as f32 / 127.0;
                            self.fmod_depth_base = self.cutoff.lfo_depth; // lfo_fmod_depth already converted
                            self.amod_depth_base = m.mw.lfo_amod_depth as f32 / 127.0;
                            self.vib_pitch_base = self.lfo_pitch_depth; // vibrato depth already converted

                            // AC1/AC2 (08 pp 59-66): control number + depth
                            self.ac1_cc = m.ac[0].controller_number;
                            self.ac2_cc = m.ac[1].controller_number;
                            self.mod_ac1_pitch = d(m.ac[0].pitch_control);
                            self.mod_ac1_filter = d(m.ac[0].filter_control);
                            self.mod_ac1_amp = d(m.ac[0].amplitude_control);
                            self.mod_ac2_pitch = d(m.ac[1].pitch_control);
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
                            // Sample level: element[19] detune + element[70] wave_pitch (64 = neutral)
                            pitch_extra +=
                                (sample.detune as i32 - 64) + (sample.wave_pitch as i32 - 64);
                            // Drum note: pitch_coarse (0x40 = center, ±64 semitones) + pitch_fine (cents)
                            if let Some(d) = self.drum_params {
                                pitch_extra += (d.pitch_coarse as i32 - 64) * 100
                                    + (d.pitch_fine as i32 - 64);
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

                            // AEG rate overrides (element[54]/[56]/[57], active when non-zero)
                            // aeg_d2 = Decay2 → AEG has no second decay stage, approximately mapped to the sustain level
                            if sample.aeg_d1 != 0 {
                                self.amp.aeg.decay_time = eg_time_ms(sample.aeg_d1);
                            }
                            if sample.aeg_d2 != 0 {
                                self.amp.aeg.sustain_level = 1.0 - sample.aeg_d2 as f32 / 127.0;
                            }
                            if sample.aeg_rel != 0 {
                                self.amp.aeg.release_time = eg_time_ms(sample.aeg_rel);
                            }

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

                            // HPF modulation depth (0A pp 22-29)
                            let d = |v: u8| v as f32 / 64.0;
                            self.mod_hpf_mw = d(mex.mw_hpf_control_depth);
                            self.mod_hpf_bend = d(mex.bend_hpf_control_depth);
                            self.mod_hpf_cat = d(mex.cat_hpf_control_depth);
                            self.mod_hpf_pat = d(mex.pat_hpf_control_depth);

                            // CBC1/CBC2 (0A pp 25-36): control number + depth
                            self.cbc1_cc = mex.cbc1_control_number;
                            self.cbc2_cc = mex.cbc2_control_number;
                            self.mod_cbc1_pitch = d(mex.cbc1_pitch_control);
                            self.mod_cbc1_filter = d(mex.cbc1_lpf_control);
                            self.mod_cbc1_amp = d(mex.cbc1_amplitude_control);
                            self.mod_cbc2_pitch = d(mex.cbc2_pitch_control);
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
                        if let Ok(wt) = crate::lfo::wave_type::WaveType::try_from(
                            self.oscillator.lfo_wave as u8,
                        ) {
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
        // AEG stage: protect voices still in attack (most recently struck)
        if self.amp.aeg.state == crate::audio::tone_generator::amp::aeg::AEGStage::Attack {
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
            .map_or(false, |p| p.snapshot().controller.sustain)
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
                // Update LFO / FEG / cutoff / Amp parameters every PARAM_BLOCK samples
                self.param_counter += 1;
                if self.param_counter >= PARAM_BLOCK {
                    self.param_counter = 0;
                    let block_elapsed =
                        Duration::from_secs_f32(PARAM_BLOCK as f32 / self.sample_rate);

                    // LFO advance → waveform output
                    self.lfo.update_accumulator(
                        self.lfo_freq,
                        PARAM_BLOCK,
                        self.sample_rate as u32,
                    );
                    self.lfo.make_wave();

                    // FEG advance
                    let feg_level = self.feg.tick(block_elapsed);

                    // CutOff = base + part offset + FEG×depth + LFO×depth
                    let hz = self
                        .cutoff
                        .compute_hz(feg_level, self.lfo.lpf.output);
                    self.lpf.set_params(hz, self.lpf_q, self.sample_rate);

                    // HPF: shares LFO CM modulation with LPF (2006LE CLFOUnit)
                    let hpf_param =
                        self.hpf_base + self.lfo.lpf.output * self.cutoff.lfo_depth;
                    self.hpf.set_params(
                        HPF::cutoff_param_to_hz(hpf_param.round().clamp(0.0, 127.0) as u8),
                        self.hpf_q,
                        self.sample_rate,
                    );

                    // LFO pitch modulation → oscillator delay input
                    let lfo_pitch = self.lfo.pitch.output * self.lfo_pitch_depth;
                    self.oscillator.set_lfo(lfo_pitch);

                    // Amp: Expression (CC#11) updated each block + real-time modulation sources (MW/Bend/CAT/PAT)
                    if let Some(p) = self.part.as_ref().map(|p| p.snapshot()) {
                        self.amp.update(p.controller.expression);

                        // Effect send levels (08 pp 2B-2F → XG_LEVEL linear gain)
                        // Drum note: DrumSetup sends override part sends (XG Spec: drum per-note sends)
                        let r = p.ram.snapshot();
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

                        // Real-time modulation values (normalized)
                        let mw = p.controller.modulation as f32 / 127.0;
                        let bend_cent = p.get_pitchbend();
                        let bend_norm = (p.pitchbend as f32 - 8192.0) / 8192.0;
                        let cat = p.cat_value as f32 / 127.0;
                        let pat = p
                            .pat_values
                            .get(self.note.map_or(0, |n| n as u8) as usize)
                            .map_or(0.0, |&v| v as f32 / 127.0);

                        // LFO depth controlled by MW (08 pp 20-22)
                        self.amp.lfo_depth = self.amod_depth_base * mw;
                        self.cutoff.lfo_depth = self.fmod_depth_base * mw;
                        self.lfo_pitch_depth =
                            self.vib_pitch_base + self.pmod_depth_base * mw * 100.0;

                        // Direct modulation: pitch (cents) / filter (param) / amp (dB)
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

                        // HPF modulation (0A pp 22-29)
                        self.hpf.mod_offset = mw * self.mod_hpf_mw * 24.0
                            + bend_norm * self.mod_hpf_bend * 24.0
                            + cat * self.mod_hpf_cat * 24.0
                            + pat * self.mod_hpf_pat * 24.0;

                        // AC1/AC2 (08 pp 59-66): control number → real-time CC value
                        let ac1 = p.controller.cc_values[self.ac1_cc as usize] as f32 / 127.0;
                        let ac2 = p.controller.cc_values[self.ac2_cc as usize] as f32 / 127.0;
                        self.oscillator.pitch_mod += ac1 * self.mod_ac1_pitch * 100.0
                            + ac2 * self.mod_ac2_pitch * 100.0;
                        self.cutoff.mod_offset += ac1 * self.mod_ac1_filter * 24.0
                            + ac2 * self.mod_ac2_filter * 24.0;
                        self.amp.mod_gain_db += ac1 * self.mod_ac1_amp * 24.0
                            + ac2 * self.mod_ac2_amp * 24.0;

                        // CBC1/CBC2 (0A pp 25-36): control number → real-time CC value
                        let cbc1 = p.controller.cc_values[self.cbc1_cc as usize] as f32 / 127.0;
                        let cbc2 = p.controller.cc_values[self.cbc2_cc as usize] as f32 / 127.0;
                        self.oscillator.pitch_mod += cbc1 * self.mod_cbc1_pitch * 100.0
                            + cbc2 * self.mod_cbc2_pitch * 100.0;
                        self.cutoff.mod_offset += cbc1 * self.mod_cbc1_filter * 24.0
                            + cbc2 * self.mod_cbc2_filter * 24.0;
                        self.amp.mod_gain_db += cbc1 * self.mod_cbc1_amp * 24.0
                            + cbc2 * self.mod_cbc2_amp * 24.0;
                        // CBC LFO depth (pmod/fmod/amod)
                        self.lfo_pitch_depth +=
                            cbc1 * self.cbc1_pmod * 100.0 + cbc2 * self.cbc2_pmod * 100.0;
                        self.cutoff.lfo_depth +=
                            cbc1 * self.cbc1_fmod * 40.0 + cbc2 * self.cbc2_fmod * 40.0;
                        self.amp.lfo_depth +=
                            cbc1 * self.cbc1_amod + cbc2 * self.cbc2_amod;

                        // offset level (0A pp 3F-44): modulation source → level offset (±24dB)
                        self.amp.mod_gain_db += (mw - 0.5) * self.mod_mw_level * 24.0
                            + (bend_norm - 0.0) * self.mod_bend_level * 24.0
                            + (cat - 0.5) * self.mod_cat_level * 24.0
                            + (pat - 0.5) * self.mod_pat_level * 24.0
                            + (ac1 - 0.5) * self.mod_ac1_level * 24.0
                            + (ac2 - 0.5) * self.mod_ac2_level * 24.0;
                    }
                }

                // Oscillator → LPF → HPF → Amp → EQ → (Pan in tick_stereo)
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

/// element[54]/[56]/[57] AEG rate → time (exponential approximation, larger value = faster)
fn eg_time_ms(v: u8) -> std::time::Duration {
    let ms = 2000.0 * 2f32.powf(-(v as f32) / 8.0);
    std::time::Duration::from_secs_f32(ms / 1000.0)
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

/// XG_LEVEL table (dB) → linear gain
fn xg_level_gain(v: u8) -> f32 {
    let db = crate::midi::effect_params::parameter_table::XG_LEVEL[v.min(127) as usize];
    if db.is_infinite() {
        0.0
    } else {
        10f32.powf(db / 20.0)
    }
}

/// 08 pp 15 Vibrato Rate (0-127) → LFO frequency (Hz)
/// Lookup via XG Spec Table #1 (0.00 - 39.7 Hz)
fn vib_to_hz(param: u8) -> f32 {
    crate::midi::effect_params::parameter_table::XG_LFO_FREQ_TABLE[(param & 0x7F) as usize]
}
