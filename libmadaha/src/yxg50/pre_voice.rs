use serde::{Deserialize, Serialize};
const CHUNK_SIZE: usize = 78;

pub fn load_prevoice(value: Box<[u8]>) -> Box<[Prevoice]> {
    let mut start: usize = 0;
    let mut prevoices = vec![];

    loop {
        if let Some(v) = value.get(start..)
            && v.len() != 0
        {
            let mut p = Prevoice::from(v);
            p.offset = start;
            let size = (p.elements.len() == 1)
                .then(|| 2 + CHUNK_SIZE)
                .unwrap_or(2 + 2 * CHUNK_SIZE);
            if size == 0 {
                break;
            }
            start += size;
            prevoices.push(p)
        } else {
            break;
        }
    }

    prevoices.into_boxed_slice()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Prevoice {
    pub id: u8,
    pub flag: u8,
    /// Byte offset of this pre-voice definition within seg13/seg14
    /// (prevoiceIdx is an index where byte offset = idx * 2).
    #[serde(skip)]
    pub offset: usize,
    pub elements: Box<[Element]>,
}

impl Prevoice {
    pub fn get_elements(&self) -> Option<(&Element, Option<&Element>)> {
        Some((self.elements.get(0)?, self.elements.get(1)))
    }
}

impl From<&[u8]> for Prevoice {
    fn from(value: &[u8]) -> Self {
        // Dual-element detection: Ghidra FUN_10016fa0 — count = (header[1] & 2) ? 2 : 1.
        // The old `& 0x3 == 3` required BOTH bits, so voices whose header only
        // sets bit 1 (e.g. Dream, bank LSB=41 prog=0) lost their second element.
        let elements = if value[1] & 0x2 != 0 {
            vec![
                value
                    .get(2..2 + CHUNK_SIZE)
                    .map(|d| Element::from(d.as_array().unwrap()))
                    .unwrap(),
                value
                    .get(2 + CHUNK_SIZE..2 + 2 * CHUNK_SIZE)
                    .map(|d| Element::from(d.as_array().unwrap()))
                    .unwrap(),
            ]
            .into_boxed_slice()
        } else if value[1] & 0x1 == 1 {
            vec![
                value
                    .get(2..2 + CHUNK_SIZE)
                    .map(|d| Element::from(d.as_array().unwrap()))
                    .unwrap(),
            ]
            .into_boxed_slice()
        } else {
            vec![].into_boxed_slice()
        };

        Self {
            id: value[0],
            flag: value[1],
            offset: 0,
            elements,
        }
    }
}

/// S-YXG50 TBL Element (78 bytes)
///
/// Confirmed via Ghidra decompilation (S-YXG50.dll), synced from note_opencode.md (2026-07-23);
/// field semantics re-verified byte-level (2026-08-12, dynamic S-YXG50 + Ghidra disassembly):
/// 0x44= tbl_68, 0x45= eg_phase→0x1c7, 0x46= wave_pitch→0x1ca(KeyOnDelay 0x500),
/// 0x47= eg_enable→voice[0x64]/[0x6a], 0x48= key_on_delay→voice[0x66]/[0x69],
/// 0x49= trig_mode (FUN_10012900), 0x4a= alt_ovr, 0x4b= off_hi, 0x4c= off_lo, 0x4d= sensitivity.
///
/// Each pre-voice definition contains 1 or 2 elements.
/// element[i] is located at `voice_base + 2 + i * 78`.
///
/// File layout:
/// ```text
///  [0..4]  waveform/key/velocity match parameters
///  [5..10] LFO/velocity/pitch/volume offsets
///  [11..14] Pitch EG + Filter
///  [15..17] mode + range + voice type
///  [18..30] NoteShift + Detune + PEG parameters
///  [31..77] DSP synthesis parameters (EG/filter/AEG/LFO/output)
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    // ── waveform/key/velocity matching (5 bytes) ══════════════════════════════
    /// dataSeg15 index (0-245)
    pub index: u8,
    /// Key range lower bound (FUN_10017060: key_range matching)
    pub key_min: u8,
    /// Key range upper bound
    pub key_max: u8,
    /// Velocity lower bound
    pub vel_min: u8,
    /// Velocity upper bound
    pub vel_max: u8,

    // ── LFO/velocity/pitch/volume (4 bytes) ══════════════════════════════
    /// LFO waveform selection (FUN_100155B0: mask 0x7 → LFO wave table 0x10048130;
    /// musicbox 1/1)
    pub lfo_wave: u8,
    /// Velocity/layer threshold (FUN_100155B0: TEST → engine 0x1c6=0x80 when non-zero;
    /// musicbox 1/1)
    pub vel_threshold: u8,
    /// Pitch offset (signed; applied through the voice cache — 0x10015460 reads the
    /// cached copy, not element[7] directly; musicbox 25)
    pub pitch_offset: i8,
    /// Volume offset (signed; applied through the voice cache — 0x100156C0 reads
    /// the wave entry, not element[8] directly; musicbox 0)
    pub vol_offset: i8,

    // ── pitch fine (2 bytes) ════════════════════════════════════════
    /// Pitch fine: combined = (elem[9]-8)*256 + elem[10]*16 → 12-bit (read via cache;
    /// musicbox 0/0)
    pub pitch_fine_h: u8,
    pub pitch_fine_l: u8,

    // ── Pitch EG + Filter (4 bytes) ═══════════════════════════════
    /// Pitch EG Attack rate (0=fastest)
    pub pitch_eg_attack: u8,
    /// Pitch EG Decay/Release rate (0=fastest, range 0-3)
    pub pitch_eg_decay: u8,
    /// Filter cutoff frequency (64=center)
    pub filter_cutoff: u8,
    /// ⚠ 命名修正（2026-08-12）：rwatch 证实 elem[14] 在 0x100154cd 被读为
    /// **音高分量**（FUN_10015460 公式：`(range−baseKey)×100 + tone + 键表 − 0x40
    /// + elem[14] + note[7] − 0x40`，0x40 中心）——非滤波器共鸣（引擎渲染链无共振滤波）。
    /// madaha 已接入 oscillator 音高（pitch_comp − 64 分）。
    pub pitch_comp: u8,

    // ── mode/range/type (3 bytes) ══════════════════════════════════
    /// Pitch mode (0=direct addition, 1-4=lookup table)
    pub pitch_mode: u8,
    /// Range base value (60=middle C)
    pub range_base: u8,
    /// Voice type — ⚠ **引擎未使用**（2026-08-12 rwatch 12+ KeyOn 多音色无读取，
    /// 疑似遗留字段；madaha 若使用需复核）
    pub voice_type: u8,

    // ── NoteShift + Detune + PEG front (4 bytes) ═════════════════
    /// Note Shift (32-96, 64=center); overwritten at runtime
    pub note_shift: u8,
    /// Detune (14-114, 64=center); overwritten at runtime
    pub detune: u8,
    /// PEG Center Note Low
    pub peg_center_low: u8,
    /// PEG Center Note High
    pub peg_center_high: u8,

    // ── PEG front section (4 bytes) ═══════════════════════════════════════
    /// PEG Vel Sense Level
    pub peg_vel_sense_level: u8,
    /// PEG Vel Sense Rate
    pub peg_vel_sense_rate: u8,
    /// PEG Rate Scaling
    pub peg_rate_scaling: u8,
    /// PEG Center Note — ⚠ **引擎未读**（2026-08-12 rwatch 12+ KeyOn 无读取）
    pub peg_center_note: u8,

    // ── PEG rates (5 bytes, range 0-127) ═══════════════════════════
    /// PEG Rate 0 — ⚠ **引擎未读**（同前；madaha 已停用，stage1 改用 peg_rate1）
    pub peg_rate0: u8,
    /// PEG Rate 1
    pub peg_rate1: u8,
    /// PEG Rate 2
    pub peg_rate2: u8,
    /// PEG Rate 3
    pub peg_rate3: u8,
    /// PEG Rate 4 — ⚠ **引擎未读**（同前；madaha 已停用，release 暂用 stage3 速率）
    pub peg_rate4: u8,

    // ── DSP synthesis parameters [31..77] ════════════════════════════════════
    /// DSP parameter base index (read by FUN_10013456 → vtable[0x4F0])
    pub dsp_base: u8,
    /// Cutoff modulation depth (rwatch @0x1001427d, FUN_10014200:
    /// `(~v & 0x7f) × (elem[32] × 0x24) >> 8` — 参与 2D 表 0x10047F50 截止查找)
    pub cutoff_mod: u8,
    /// Lookup table index → pitch scaling (read by 0x10015887, 2D lookup 0x100473D0)
    pub tbl_index: u8,
    /// Curve A breakpoint x0 (4-point piecewise-linear key curve at elem[34..41],
    /// evaluated at note key by FUN_100164f0, ×32 → 12-bit DSP param voice[0x18])
    pub curve_a_x0: u8,
    /// Curve A breakpoint x1 (also read as pitch_coarse at 0x10006D4D → voice[0xB4])
    pub pitch_coarse: u8,
    /// Curve A breakpoint x2
    pub curve_a_x2: u8,
    /// Curve A breakpoint x3
    pub curve_a_x3: u8,
    /// Curve A value y0
    pub curve_a_y0: u8,
    /// Curve A value y1
    pub curve_a_y1: u8,
    /// Curve A value y2 (also read as Filter EG enable at 0x10019643)
    pub eg_filt_en: u8,
    /// Curve A value y3 (also read as Amp EG enable at 0x10007315 → voice[0x6E])
    pub eg_amp_en: u8,
    /// LFO enable (0x10007354)
    pub lfo_en: u8,
    /// Pitch EG enable (0x100073E6: non-zero→calls the vtable[0x18C] PEG)
    pub eg_pitch_en: u8,
    /// Key-follow amount (0x10013e24, 0x40-centered: (key−[45])×([44]−0x40)/16 → voice[0x57];
    /// also read as output enable at 0x100074FA)
    pub output_en: u8,
    /// Key-follow reference key (0x10013e34 rwatch-verified; musicbox 60)
    pub keyfol_ref: u8,
    /// Cutoff override flag (0x10007834: 0=default lookup, non-zero=override)
    pub ovr_cutoff: u8,
    /// Cutoff Scaling stage 1 enable
    pub cs_en_1: u8,
    /// Cutoff Scaling stage 2 enable
    pub cs_en_2: u8,
    /// Level Scaling enable (0x100196E1 → FUN_1001B630)
    pub ls_en: u8,
    /// Level Scaling store
    pub ls_store: u8,
    /// Level Scaling post-processing comparison
    pub ls_cmp: u8,
    /// Level Scaling flag
    pub ls_flag: u8,
    /// Level Scaling 3rd parameter (0x10013bb0: compared against ls_cmp[51]/ls_flag[52]
    /// to select the CS scaling stage — NOT padding; musicbox 64/64)
    pub ls_cmp2: u8,
    /// AEG Decay1 Rate override enable — audit: elem[54] → voice[0x60] flag (≥64 → 1)
    /// + voice[0x61] = param value (elem[55] is the value, not unused)
    pub aeg_d1: u8,
    /// AEG Decay1 rate value (elem[55] → voice[0x61] via 0x100062B0 param; musicbox 115/24 —
    /// NOT padding, exact consumer pending)
    pub aeg_d1_val: u8,
    /// AEG Decay2 Rate override enable — audit: elem[56] → voice[0xc0] flag + voice[0xc1] = param
    /// (elem[58] is the value). NOT a sustain-level source — the madaha `1 − aeg_d2/127`
    /// sustain mapping is an approximation pending the voice-layout rebuild.
    /// Also Curve B breakpoint x0 (curve at elem[56..63], rwatch-verified)
    pub aeg_d2: u8,
    /// AEG Release Rate override enable — audit: elem[57] → voice[0x68] = param value.
    /// Also Curve B breakpoint x1
    pub aeg_rel: u8,
    /// Curve B breakpoint x2 (elem[58], rwatch-verified @0x10016523)
    pub curve_b_x2: u8,
    /// Curve B breakpoint x3 (elem[59], rwatch-verified @0x1001650f)
    pub curve_b_x3: u8,
    /// Curve B value y0 (elem[60])
    pub curve_b_y0: u8,
    /// Curve B value y1 (elem[61])
    pub curve_b_y1: u8,
    /// Curve B value y2 (elem[62])
    pub curve_b_y2: u8,
    /// Curve B value y3 (elem[63])
    pub curve_b_y3: u8,
    /// EG rate remap index (0x100142AC: shift left 7 → 2D lookup; reads elem[64]=0x40)
    pub rate_idx: u8,
    /// Table index → voice[0x76]/[0x77] (0x100480A0; 0xF sentinel → 0x100480B0[key];
    /// FUN_10015940, feeds FUN_10013580 param calc)
    pub tbl65_idx: u8,
    /// Key-follow depth (0x10015783 rwatch-verified: (key−[67])×([66]−0x40)×16>>8 → voice[0x65])
    pub keyfollow_depth: u8,
    /// Sample format flag (0x10038D6C: reads elem[67]=0x43; 0=8bit, non-zero=16bit).
    /// Dual-use: also the key-follow reference key at 0x1001577e (key−[67] in FUN_10015770)
    pub fmt_flag: u8,
    /// Lookup table index (0x10015834: reads elem[68]=0x44 → word table 0x10048134 → voice[0xE])
    pub tbl_68: u8,
    /// Key-corrected parameter (0x10012550: elem[69]=0x45 → engine.field_0x1c7 → voice[0x67];
    /// 0x7e special-cased to 0xff; musicbox elem value 63 → 0x1c7=0xff)
    pub eg_phase: u8,
    /// KeyOnDelay source (0x100125B0: elem[70]=0x46 → engine.field_0x1ca → 0x500 stage table;
    /// ×2 after key correction — musicbox 34/29 → 0x1ca=68/58; also a trigger-table index
    /// (DAT_10046d48) in the 0x10012799 path)
    pub wave_pitch: u8,
    /// Key-corrected EG parameter (0x10012600: elem[71]=0x47 → voice[0x64]+voice[0x6a];
    /// trigger-related clamp 0x30 in 0x10012781; musicbox 22/21)
    pub eg_enable: u8,
    /// Key-on delay (0x10012670: elem[72]=0x48 → voice[0x69]+voice[0x66], key-corrected;
    /// musicbox VST=31 / S-YXG50=22/21; consumed by the madaha AEG Delay stage)
    pub key_on_delay: u8,
    /// Trigger/retrigger parameter (0x100127A0: reads elem[73]=0x49 — NOT eg_delay;
    /// FUN_10012900 ratio (127-v)<<16 / DAT_10046d48[w*2]; 0x10012700 sets voice[0x6b]=0x7e,
    /// engine 0x1c9=0 when non-zero — musicbox 127)
    pub trig_mode: u8,
    /// Override lookup value (0x10014238: reads elem[74]=0x4a; non-zero → overrides 0x10047AD8)
    pub alt_ovr: u8,
    /// Sample offset high (0x10015690: reads elem[75]=0x4b)
    pub off_hi: u8,
    /// Sample offset low ([76]=0x4c low 7 bits)
    pub off_lo: u8,
    /// Signed sensitivity (0x10013530: reads elem[77]=0x4d, signed −0x40 → modulates element[31])
    pub sensitivity: u8,
    /// Reserved: sustain pedal mode (2006LE format: 0=none, 1=half-hold, 2=damper).
    /// S-YXG50 data has no such field — parsed as 0 (no damper behavior).
    /// Populated when reading 2006LE data files; not yet consumed.
    pub sustain_mode: u8,
}

