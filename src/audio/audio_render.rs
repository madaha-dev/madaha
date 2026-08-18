use std::f32::consts::FRAC_PI_4;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use wd_log::log_debug_ln;

use super::dsp::{build_chorus, build_reverb, build_variation};
use crate::audio::tone_generator::ToneGeneratorStatus;
use crate::double_buffer::DoubleBuffered;
use crate::fast_sine::{fast_cos, fast_sin};
use crate::midi::Part;
use crate::midi::consts::DEFAULT_MASTER_VOLUME;
use crate::midi::effect_params::interface::EffectType;
use crate::midi::effect_params::parameter_table::XG_LEVEL;
use crate::midi::effect_params::variation_type::XGVariationType;
use crate::midi::note::Note;
use crate::midi::ram::xg::effect_insertion::EffectInsertion;
use crate::midi::ram::xg::effects::{Chorus, EffectData, Reverb, Variation};
use crate::midi::ram::xg::multi_eq::MultiEQ;

use super::AudioRender;
use super::AudioRenderActions;
use super::tone_generator::ToneGeneratorStatus::{Idle, Running};
use super::tone_generator::interface::ToneGeneratorInterface;

/// XG_LEVEL table (dB) → linear gain (precomputed; powf per call was ~17us on
/// this machine — 8+ calls per frame made the render loop ~8x slower than real-time)
static XG_GAIN: LazyLock<[f32; 128]> = LazyLock::new(|| {
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
#[inline]
fn xg_level_gain(v: u8) -> f32 {
    XG_GAIN[v.min(127) as usize]
}

/// pan parameter (64=center) → equal-power left/right gains
/// Precomputed: the per-call fast_cos/fast_sin path measured ~42us/call on this
/// machine — 3 pan_gain calls per frame made the render loop ~8x slower than RT.
static PAN_GAIN: LazyLock<[(f32, f32); 128]> = LazyLock::new(|| {
    let mut t = [(0.0f32, 0.0f32); 128];
    for (i, e) in t.iter_mut().enumerate() {
        let tv = (i as f32 - 64.0) / 64.0; // -1..1
        let theta = (tv + 1.0) * FRAC_PI_4;
        *e = (fast_cos(theta), fast_sin(theta));
    }
    t
});

/// pan parameter (64=center) → equal-power left/right gains
#[inline]
fn pan_gain(v: u8) -> (f32, f32) {
    PAN_GAIN[v.min(127) as usize]
}

/// Reverb → [u16;16] parameter array
fn reverb_params(r: &Reverb) -> [u16; 16] {
    [
        r.param1, r.param2, r.param3, r.param4, r.param5, r.param6, r.param7, r.param8, r.param9,
        r.param10, r.param11, r.param12, r.param13, r.param14, r.param15, r.param16,
    ]
    .map(|v| v as u16)
}

/// Chorus → [u16;16] parameter array
fn chorus_params(c: &Chorus) -> [u16; 16] {
    [
        c.param1, c.param2, c.param3, c.param4, c.param5, c.param6, c.param7, c.param8, c.param9,
        c.param10, c.param11, c.param12, c.param13, c.param14, c.param15, c.param16,
    ]
    .map(|v| v as u16)
}

/// EffectInsertion → [u16;16] (first 10 parameters, last 6 slots left empty)
fn insertion_params(i: &EffectInsertion) -> [u16; 16] {
    let mut p = [0u16; 16];
    p[0] = i.ins_effect_param1 as u16;
    p[1] = i.ins_effect_param2 as u16;
    p[2] = i.ins_effect_param3 as u16;
    p[3] = i.ins_effect_param4 as u16;
    p[4] = i.ins_effect_param5 as u16;
    p[5] = i.ins_effect_param6 as u16;
    p[6] = i.ins_effect_param7 as u16;
    p[7] = i.ins_effect_param8 as u16;
    p[8] = i.ins_effect_param9 as u16;
    p[9] = i.ins_effect_param10 as u16;
    p
}

/// Variation → [u16;16] (14-bit parameters take the high 7 bits)
fn variation_params(v: &Variation) -> [u16; 16] {
    [
        v.param1_msb,
        v.param2_msb,
        v.param3_msb,
        v.param4_msb,
        v.param5_msb,
        v.param6_msb,
        v.param7_msb,
        v.param8_msb,
        v.param9_msb,
        v.param10_msb,
        v.param11,
        v.param12,
        v.param13,
        v.param14,
        v.param15,
        v.param16,
    ]
    .map(|x| x as u16)
}

impl AudioRender {
    /// Retarget the render clock to the sink's actual negotiated rate (ALSA
    /// hw_params may return a different rate than configured; the render
    /// virtual clock must follow it or pitch shifts by rate/actual_rate).
    pub fn set_output_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        for tg in self.tone_generators.iter_mut() {
            tg.set_output_rate(rate);
        }
    }

    /// Audio render loop: drain events + render one frame to sink
    pub fn audio_render(&mut self) {
        // Drain all events in channel.
        loop {
            if !self.drain_event() {
                break;
            }
        }
        self.render_frame();
    }

    /// Render one frame (L/R → sink)
    ///
    /// XG Spec signal flow:
    ///   part → dry / chorus_send / reverb_send / variation_send
    ///   (when an insertion effect exists, only the dry path is used, no sends)
    ///   master bus → master_volume → MultiEQ → Master Attenuator → sink
    fn render_frame(&mut self) {
        let elapsed = Duration::from_secs_f32(1.0 / self.sample_rate);
        // Watchdog: accumulate silence while nothing is sounding (drives the
        // idle-sleep decision in needs_render).
        let any_active = self.tone_generators.iter().any(|t| t.status != Idle);
        self.tick_idle_watchdog(any_active);
        let mut dry = [0.0f32; 2];

        // Collect active notes (pitch classes) for harmony-family effects
        let mut active: Vec<u8> = Vec::with_capacity(16);
        for tg in self.tone_generators.iter() {
            if let Some(n) = tg.note {
                let pc = n as u8 % 12;
                if !active.contains(&pc) {
                    active.push(pc);
                }
            }
        }
        active.sort_unstable();
        self.variation.set_active_notes(&active);
        for fx in self.insertion_instances.values_mut() {
            fx.set_active_notes(&active);
        }
        let mut ch_in = [0.0f32; 2];
        let mut rev_in = [0.0f32; 2];
        let mut var_in = [0.0f32; 2];

        // Before the Init event arrives (shared=None): effects bypass + no volume control
        let shared = self.shared.clone();

        // Variation insertion mode target part (connection=0)
        let var_insert_part: Option<usize> = shared.as_ref().and_then(|s| {
            let fx = s.effect1.snapshot();
            (fx.variation.connection == 0).then_some(fx.variation.part as usize)
        });

        // Active insertion effect numbers (union across all voices, used for parameter updates)
        let mut active_ins: Vec<u8> = Vec::new();
        for tg in self.tone_generators.iter_mut() {
            if tg.status == Idle {
                continue;
            }

            let (l, r) = tg.tick_stereo(elapsed);
            // Variation insertion mode: voices of the specified part go through variation (no sends)
            if var_insert_part == Some(tg.part_id) && tg.insertion_effects.is_empty() {
                let out = self.variation.process((l, r));
                dry[0] += out.0;
                dry[1] += out.1;
                continue;
            }
            if tg.insertion_effects.is_empty() {
                // No insertion effect: 4-way sends
                dry[0] += l * tg.dry_level;
                dry[1] += r * tg.dry_level;
                ch_in[0] += l * tg.chorus_send;
                ch_in[1] += r * tg.chorus_send;
                rev_in[0] += l * tg.reverb_send;
                rev_in[1] += r * tg.reverb_send;
                var_in[0] += l * tg.variation_send;
                var_in[1] += r * tg.variation_send;
            } else {
                // With insertion effect: chained processing (2006LE Mac: no sends), output goes only through the dry path
                for &nn in &tg.insertion_effects {
                    if !active_ins.contains(&nn) {
                        active_ins.push(nn);
                    }
                }
                let mut out = (l, r);
                for &nn in &tg.insertion_effects {
                    if let Some(fx) = self.insertion_instances.get_mut(&nn) {
                        out = fx.process(out);
                    }
                }
                dry[0] += out.0;
                dry[1] += out.1;
            }
        }

        // Insertion effect parameter update (only active nn)
        if let Some(shared) = &shared {
            let ins_arr = shared.effect_instertion.snapshot();
            for &nn in &active_ins {
                self.update_insertion(nn, &ins_arr[nn as usize]);
            }

            // Insertion effect real-time modulation (control depths, 64=neutral)
            let sources = self.read_mod_sources();
            for &nn in &active_ins {
                let ins = &ins_arr[nn as usize];
                let depths = [
                    ins.mw_ins_control_depth,
                    ins.bend_ins_control_depth,
                    ins.cat_ins_control_depth,
                    0x40, // no PAT depth for insertion effects
                    ins.ac1_ins_control_depth,
                    ins.ac2_ins_control_depth,
                    ins.cbc1_ins_control_depth,
                    ins.cbc2_ins_control_depth,
                ];
                if let Some(fx) = self.insertion_instances.get_mut(&nn) {
                    for (i, &d) in depths.iter().enumerate() {
                        if d != 64 {
                            fx.modulate(i as u8, sources[i] * (d as f32 - 64.0) / 64.0);
                        }
                    }
                }
            }

            // Variation real-time modulation
            let vs = shared.effect1.snapshot();
            let v = &vs.variation;
            let vdepths = [
                v.mw_variation_control_depth,
                v.bend_variation_control_depth,
                v.cat_variation_control_depth,
                0x40, // no PAT depth on Variation
                v.ac1_variation_control_depth,
                v.ac2_variation_control_depth,
                v.cbc1_variation_control_depth,
                v.cbc2_variation_control_depth,
            ];
            for (i, &d) in vdepths.iter().enumerate() {
                if d != 64 {
                    self.variation
                        .modulate(i as u8, sources[i] * (d as f32 - 64.0) / 64.0);
                }
            }
        }

        // Before the Init event arrives (shared=None): effects bypass + no volume control
        let Some(shared) = shared else {
            let out_l = dry[0] + ch_in[0] + rev_in[0] + var_in[0];
            let out_r = dry[1] + ch_in[1] + rev_in[1] + var_in[1];
            let (out_l, out_r) = self.master_limiter.process(out_l, out_r);
            self.sink.push_frame(out_l, out_r);
            return;
        };

        let fx = shared.effect1.snapshot();
        self.update_system_effects(&fx, &shared.multi_eq.snapshot());
        let sys = shared.system.snapshot();

        let mut out_l = dry[0];
        let mut out_r = dry[1];

        // ── System effect chain (XG Spec): Variation → Chorus → Reverb ──
        // 1. Variation (dual mode: 0=Insertion for a specified part, 1=System)
        if fx.variation.connection == 1 {
            // System mode: input = each part's variation_send
            //   serial components: send_to_chorus → Chorus, send_to_reverb → Reverb
            //   parallel components: × return × pan → master bus
            let (vl, vr) = self.variation.process((var_in[0], var_in[1]));
            let s2c = xg_level_gain(fx.variation.send_to_chorus);
            let s2r = xg_level_gain(fx.variation.send_to_reverb);
            ch_in[0] += vl * s2c;
            ch_in[1] += vr * s2c;
            rev_in[0] += vl * s2r;
            rev_in[1] += vr * s2r;
            let (gl, gr) = pan_gain(fx.variation.variation_pan);
            let ret = xg_level_gain(fx.variation.variation_return);
            out_l += vl * ret * gl;
            out_r += vr * ret * gr;
        }
        // connection=0 (Insertion): variation already handled the specified part in the voice loop, var_in ignored

        // 2. Chorus → master bus + feed back to Reverb (send_to_reverb)
        let (cl, cr) = self.chorus.process((ch_in[0], ch_in[1]));
        let s2r = xg_level_gain(fx.chorus.send_to_reverb);
        rev_in[0] += cl * s2r;
        rev_in[1] += cr * s2r;
        let (gl, gr) = pan_gain(fx.chorus.chorus_pan);
        let ret = xg_level_gain(fx.chorus.chorus_return);
        out_l += cl * ret * gl;
        out_r += cr * ret * gr;

        // 3. Reverb (input = part reverb_send + Chorus feed + Variation feed)
        let (rl, rr) = self.reverb.process((rev_in[0], rev_in[1]));
        let (gl, gr) = pan_gain(fx.reverb.reverb_pan);
        let ret = xg_level_gain(fx.reverb.reverb_return);
        out_l += rl * ret * gl;
        out_r += rr * ret * gr;

        // ── master bus → master_volume → MultiEQ → Master Attenuator ──
        // GM2/GM1 master volume (14-bit, engine level) applied on top
        let gm2_vol = *shared.master_volume.snapshot() as f32 / DEFAULT_MASTER_VOLUME as f32;
        // 响度校准（-14 LUFS 是历史 TODO 目标）：原 ×6.756 太热——HPF 修复后单音 C4 峰值
        // 1.48、密集滑音总线 ~2-3.4× → 限幅器被迫持续压，新音音头被乘 0.3
        // （「拖动滑音音头很小」）。降为 ×4.2（≈-4.1dB）：单音攻击峰值 ~0.90 天然不削波，
        // 限幅器只在真正密集的汇合处兜底，音头得以保留（更贴近母机固定增益分级行为）。
        // 代价：programme loudness ≈ -18 LUFS（比 -14 目标低，换取动态/音头）。
        const LUFS_GAIN: f32 = 4.2;
        let vol = xg_level_gain(sys.master_volume) * gm2_vol * LUFS_GAIN;
        out_l *= vol;
        out_r *= vol;
        (out_l, out_r) = self.multi_eq.process((out_l, out_r));
        let att = xg_level_gain(127 - sys.attenuator.min(127));
        out_l *= att;
        out_r *= att;

        // DC offset correction (XG Spec: serial effect chains introduce DC components)
        if self.dc_enabled {
            out_l = self.dc_l.tick(out_l);
            out_r = self.dc_r.tick(out_r);

        }

        self.dbg_frames += 1;
        if self.dbg_frames % 4410 == 1 {
            //wd_log::log_debug_ln!("DBG-OUT l={} r={} dry={}", out_l, out_r, dry[0]);
        }
        // Output-side dynamic loudness (slow AGC toward target LUFS): meter the
        // PRE-gain master bus (no feedback loop), then apply the adaptive gain
        // before the peak limiter, which still owns instantaneous protection.
        if self.loudness_norm_enabled {
            self.loudness_norm.set_target(self.loudness_target_lufs);
            self.loudness_norm.push_frame(out_l, out_r);
            let m = self.loudness_norm.gain();
            out_l *= m;
            out_r *= m;
        }
        // Master peak limiter: keep hot transients (restored low energy) from
        // driving the output bus over ±1.0 (was hard-clipping at the DAC).
        let (out_l, out_r) = self.master_limiter.process(out_l, out_r);
        self.sink.push_frame(out_l, out_r);
    }

    /// Insertion effect update (type/parameter change detection, only active nn)
    fn update_insertion(
        &mut self,
        nn: u8,
        ins: &EffectInsertion,
    ) {
        let key = (
            ins.ins_effect_type_msb,
            ins.ins_effect_type_lsb,
            insertion_params(ins),
        );
        if self.insertion_key.get(&nn) == Some(&key) {
            return;
        }
        let vtype = XGVariationType::get_type(ins.ins_effect_type_msb, ins.ins_effect_type_lsb);
        self.insertion_instances
            .insert(nn, build_variation(vtype, &key.2, self.sample_rate));
        self.insertion_key.insert(nn, key);
    }

    /// Read the 8 modulation source values from the first active voice's part
    /// (0=MW, 1=Bend, 2=CAT, 3=PAT, 4=AC1, 5=AC2, 6=CBC1, 7=CBC2)
    fn read_mod_sources(&self) -> [f32; 8] {
        for tg in self.tone_generators.iter() {
            if tg.status == Idle {
                continue;
            }
            let Some(p) = tg.part.as_ref().map(|p| p.snapshot()) else {
                continue;
            };
            let c = &p.controller;
            let note = tg.get_note().map_or(0, |n| n as u8 as usize);
            return [
                c.modulation as f32 / 127.0,
                (p.pitchbend as f32 - 8192.0) / 8192.0,
                p.cat_value as f32 / 127.0,
                p.pat_values.get(note).map_or(0.0, |&v| v as f32 / 127.0),
                c.cc_values[tg.ac1_cc as usize] as f32 / 127.0,
                c.cc_values[tg.ac2_cc as usize] as f32 / 127.0,
                c.cc_values[tg.cbc1_cc as usize] as f32 / 127.0,
                c.cc_values[tg.cbc2_cc as usize] as f32 / 127.0,
            ];
        }
        [0.0; 8]
    }

    /// System effect parameter update (type/parameter change detection)
    fn update_system_effects(
        &mut self,
        fx: &EffectData,
        eq: &MultiEQ,
    ) {
        let r_key = (
            fx.reverb.type_msb,
            fx.reverb.type_lsb,
            reverb_params(&fx.reverb),
        );
        if r_key != self.reverb_key {
            self.reverb = build_reverb(self.sample_rate, &r_key.2);
            self.reverb_key = r_key;
        }
        let c_key = (
            fx.chorus.type_msb,
            fx.chorus.type_lsb,
            chorus_params(&fx.chorus),
        );
        if c_key != self.chorus_key {
            self.chorus = build_chorus(self.sample_rate, &c_key.2);
            self.chorus_key = c_key;
        }
        let v_key = (
            fx.variation.type_msb,
            fx.variation.type_lsb,
            variation_params(&fx.variation),
        );
        if v_key != self.variation_key {
            let vtype = XGVariationType::get_type(fx.variation.type_msb, fx.variation.type_lsb);
            self.variation = build_variation(vtype, &v_key.2, self.sample_rate);
            self.variation_key = v_key;
        }
        // MultiEQ (compare type + band values when snapshotting each frame)
        let eq_key = (eq.eq_type, eq.band1, eq.band2, eq.band3, eq.band4, eq.band5);
        if eq_key != self.multi_eq_key {
            self.multi_eq.set_from(eq, self.sample_rate);
            self.multi_eq_key = eq_key;
        }
    }

    fn drain_event(&mut self) -> bool {
        if let Ok(ev) = self.rx.try_recv() {
            self.handle_action(ev);
            true
        } else {
            false
        }
    }

    pub(crate) fn handle_action(&mut self, ev: AudioRenderActions) {
        use AudioRenderActions::*;
        match ev {
            Init { shared } => {
                self.shared = Some(shared);
            }
            Play { note, vel, part } => {
                self.note_handler(note, vel, part);
                self.log_polyphony();
            }
            Release { note, part } => {
                self.release_handler(note, part);
            }
            ReleaseAll { part } => {
                self.release_all_handler(part);
            }
            KillAll { part } => {
                self.kill_all_handler(part);
            }
            SustainChange { part, on } => {
                self.sustain_change_handler(part, on);
            }
            SostenutoChange { part, on } => {
                self.sostenuto_change_handler(part, on);
            }
        }
    }

    fn log_polyphony(&self) {
        log_debug_ln!(
            "polyphony stats: {}/{}, including releasing={}",
            self.tone_generators
                .iter()
                .filter(|&tg| matches!(
                    tg.status,
                    ToneGeneratorStatus::Running | ToneGeneratorStatus::Releasing
                ))
                .count(),
            self.max_polyphony,
            self.tone_generators
                .iter()
                .filter(|&tg| tg.status == ToneGeneratorStatus::Releasing)
                .count()
        );
    }

    fn note_handler(
        &mut self,
        note: Note,
        vel: u8,
        part: Arc<DoubleBuffered<Part>>,
    ) {
        // Dual element: check each element's velocity range against the current velocity, allocate one voice per hit
        let element_count = part
            .snapshot()
            .program_entry
            .as_ref()
            .and_then(|program| program[note as usize].as_ref())
            .map_or(1, |key| key.element_count(vel));

        // Single-assign mode (key_assign=0): re-triggering the same note in the
        // same part replaces the running voice instead of stacking another one.
        // The old voice terminates immediately (XG mono semantics — a long
        // element release would otherwise overlap the new note's attack).
        if part.snapshot().ram.snapshot().key_assign == 0 {
            for tg in self.tone_generators.iter_mut() {
                if tg.bonded_to_part(&part) && tg.get_note() == Some(note) && tg.status == Running {
                    tg.kill();
                }
            }
        }

        // Drum alternate group voice stealing: kill Running voices in the same part and group first
        if let Some(shared) = &self.shared {
            let snap = part.snapshot();
            if let (Some(prog), Some(key)) = (
                &snap.program_entry,
                snap.program_entry
                    .as_ref()
                    .and_then(|p| p[note as usize].as_ref()),
            ) {
                let _ = prog;
                if key.drum_setup.is_some() {
                    let part_mode = snap.ram.snapshot().part_mode as usize;
                    let setup_idx = part_mode.saturating_sub(2).min(15);
                    let note_idx = (note as u8 as usize).saturating_sub(12).min(78);
                    let group = shared.drum_setup.snapshot()[setup_idx][note_idx].alternate_group;
                    if group != 0 {
                        for tg in self.tone_generators.iter_mut() {
                            if tg.bonded_to_part(&part)
                                && tg.drum_group == group
                                && tg.status == Running
                            {
                                tg.kill();
                            }
                        }
                    }
                }
            }
        }

        // One NoteOn → one group id shared by all element voices (dual-element
        // voices release together on NoteOff).
        let note_on_id = self.note_on_counter;
        self.note_on_counter = self.note_on_counter.wrapping_add(1);
        let mut last_alloc: Option<usize> = None;

        for element_index in 0..element_count {
            // 分配：优先取空闲 voice（冗余池——pool = count × poly_replicant，
            // 保证分配不因复音限制阻塞）；仅当整个池满（active ≥ len）才兜底
            // 评分释放腾位。
            let active_count = self
                .tone_generators
                .iter()
                .filter(|t| t.status != Idle)
                .count();
            let free: Option<usize> = if active_count < self.tone_generators.len() {
                // Associative placement: the first element voice is found
                // sequentially; dual-element companions land right next to
                // their partner (adjacent slots, predictable order).
                match last_alloc {
                    Some(prev) => self.find_adjacent_idle(prev),
                    None => self.find_idle_voice(),
                }
            } else {
                None
            };
            let index = match free {
                Some(i) => i,
                None => {
                    // 池满兜底：highest score gets killed (only among NON-Idle
                    // voices — an idle voice scores 0 and would always win a
                    // plain max_by_key, defeating the pool's purpose).
                    // scoring weights: low score = protected (new note×0.1 / sustained×0.1 / drum×0.05),
                    // high score = preferred for killing (Releasing×1.5, older notes accumulate time_weight)
                    let i = self
                        .tone_generators
                        .iter()
                        .enumerate()
                        .filter(|(_, t)| t.status != Idle)
                        .max_by_key(|(_, t)| t.scoring())
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.tone_generators[i].kill();
                    i
                }
            };

            let drum_setup = self.shared.as_ref().map(|s| s.drum_setup.clone());
            self.tone_generators[index].play(
                note,
                vel,
                note_on_id,
                part.clone(),
                element_index,
                drum_setup,
            );
            last_alloc = Some(index);
        }

        // 事件级检查（NoteOn 事件）：复音数超出设定值（max_polyphony，如 512）
        // → 评分释放（高分先杀，双元素同 note_on_id 组一起释放），直到回到
        // 限制以内。冗余池只保证分配不阻塞，复音限制由这里维护。
        let mut active = self
            .tone_generators
            .iter()
            .filter(|t| t.status != Idle)
            .count();
        while active > self.max_polyphony as usize {
            let victim = self
                .tone_generators
                .iter()
                .enumerate()
                .filter(|(_, t)| t.status != Idle)
                .max_by_key(|(_, t)| t.scoring())
                .map(|(i, _)| i);
            match victim {
                Some(v) => {
                    let id = self.tone_generators[v].note_on_id;
                    for tg in self.tone_generators.iter_mut() {
                        if tg.note_on_id == id && tg.status != Idle {
                            tg.kill();
                            active -= 1;
                        }
                    }
                }
                None => break,
            }
        }
    }

    fn release_handler(&mut self, note: Note, part: Arc<DoubleBuffered<Part>>) {
        // CC#64 sustain (Hold1): while the pedal is pressed, NoteOffs are
        // suspended (the voice keeps ringing at its current EG level) and are
        // released in bulk when the pedal is released.
        if part.snapshot().controller.sustain {
            self.sustain_held
                .entry(part.snapshot().id)
                .or_default()
                .push(note);
            // Damper policy: the element's sustain_mode wins when set
            // (1=half-hold/frozen, 2=damper decay); S-YXG50 data has no
            // field (0) → fall back to XG acoustic-piano programs (0-7).
            let snap = part.snapshot();
            let prog = snap.get_ram().program_number;
            for tg in self.tone_generators.iter_mut() {
                if tg.bonded_to_part(&part) && tg.get_note() == Some(note) && tg.status == Running {
                    let damper = match tg.sustain_mode {
                        1 => false,
                        2 => true,
                        _ => prog <= 7,
                    };
                    tg.set_damper_hold(damper);
                }
            }
            return;
        }
        // CC#66 sostenuto: only notes that were already sounding when the
        // pedal was pressed are held; later notes release normally.
        let pid = part.snapshot().id;
        if self
            .sostenuto_active
            .get(&pid)
            .is_some_and(|active| active.contains(&note))
        {
            self.sostenuto_held.entry(pid).or_default().push(note);
            return;
        }
        self.release_note(note, part);
    }

    /// Release the earliest NoteOn group: every still-running voice of that
    /// note with the same note_on_id (dual-element voices of one NoteOn share
    /// the id and release together). NoteOff releases one group per NoteOff
    /// (XG behavior — stacked same-note retriggers release one group each).
    ///
    /// Filtering on Running is required: rapid same-note retriggers can
    /// stack voices with IDENTICAL attack_time instants, and without it the
    /// second NoteOff would pick the same (already released) voice again —
    /// `release()` on a Releasing voice is a no-op, leaving the other voice
    /// ringing forever.
    fn release_note(&mut self, note: Note, part: Arc<DoubleBuffered<Part>>) {
        let target_id = self
            .tone_generators
            .iter()
            .filter(|t| {
                t.bonded_to_part(&part) && t.get_note() == Some(note) && t.status == Running
            })
            .min_by_key(|t| t.note_on_id)
            .map(|t| t.note_on_id);
        if let Some(id) = target_id {
            self.tone_generators
                .iter_mut()
                .filter(|t| {
                    t.bonded_to_part(&part)
                        && t.get_note() == Some(note)
                        && t.status == Running
                        && t.note_on_id == id
                })
                .for_each(|t| t.release());
        }
    }

    /// CC#64 sustain pedal change. On release (on=false), every suspended
    /// NoteOff is executed (each goes through the normal release path).
    fn sustain_change_handler(&mut self, part: Arc<DoubleBuffered<Part>>, on: bool) {
        if on {
            return;
        }
        let pid = part.snapshot().id;
        if let Some(suspended) = self.sustain_held.remove(&pid) {
            for note in suspended {
                self.release_note(note, part.clone());
            }
        }
    }

    /// CC#66 sostenuto pedal change. On press, snapshot the currently
    /// sounding notes (only these will be held); on release, release the
    /// suspended NoteOffs.
    fn sostenuto_change_handler(&mut self, part: Arc<DoubleBuffered<Part>>, on: bool) {
        let pid = part.snapshot().id;
        if on {
            let active: Vec<Note> = self
                .tone_generators
                .iter()
                .filter(|t| t.bonded_to_part(&part) && t.status == Running)
                .filter_map(|t| t.get_note())
                .collect();
            self.sostenuto_active.insert(pid, active);
        } else {
            self.sostenuto_active.remove(&pid);
            if let Some(suspended) = self.sostenuto_held.remove(&pid) {
                for note in suspended {
                    self.release_note(note, part.clone());
                }
            }
        }
    }

    fn release_all_handler(&mut self, part: Arc<DoubleBuffered<Part>>) {
        // CC#123 All Notes Off: release every running voice (including those
        // held by the sustain/sostenuto pedals) and drop the suspended NoteOffs.
        let pid = part.snapshot().id;
        self.sustain_held.remove(&pid);
        self.sostenuto_active.remove(&pid);
        self.sostenuto_held.remove(&pid);
        self.tone_generators
            .iter_mut()
            .filter(|t| t.bonded_to_part(&part) && t.status == Running)
            .for_each(|t| t.release());
    }

    fn kill_all_handler(&mut self, part: Arc<DoubleBuffered<Part>>) {
        // CC#120 All Sound Off: kill immediately (no release), drop the
        // suspended NoteOffs as well.
        let pid = part.snapshot().id;
        self.sustain_held.remove(&pid);
        self.sostenuto_active.remove(&pid);
        self.sostenuto_held.remove(&pid);
        self.tone_generators
            .iter_mut()
            .filter(|t| t.bonded_to_part(&part) && t.status == Running)
            .for_each(|t| t.kill());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xg_level_gain_mapping() {
        assert!((xg_level_gain(127) - 1.0).abs() < 1e-5); // 0dB
        assert_eq!(xg_level_gain(0), 0.0); // -∞ → silence
        // middle values monotonically decrease
        assert!(xg_level_gain(64) < xg_level_gain(80));
        assert!(xg_level_gain(80) < xg_level_gain(127));
    }

    #[test]
    fn attenuator_reversed_mapping() {
        // Master Attenuator: smaller value = higher output, 0x7F = silence
        let g0 = xg_level_gain(127 - 0); // 0 → 0dB
        let g127 = xg_level_gain(127 - 127); // 127 → silence
        assert!((g0 - 1.0).abs() < 1e-5);
        assert_eq!(g127, 0.0);
        // monotonic: larger value = smaller output
        assert!(xg_level_gain(127 - 30) < xg_level_gain(127 - 10));
    }
}
