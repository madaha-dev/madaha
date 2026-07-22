/// S-YXG50 TBL Element (78 bytes)
///
/// 每个 pre-voice 定义包含 1 或 2 个 element。
/// element[i] 在 `voice_base + 2 + i * 78` 处。
///
/// 文件布局:
/// ```text
///  [0..4]  波形/键位/力度 匹配参数
///  [5..10] LFO/力度/音高/音量 偏移
///  [11..14] Pitch EG + Filter
///  [15..17] 模式 + 范围 + 音色类型
///  [18..30] NoteShift + Detune + PEG 参数
///  [31..77] DSP 合成参数 (EG/滤波器/AEG/LFO/输出)
/// ```
#[derive(Debug)]
#[repr(C, packed)]
pub struct Element {
    // ── 波形/键位/力度 匹配 (5 bytes) ──
    /// dataSeg15 索引 (0-245)
    pub index: u8,

    pub key_center: u8,
    pub key_span: u8,

    /// 力度中心
    pub vel_center: u8,
    pub vel_span: u8,

    // ── LFO/力度/音高/音量 (4 bytes) ──
    /// LFO 波形选择 (低3位 mask 0x7); 高5位 = 波形变化/鼓键号
    pub lfo_wave: u8,
    /// 力度分层阈值
    pub vel_threshold: u8,
    /// 音高偏移 (signed, -128~+127)
    pub pitch_offset: i8,
    /// 音量偏移 (signed, -128~+127)
    pub vol_offset: i8,
    // ── 音高微调 (2 bytes) ──
    /// 音高微调: combined = (elem[9]-8) * 256 + elem[10] * 16 → 12-bit 值
    pub pitch_fine_h: u8,
    pub pitch_fine_l: u8,
    // ── Pitch EG + Filter (4 bytes) ──
    /// Pitch EG 起音速率 (0=最快)
    pub pitch_eg_attack: u8,
    /// Pitch EG 衰减速率 (0=最快)
    pub pitch_eg_decay: u8,
    /// 滤波器截止频率 (64=中心)
    pub filter_cutoff: u8,
    /// 滤波器共鸣 (64=中心)
    pub filter_resonance: u8,
    // ── 模式/范围/类型 (3 bytes) ──
    /// 调音模式 (0=直加, 1-4=查表)
    pub pitch_mode: u8,
    /// 范围基值 (60=中央C)
    pub range_base: u8,
    /// 音色类型 (0=标准poly, 1=默认, 2=合成器, 3=SFX)
    pub voice_type: u8,
    // ── NoteShift + Detune + PEG front (4 bytes) ──
    /// Note Shift (32-96, 64=中心); 运行时被覆写为 scratch
    pub note_shift: u8,
    /// Detune (14-114, 64=中心); 运行时被覆写为 scratch
    pub detune: u8,
    /// 疑似 Pitch Center Note / PEG Center; 待确认
    pub unk_20: u8,
    /// 疑似 PEG Center Note; 待确认
    pub unk_21: u8,
    // ── PEG 前段 (4 bytes) ──
    /// PEG Vel Sense Level
    pub peg_vel_sense_level: u8,
    /// PEG Vel Sense Rate
    pub peg_vel_sense_rate: u8,
    /// PEG Rate Scaling
    pub peg_rate_scaling: u8,
    /// PEG Center Note
    pub peg_center_note: u8,
    // ── PEG 速率 (5 bytes, 值域 0-127) ──
    /// PEG Rate / Center (67% mode=64)
    pub peg_center_or_rate1: u8,
    /// PEG Rate 1 (0-127, 97% mode=64)
    pub peg_rate1: u8,
    /// PEG Rate 2
    pub peg_rate2: u8,
    /// PEG Rate 3
    pub peg_rate3: u8,
    /// PEG Rate 4
    pub peg_rate4: u8,
    pub eg_param_31: u8,
    pub unk_32: u8,
    pub tbl_index: u8,
    pub unk_34: u8,
    pub unk_35: u8,
    pub unk_36: u8,
    pub unk_37: u8,
    pub unk_38: u8,
    pub unk_39: u8,
    pub unk_40: u8,
    pub unk_41: u8,
    pub unk_42: u8,
    pub unk_43: u8,
    pub unk_44: u8,
    pub unk_45: u8,
    pub override_flag: u8,
    pub unk_47: u8,
    pub unk_48: u8,
    pub unk_49: u8,
    pub unk_50: u8,
    pub unk_51: u8,
    pub unk_52: u8,
    pub unk_53: u8,
    pub unk_54: u8,
    pub unk_55: u8,
    pub unk_56: u8,
    pub unk_57: u8,
    pub unk_58: u8,
    pub unk_59: u8,
    pub unk_60: u8,
    pub unk_61: u8,
    pub unk_62: u8,
    pub unk_63: u8,
    pub rate_index: u8,
    pub unk_65: u8,
    pub unk_66: u8,
    pub unk_67: u8,
    pub unk_68: u8,
    pub eg_trigger: u8,
    pub eg_hold: u8,
    pub eg_flag: u8,
    pub eg_delay: u8,
    pub unk_73: u8,
    pub alt_override: u8,
    pub offset_hi: u8,
    pub offset_lo: u8,
    pub sensitivity: u8,
}