impl From<Box<[u8]>> for Element {
    fn from(value: Box<[u8]>) -> Self {
        Self {
            index: value[0],
            key_min: value[1],
            key_max: value[2],
            vel_min: value[3],
            vel_max: value[4],
            lfo_wave: value[5],
            vel_threshold: value[6],
            pitch_offset: value[7] as i8,
            vol_offset: value[8] as i8,
            pitch_fine_h: value[9],
            pitch_fine_l: value[10],
            pitch_eg_attack: value[11],
            pitch_eg_decay: value[12],
            filter_cutoff: value[13],
            pitch_comp: value[14],
            pitch_mode: value[15],
            range_base: value[16],
            voice_type: value[17],
            note_shift: value[18],
            detune: value[19],
            peg_center_low: value[20],
            peg_center_high: value[21],
            peg_vel_sense_level: value[22],
            peg_vel_sense_rate: value[23],
            peg_rate_scaling: value[24],
            peg_center_note: value[25],
            peg_rate0: value[26],
            peg_rate1: value[27],
            peg_rate2: value[28],
            peg_rate3: value[29],
            peg_rate4: value[30],
            dsp_base: value[31],
            cutoff_mod: value[32],
            tbl_index: value[33],
            curve_a_x0: value[34],
            pitch_coarse: value[35],
            curve_a_x2: value[36],
            curve_a_x3: value[37],
            curve_a_y0: value[38],
            curve_a_y1: value[39],
            eg_filt_en: value[40],
            eg_amp_en: value[41],
            lfo_en: value[42],
            eg_pitch_en: value[43],
            output_en: value[44],
            keyfol_ref: value[45],
            ovr_cutoff: value[46],
            cs_en_1: value[47],
            cs_en_2: value[48],
            ls_en: value[49],
            ls_store: value[50],
            ls_cmp: value[51],
            ls_flag: value[52],
            ls_cmp2: value[53],
            aeg_d1: value[54],
            aeg_d1_val: value[55],
            aeg_d2: value[56],
            aeg_rel: value[57],
            curve_b_x2: value[58],
            curve_b_x3: value[59],
            curve_b_y0: value[60],
            curve_b_y1: value[61],
            curve_b_y2: value[62],
            curve_b_y3: value[63],
            rate_idx: value[64],
            tbl65_idx: value[65],
            keyfollow_depth: value[66],
            fmt_flag: value[67],
            tbl_68: value[68],
            eg_phase: value[69],
            wave_pitch: value[70],
            eg_enable: value[71],
            key_on_delay: value[72],
            trig_mode: value[73],
            alt_ovr: value[74],
            off_hi: value[75],
            off_lo: value[76],
            sensitivity: value[77],
            sustain_mode: 0,
        }
    }
}

