/// S-YXG50 TBL Element (78 bytes)
///
/// Confirmed via Ghidra decompilation (S-YXG50.dll), synced from note_opencode.md (2026-07-23)
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
#[derive(Debug)]
#[repr(C, packed)]
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
    /// LFO waveform selection (low 3 bits mask 0x7 = wave type); high 5 bits = drum key number
    pub lfo_wave: u8,
    /// Velocity layer threshold (read by FUN_10004DF0)
    pub vel_threshold: u8,
    /// Pitch offset (signed, -128~+127)
    pub pitch_offset: i8,
    /// Volume offset (signed, -128~+127)
    pub vol_offset: i8,

    // ── pitch fine (2 bytes) ════════════════════════════════════════
    /// Pitch fine: combined = (elem[9]-8)*256 + elem[10]*16 → 12-bit
    pub pitch_fine_h: u8,
    pub pitch_fine_l: u8,

    // ── Pitch EG + Filter (4 bytes) ═══════════════════════════════
    /// Pitch EG Attack rate (0=fastest)
    pub pitch_eg_attack: u8,
    /// Pitch EG Decay/Release rate (0=fastest, range 0-3)
    pub pitch_eg_decay: u8,
    /// Filter cutoff frequency (64=center)
    pub filter_cutoff: u8,
    /// Filter resonance (64=center)
    pub filter_resonance: u8,

    // ── mode/range/type (3 bytes) ══════════════════════════════════
    /// Pitch mode (0=direct addition, 1-4=lookup table)
    pub pitch_mode: u8,
    /// Range base value (60=middle C)
    pub range_base: u8,
    /// Voice type (0=standard poly, 1=default, 2=synthesizer, 3=SFX)
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
    /// PEG Center Note
    pub peg_center_note: u8,

    // ── PEG rates (5 bytes, range 0-127) ═══════════════════════════
    /// PEG Rate / Center (mode=64)
    pub peg_rate0: u8,
    /// PEG Rate 1
    pub peg_rate1: u8,
    /// PEG Rate 2
    pub peg_rate2: u8,
    /// PEG Rate 3
    pub peg_rate3: u8,
    /// PEG Rate 4
    pub peg_rate4: u8,

    // ── DSP synthesis parameters [31..77] ════════════════════════════════════
    /// DSP parameter base index (read by FUN_10013456 → vtable[0x4F0])
    pub dsp_base: u8,
    /// Unused (padding)
    pub _pad32: u8,
    /// Lookup table index → pitch scaling (read by 0x10015887, 2D lookup 0x100473D0)
    pub tbl_index: u8,
    /// Unused
    pub _pad34: u8,
    /// Pitch coarse offset (read by 0x10006D4D → voice[0xB4])
    pub pitch_coarse: u8,
    /// Unused
    pub _pad36: u8,
    pub _pad37: u8,
    pub _pad38: u8,
    pub _pad39: u8,
    /// Filter EG enable (0x10019643: non-zero→triggers the Filter EG chain)
    pub eg_filt_en: u8,
    /// Amp EG enable (0x10007315: non-zero→computes AEG → voice[0x6E])
    pub eg_amp_en: u8,
    /// LFO enable (0x10007354)
    pub lfo_en: u8,
    /// Pitch EG enable (0x100073E6: non-zero→calls the vtable[0x18C] PEG)
    pub eg_pitch_en: u8,
    /// Output enable (0x100074FA)
    pub output_en: u8,
    /// Unused
    pub _pad45: u8,
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
    /// Unused
    pub _pad53: u8,
    /// AEG Decay1 Rate override enable
    pub aeg_d1: u8,
    /// Unused
    pub _pad55: u8,
    /// AEG Decay2 Rate override enable
    pub aeg_d2: u8,
    /// AEG Release Rate override enable
    pub aeg_rel: u8,
    /// Unused
    pub _pad58: u8,
    pub _pad59: u8,
    pub _pad60: u8,
    pub _pad61: u8,
    pub _pad62: u8,
    pub _pad63: u8,
    /// EG rate remap index (0x100142AC: shift left 7 → 2D lookup)
    pub rate_idx: u8,
    /// Unused
    pub _pad65: u8,
    pub _pad66: u8,
    /// Sample format flag (0x10038D6C: 0=8bit, non-zero=16bit)
    pub fmt_flag: u8,
    /// Lookup table index (0x10015834 → word table 0x10048134 → voice[0xE])
    pub tbl_68: u8,
    /// EG stage offset/counter (0x10012550 → voice[0x1C7])
    pub eg_phase: u8,
    /// Sample layer pitch fine (0x100125B0 → voice[0x1CA])
    pub wave_pitch: u8,
    /// EG master enable (0x10012600 → voice[0x64])
    pub eg_enable: u8,
    /// EG delay (0x10012670 → voice[0x69][0x66])
    pub eg_delay: u8,
    /// Trigger/retrigger mode (0x100127A0)
    pub trig_mode: u8,
    /// Override lookup value (0x10014238: non-zero→overrides 0x10047AD8)
    pub alt_ovr: u8,
    /// Sample offset high (0x10015691 → voice[0x1CC])
    pub off_hi: u8,
    /// Sample offset low ([75] low 7 bits)
    pub off_lo: u8,
    /// Sensitivity signed (0x10013530: elem-64 → modulates element[31])
    pub sensitivity: u8,
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
            filter_resonance: value[14],
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
            _pad32: value[32],
            tbl_index: value[33],
            _pad34: value[34],
            pitch_coarse: value[35],
            _pad36: value[36],
            _pad37: value[37],
            _pad38: value[38],
            _pad39: value[39],
            eg_filt_en: value[40],
            eg_amp_en: value[41],
            lfo_en: value[42],
            eg_pitch_en: value[43],
            output_en: value[44],
            _pad45: value[45],
            ovr_cutoff: value[46],
            cs_en_1: value[47],
            cs_en_2: value[48],
            ls_en: value[49],
            ls_store: value[50],
            ls_cmp: value[51],
            ls_flag: value[52],
            _pad53: value[53],
            aeg_d1: value[54],
            _pad55: value[55],
            aeg_d2: value[56],
            aeg_rel: value[57],
            _pad58: value[58],
            _pad59: value[59],
            _pad60: value[60],
            _pad61: value[61],
            _pad62: value[62],
            _pad63: value[63],
            rate_idx: value[64],
            _pad65: value[65],
            _pad66: value[66],
            fmt_flag: value[67],
            tbl_68: value[68],
            eg_phase: value[69],
            wave_pitch: value[70],
            eg_enable: value[71],
            eg_delay: value[72],
            trig_mode: value[73],
            alt_ovr: value[74],
            off_hi: value[75],
            off_lo: value[76],
            sensitivity: value[77],
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
}