impl From<&[u8; 78]> for Element {
    fn from(value: &[u8; 78]) -> Self {
        Self {
            index: value[0],
            key_center: value[1],
            key_span: value[2],
            vel_center: value[3],
            vel_span: value[4],
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
            unk_20: value[20],
            unk_21: value[21],
            peg_vel_sense_level: value[22],
            peg_vel_sense_rate: value[23],
            peg_rate_scaling: value[24],
            peg_center_note: value[25],
            peg_center_or_rate1: value[26],
            peg_rate1: value[27],
            peg_rate2: value[28],
            peg_rate3: value[29],
            peg_rate4: value[30],
            eg_param_31: value[31],
            unk_32: value[32],
            tbl_index: value[33],
            unk_34: value[34],
            unk_35: value[35],
            unk_36: value[36],
            unk_37: value[37],
            unk_38: value[38],
            unk_39: value[39],
            unk_40: value[40],
            unk_41: value[41],
            unk_42: value[42],
            unk_43: value[43],
            unk_44: value[44],
            unk_45: value[45],
            override_flag: value[46],
            unk_47: value[47],
            unk_48: value[48],
            unk_49: value[49],
            unk_50: value[50],
            unk_51: value[51],
            unk_52: value[52],
            unk_53: value[53],
            unk_54: value[54],
            unk_55: value[55],
            unk_56: value[56],
            unk_57: value[57],
            unk_58: value[58],
            unk_59: value[59],
            unk_60: value[60],
            unk_61: value[61],
            unk_62: value[62],
            unk_63: value[63],
            rate_index: value[64],
            unk_65: value[65],
            unk_66: value[66],
            unk_67: value[67],
            unk_68: value[68],
            eg_trigger: value[69],
            eg_hold: value[70],
            eg_flag: value[71],
            eg_delay: value[72],
            unk_73: value[73],
            alt_override: value[74],
            offset_hi: value[75],
            offset_lo: value[76],
            sensitivity: value[77],
        }
    }
}

impl Element {
    /// 组合音高微调: `(elem[9]-8) * 256 + elem[10] * 16`
    pub fn pitch_fine(&self) -> i16 {
        ((self.pitch_fine_h as i16 - 8) * 256) + (self.pitch_fine_l as i16 * 16)
    }
    /// LFO 波形选择 (低3位)
    pub fn lfo_waveform(&self) -> u8 {
        self.lfo_wave & 0x07
    }
    /// LFO 波形变化/鼓键号 (高5位)
    pub fn lfo_wave_variation(&self) -> u8 {
        self.lfo_wave >> 3
    }
    /// 采样负偏移: `(synth_params[44] << 7) | synth_params[45]`
    pub fn sample_offset_pair(&self) -> u16 {
        ((self.offset_hi as u16) << 7) | self.offset_lo as u16
    }
    /// 灵敏度 (signed): `synth_params[46] as i8 - 64`
    pub fn sensitivity(&self) -> i8 {
        self.sensitivity as i8 - 64
    }
    /// 匹配公式: `abs(note - center) <= span`
    pub fn matches_key(&self, note: u8) -> bool {
        (note as i16 - self.key_center as i16).unsigned_abs() <= self.key_span as u16
    }
    /// 匹配公式: `abs(vel - center) <= span`
    pub fn matches_vel(&self, vel: u8) -> bool {
        (vel as i16 - self.vel_center as i16).unsigned_abs() <= self.vel_span as u16
    }
}