impl From<&[u8; 78]> for Element {
    fn from(value: &[u8; 78]) -> Self {
        Self {
            index: value[0],
            key_min: value[1],
            key_max: value[2],
            vel_min: value[3],
            vel_max: value[4],
            lfo_wave: value[5],
            vel_threshold: value[6],
            pitch_offset: value[7] as i8,
            vol_offset: value[8] as i8,
            pitch_fine_h: value[9],
            pitch_fine_l: value[10],
            pitch_eg_attack: value[11],
            pitch_eg_decay: value[12],
            filter_cutoff: value[13],
            pitch_comp: value[14],
            pitch_mode: value[15],
            range_base: value[16],
            voice_type: value[17],
            note_shift: value[18],
            detune: value[19],
            peg_center_low: value[20],
            peg_center_high: value[21],
            peg_vel_sense_level: value[22],
            peg_vel_sense_rate: value[23],
            peg_rate_scaling: value[24],
            peg_center_note: value[25],
            peg_rate0: value[26],
            peg_rate1: value[27],
            peg_rate2: value[28],
            peg_rate3: value[29],
            peg_rate4: value[30],
            dsp_base: value[31],
            cutoff_mod: value[32],
            tbl_index: value[33],
            curve_a_x0: value[34],
            pitch_coarse: value[35],
            curve_a_x2: value[36],
            curve_a_x3: value[37],
            curve_a_y0: value[38],
            curve_a_y1: value[39],
            eg_filt_en: value[40],
            eg_amp_en: value[41],
            lfo_en: value[42],
            eg_pitch_en: value[43],
            output_en: value[44],
            keyfol_ref: value[45],
            ovr_cutoff: value[46],
            cs_en_1: value[47],
            cs_en_2: value[48],
            ls_en: value[49],
            ls_store: value[50],
            ls_cmp: value[51],
            ls_flag: value[52],
            ls_cmp2: value[53],
            aeg_d1: value[54],
            aeg_d1_val: value[55],
            aeg_d2: value[56],
            aeg_rel: value[57],
            curve_b_x2: value[58],
            curve_b_x3: value[59],
            curve_b_y0: value[60],
            curve_b_y1: value[61],
            curve_b_y2: value[62],
            curve_b_y3: value[63],
            rate_idx: value[64],
            tbl65_idx: value[65],
            keyfollow_depth: value[66],
            fmt_flag: value[67],
            tbl_68: value[68],
            eg_phase: value[69],
            wave_pitch: value[70],
            eg_enable: value[71],
            key_on_delay: value[72],
            trig_mode: value[73],
            alt_ovr: value[74],
            off_hi: value[75],
            off_lo: value[76],
            sensitivity: value[77],
            sustain_mode: 0,
        }
    }
}

impl From<[u8; 78]> for Element {
    fn from(value: [u8; 78]) -> Self {
        Self::from(&value)
    }
}

impl Element {
    /// Combined pitch fine: `(elem[9]-8)*256 + elem[10]*16` → 12-bit value
    pub fn pitch_fine(&self) -> i16 {
        ((self.pitch_fine_h as i16 - 8) * 256) + (self.pitch_fine_l as i16 * 16)
    }
    /// LFO waveform selection (low 3 bits)
    pub fn lfo_waveform(&self) -> u8 {
        self.lfo_wave & 0x07
    }
    /// LFO waveform variation/drum key number (high 5 bits)
    pub fn lfo_wave_variation(&self) -> u8 {
        self.lfo_wave >> 3
    }
    /// Combined sample offset: `(off_hi << 7) | off_lo`
    pub fn sample_offset_pair(&self) -> u16 {
        ((self.off_hi as u16) << 7) | self.off_lo as u16
    }
    /// Sensitivity signed: `value - 64`
    pub fn sensitivity_signed(&self) -> i8 {
        self.sensitivity as i8 - 64
    }
    /// Key matching: `min <= note <= max`
    pub fn matches_key(&self, note: u8) -> bool {
        note >= self.key_min && note <= self.key_max
    }
    /// Velocity matching: `min <= vel <= max`
    pub fn matches_vel(&self, vel: u8) -> bool {
        vel >= self.vel_min && vel <= self.vel_max
    }

    /// S-YXG50 键位范围计算（FUN_100140f0 / ElementCalc_Pitch）：
    /// 决定 note.range（参与音高键跟随与 WaveEntry 链扫描）：
    /// - mode 0：range = key（默认）
    /// - mode 1-4：range = range_base + 表0x10047750[mode]×(key−range_base)/100
    ///   （表值 50%/20%/10%/5%）
    /// - mode ≥5：range = range_base
    pub fn compute_range(&self, key: u8) -> u8 {
        element_range(self.pitch_mode, self.range_base, key)
    }
}

/// S-YXG50 键位范围自由函数（供合成器/SampleMeta 使用，FUN_100140f0）
pub fn element_range(pitch_mode: u8, range_base: u8, key: u8) -> u8 {
    const PITCH_MODE_SCALE: [i32; 4] = [50, 20, 10, 5]; // 0x10047750[1..4]
    match pitch_mode {
        0 => key,
        1..=4 => {
            let base = key as i32;
            let rb = range_base as i32;
            let scale = PITCH_MODE_SCALE[pitch_mode as usize - 1];
            (rb + scale * (base - rb) / 100).clamp(0, 127) as u8
        }
        _ => range_base,
    }
}

/// FUN_100164f0：4 断点分段线性键位曲线（曲线 A = elem[34..41]、
/// 曲线 B = elem[56..63]）。返回 y−0x40（0x40 中心），结果 clamp [−0x40, 0x3f]。
/// 语义（汇编逐级确认）：
/// - v ≤ x0 → y0；v ≥ x3 → y3；v == x2 → y2；v == x1 → y1
/// - 区间内 → y_cur + (v−x_cur)·(y_next−y_cur)/(x_next−x_cur) − 0x40（FUN_100165b0）
pub fn piecewise_curve(key: u8, x: [u8; 4], y: [u8; 4]) -> i32 {
    if key <= x[0] {
        return y[0] as i32 - 0x40;
    }
    if key >= x[3] {
        return y[3] as i32 - 0x40;
    }
    if key == x[2] {
        return y[2] as i32 - 0x40;
    }
    if key > x[2] {
        return curve_interp(key, x[2], y[2], x[3], y[3]);
    }
    if key == x[1] {
        return y[1] as i32 - 0x40;
    }
    if key > x[1] {
        return curve_interp(key, x[1], y[1], x[2], y[2]);
    }
    curve_interp(key, x[0], y[0], x[1], y[1])
}

/// FUN_100165b0：分段插值 `y0 + (v−x0)·(y1−y0)/(x1−x0) − 0x40`，clamp [−0x40, 0x3f]
fn curve_interp(v: u8, x0: u8, y0: u8, x1: u8, y1: u8) -> i32 {
    let den = (x1 as i32) - (x0 as i32);
    if den == 0 {
        return y0 as i32 - 0x40;
    }
    let num = ((v as i32) - (x0 as i32)) * ((y1 as i32) - (y0 as i32));
    let mut r = (y0 as i32) + num / den;
    if (y1 as i32) < (y0 as i32) {
        if r < 0 {
            r = 0;
        }
    } else if 0x7f < r {
        r = 0x7f;
    }
    r - 0x40
}

/// 键跟随（三组同公式，FUN_10013e20/FUN_10015770/FUN_10015f60 汇编确认）：
/// `(key − ref) × (amount − 0x40) × 16 >> 8`，amount == 0x40 → 0。
/// - 组 A：ref = elem[45]、amount = elem[44]（→ voice[0x57]）
/// - 组 B：ref = elem[67]、amount = elem[66]（→ voice[0x65]）
/// - 组 C：ref = elem[21]、amount = elem[20]（→ voice[0x4f]）
/// 消费方（voice[0x57]/[0x65]/[0x4f]）待 voice 布局重建后接线。
pub fn key_follow(key: u8, ref_: u8, amount: u8) -> i32 {
    let amt = amount as i32 - 0x40;
    if amt == 0 {
        return 0;
    }
    ((key as i32 - ref_ as i32) * amt) >> 4
}

/// PEG 速率字（FUN_10015fe0 精确语义，表 0x10048134）：
/// 输入：`rate_base`（stage0/1/2 分别 = elem[26]/[23]/[24]）、`key_in`（键跟输入：
/// stage0=elem[22] 调整、stage1=0x40、stage2=part[0x1b]）、`key_follow_c`（voice[0x4f]）、
/// `elem19_scaled`（voice[0x50]）
/// ```text
/// v = ((0x40 − key_in) >> 2) + rate_base，clamp 0..0x3f
/// v += 键跟随C；v<0→0；v>0x3e→哨兵 0x8000（即时，FUN_10015ab7 检测）
/// v += elem[19]缩放；v>0x3e→哨兵 0x1830
/// 返回 表0x10048134[v + 0x12]
/// ```
/// 表 0x10048134（88 项 u16，2026-08-14 完整 dump）——速率字直接每块累加到 PEG 电平。
pub fn peg_rate_word(rate_base: u8, key_in: u8, key_follow_c: i32, elem19_scaled: i32) -> u16 {
    let mut v = (((0x40i32 - key_in as i32) >> 2) + rate_base as i32).clamp(0, 0x3f);
    v += key_follow_c;
    if v < 0 {
        v = 0;
    } else if v > 0x3e {
        return 0x8000; // DAT_100481d6（即时速率）
    }
    v += elem19_scaled;
    if v > 0x3e {
        return 0x1830; // DAT_100481d4（极快速率）
    }
    peg_rate_table(v + 0x12)
}

/// 表 0x10048134：PEG/EG 速率曲线（完整 88 项 u16，2026-08-14 动态 dump）。
/// 0-15 指数段、16-17 零、18+ 递增曲线（非简单线性）。
pub fn peg_rate_table(idx: i32) -> u16 {
    const TABLE: [u16; 88] = [
        0x0000, 0x0014, 0x0028, 0x0050, 0x00a0, 0x00f0, 0x0140, 0x01e0, // 0-7
        0x0280, 0x0370, 0x0500, 0x0690, 0x0a00, 0x0d70, 0x1450, 0x1b30, // 8-15
        0x0000, 0x0000, // 16-17
        0x0002, 0x0004, 0x0006, 0x0008, 0x000a, 0x000c, 0x000e, 0x0010, // 18-25
        0x0012, 0x0014, 0x0016, 0x0018, 0x001a, 0x001c, 0x001e, 0x0020, // 26-33
        0x0022, 0x0024, 0x0026, 0x0028, 0x002a, 0x002c, 0x002e, 0x0030, // 34-41
        0x0032, 0x0034, 0x0036, 0x0038, 0x003c, 0x0044, 0x004c, 0x0054, // 42-49
        0x0060, 0x006c, 0x007c, 0x008c, 0x009c, 0x00b0, 0x00c8, 0x00e0, // 50-57
        0x00fc, 0x011c, 0x0140, 0x016c, 0x0198, 0x01cc, 0x0208, 0x024c, // 58-65
        0x0298, 0x02ec, 0x034c, 0x03b8, 0x0430, 0x04bc, 0x0554, 0x0604, // 66-73
        0x06cc, 0x07a8, 0x08a4, 0x09c0, 0x0b00, 0x1000, 0x1830, 0x8000, // 74-81
        0x0800, 0x07ff, 0x07fe, 0x07fc, 0x07fb, 0x07fa, // 82-87
    ];
    TABLE.get(idx as usize).copied().unwrap_or(0)
}

/// PEG 电平（FUN_10015e50 语义，`offset = elem[x] − 0x40`，返回内部电平单位）：
/// ```text
/// mag = |offset| + (offset > 0 ? 1 : 0)
/// v = (mag − (vel_sense × mag) >> 8) × 75
/// v = [÷4, ÷2, ×1, ×2][mode]     ; mode = elem[17] voice_type（0/1/2/3）
/// 返回 sign × v
/// ```
/// 内部电平 >> 2 = cents（mode 2 满幅 ±64 → ±1200 cents = ±12 半音）。
/// FUN_10015f10 = 同公式但倍率固定 ÷2（用于 Part 初始电平偏移）。
pub fn peg_level(offset: i32, vel_sense: u8, mode: u8) -> i32 {
    let sign = if offset >= 0 { 1 } else { -1 };
    let mut mag = offset.abs();
    if offset > 0 {
        mag += 1;
    }
    let mut v = mag - ((vel_sense as i32 * mag) >> 8);
    v *= 75;
    v = match mode {
        0 => v >> 2,
        1 => v >> 1,
        3 => v << 1,
        _ => v,
    };
    sign * v
}

/// PEG 力度电平敏感度（FUN_10016040，elem[18] note_shift → voice[0x51]，有符号）：
/// 正：`(0x80 − vel) × (elem18 − 0x40) × 9 >> 5`；负：`vel × (elem18 − 0x40) × −36 >> 7`
pub fn peg_vel_sense_level(elem18: u8, vel: u8) -> i32 {
    let d = elem18 as i32 - 0x40;
    if d == 0 {
        return 0;
    }
    if d > 0 {
        ((0x80 - vel as i32) * d * 9) >> 5
    } else {
        (vel as i32 * d * -36) >> 7
    }
}

/// PEG 力度速率缩放（FUN_10015f90，elem[19] detune → voice[0x50]，有符号）：
/// 正：`vel × (elem19 − 0x40) × 16 >> 8`；负：`−(0x80 − vel) × (elem19 − 0x40) × 16 >> 8`
pub fn peg_vel_sense_rate(elem19: u8, vel: u8) -> i32 {
    let d = elem19 as i32 - 0x40;
    if d == 0 {
        return 0;
    }
    if d >= 0 {
        (vel as i32 * d * 16) >> 8
    } else {
        -((0x80 - vel as i32) * d * 16) >> 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_follow_neutral() {
        // amount 64 → 0（无键跟随）
        assert_eq!(key_follow(60, 60, 64), 0);
        assert_eq!(key_follow(72, 60, 64), 0);
    }

    #[test]
    fn key_follow_value() {
        // (key−ref) × (amount−64) ×16 >>8（引擎 SAR 向下取整）
        // amount=80 → +1/键；key 72, ref 60 → 12×16/16 = 12
        assert_eq!(key_follow(72, 60, 80), 12);
        // amount=63 → −1/键 → 12×(−1)>>4 = −1（SAR floor）
        assert_eq!(key_follow(72, 60, 63), -1);
        // amount=48 → −16/16 = −1/键 → −12
        assert_eq!(key_follow(72, 60, 48), -12);
        // key == ref → 0
        assert_eq!(key_follow(60, 60, 48), 0);
    }

    #[test]
    fn peg_rate_word_basic() {
        // rate_base=63, key_in=0x40 → v=63 > 0x3e → 哨兵 0x8000（即时）
        assert_eq!(peg_rate_word(63, 0x40, 0, 0), 0x8000);
        // rate_base=0, key_in=0x40 → idx 18 → 表 0x0002
        assert_eq!(peg_rate_word(0, 0x40, 0, 0), 0x0002);
        // rate_base=15, key_in=0x40 → idx 33 → 表 0x0020
        assert_eq!(peg_rate_word(15, 0x40, 0, 0), 0x0020);
        // 键跟随 C 上推 → 哨兵 0x8000
        assert_eq!(peg_rate_word(63, 0x40, 1, 0), 0x8000);
        // elem[19] 缩放上推 → 哨兵 0x1830
        assert_eq!(peg_rate_word(60, 0x40, 0, 20), 0x1830);
        // key_in 项：rate_base=0, key_in=0 → ((0x40−0)>>2)=16 → idx 34 → 0x0022
        assert_eq!(peg_rate_word(0, 0x00, 0, 0), 0x0022);
        // key_in=0x7f → ((0x40−0x7f)>>2) = −16 → rate_base 20 − 16 = 4 → idx 22 → 0x000a
        assert_eq!(peg_rate_word(20, 0x7f, 0, 0), 0x000a);
    }

    #[test]
    fn peg_level_modes() {
        // offset 64, vel_sense 0, mode 2（×1）→ (64+1)×75 = 4875
        assert_eq!(peg_level(64, 0, 2), 4875);
        // mode 0（÷4）→ 4875 >> 2 = 1218
        assert_eq!(peg_level(64, 0, 0), 1218);
        // mode 1（÷2）→ 4875 >> 1 = 2437
        assert_eq!(peg_level(64, 0, 1), 2437);
        // mode 3（×2）→ 9750
        assert_eq!(peg_level(64, 0, 3), 9750);
        // offset 0（中性）→ 0
        assert_eq!(peg_level(0, 0, 2), 0);
        // 负 offset −32 → sign −1, mag 32 → 32×75 = 2400, mode 2 → −2400
        assert_eq!(peg_level(-32, 0, 2), -2400);
        // 负 offset mode 0 → −2400 >> 2 = −600
        assert_eq!(peg_level(-32, 0, 0), -600);
    }

    #[test]
    fn peg_vel_sense_scales() {
        // 中性 0x40 → 0
        assert_eq!(peg_vel_sense_level(0x40, 100), 0);
        assert_eq!(peg_vel_sense_rate(0x40, 100), 0);
        // 正 elem18=0x50, vel=100 → (0x80−100)×16×9>>5 = 28×144>>5 = 126
        assert_eq!(peg_vel_sense_level(0x50, 100), (0x80 - 100) * 16 * 9 >> 5);
        // 负 elem18=0x30, vel=100 → 100×(−16)×−36>>7 = 450
        assert_eq!(peg_vel_sense_level(0x30, 100), 100 * -16 * -36 >> 7);
        // 正 elem19=0x50, vel=100 → 100×16×16>>8 = 100
        assert_eq!(peg_vel_sense_rate(0x50, 100), 100 * 16 * 16 >> 8);
        // 负 elem19=0x30, vel=100 → −(0x80−100)×(−16)×16>>8 = −(−28×16×16>>8) = 28
        assert_eq!(peg_vel_sense_rate(0x30, 100), -((0x80 - 100) * -16 * 16) >> 8);
    }

    #[test]
    fn peg_rate_table_curve() {
        // 0-15 指数段（精确表值，0x10048134 dump）
        let exp = [0x0000, 0x0014, 0x0028, 0x0050, 0x00A0, 0x00F0, 0x0140, 0x01E0,
                   0x0280, 0x0370, 0x0500, 0x0690, 0x0A00, 0x0D70, 0x1450, 0x1B30];
        for (i, v) in exp.iter().enumerate() {
            assert_eq!(peg_rate_table(i as i32), *v, "idx {i}");
        }
        // 16-17 为零
        assert_eq!(peg_rate_table(16), 0);
        assert_eq!(peg_rate_table(17), 0);
        // 18+ 递增曲线（非简单线性——完整 dump 修正）
        assert_eq!(peg_rate_table(18), 0x0002);
        assert_eq!(peg_rate_table(19), 0x0004);
        assert_eq!(peg_rate_table(20), 0x0006);
        assert_eq!(peg_rate_table(45), 0x0038);
        assert_eq!(peg_rate_table(46), 0x003c);
        assert_eq!(peg_rate_table(80), 0x1830);
        assert_eq!(peg_rate_table(81), 0x8000);
    }

    #[test]
    fn element_range_mode0_is_key() {
        assert_eq!(element_range(0, 60, 72), 72);
        assert_eq!(element_range(0, 60, 0), 0);
    }

    #[test]
    fn element_range_mode_scaled() {
        // mode 1 (50%): range = 60 + 50×(72−60)/100 = 66
        assert_eq!(element_range(1, 60, 72), 66);
        // mode 2 (20%): 60 + 20×12/100 = 62
        assert_eq!(element_range(2, 60, 72), 62);
        // mode 3 (10%): 61
        assert_eq!(element_range(3, 60, 72), 61);
        // mode 4 (5%): 60 + 5×12/100 = 60
        assert_eq!(element_range(4, 60, 72), 60);
        // key == base → 不变
        assert_eq!(element_range(1, 60, 60), 60);
    }

    #[test]
    fn element_range_mode5_uses_base() {
        assert_eq!(element_range(5, 60, 72), 60);
        assert_eq!(element_range(9, 42, 100), 42);
    }

    #[test]
    fn piecewise_curve_at_breakpoints() {
        let x = [59, 64, 72, 96];
        let y = [59, 64, 73, 80];
        assert_eq!(piecewise_curve(50, x, y), 59 - 0x40); // ≤x0 → y0−0x40
        assert_eq!(piecewise_curve(59, x, y), 59 - 0x40);
        assert_eq!(piecewise_curve(64, x, y), 64 - 0x40); // x1
        assert_eq!(piecewise_curve(72, x, y), 73 - 0x40); // x2
        assert_eq!(piecewise_curve(96, x, y), 80 - 0x40); // x3
        assert_eq!(piecewise_curve(127, x, y), 80 - 0x40); // ≥x3
    }

    #[test]
    fn piecewise_curve_interpolates() {
        let x = [59, 64, 72, 96];
        let y = [59, 64, 73, 80];
        // v=60: y0 + (60−59)·(64−59)/(64−59) = 60 → −0x40+... = 60−64 = −4
        assert_eq!(piecewise_curve(60, x, y), 60 - 0x40);
        // v=63: 59 + 4×5/5 = 63
        assert_eq!(piecewise_curve(63, x, y), 63 - 0x40);
    }
}
