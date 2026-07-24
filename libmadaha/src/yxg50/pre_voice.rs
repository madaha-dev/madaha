/// S-YXG50 TBL Element (78 bytes)
///
/// 基于 Ghidra 反编译确认 (S-YXG50.dll), 同步自 note_opencode.md (2026-07-23)
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
    // ── 波形/键位/力度 匹配 (5 bytes) ══════════════════════════════
    /// dataSeg15 索引 (0-245)
    pub index: u8,
    /// 键位下限 (FUN_10017060: key_range 匹配)
    pub key_min: u8,
    /// 键位上限
    pub key_max: u8,
    /// 力度下限
    pub vel_min: u8,
    /// 力度上限
    pub vel_max: u8,

    // ── LFO/力度/音高/音量 (4 bytes) ══════════════════════════════
    /// LFO 波形选择 (低3位 mask 0x7 = wave type); 高5位 = 鼓键号
    pub lfo_wave: u8,
    /// 力度分层阈值 (FUN_10004DF0 读取)
    pub vel_threshold: u8,
    /// 音高偏移 (signed, -128~+127)
    pub pitch_offset: i8,
    /// 音量偏移 (signed, -128~+127)
    pub vol_offset: i8,

    // ── 音高微调 (2 bytes) ════════════════════════════════════════
    /// 音高微调: combined = (elem[9]-8)*256 + elem[10]*16 → 12-bit
    pub pitch_fine_h: u8,
    pub pitch_fine_l: u8,

    // ── Pitch EG + Filter (4 bytes) ═══════════════════════════════
    /// Pitch EG Attack 速率 (0=最快)
    pub pitch_eg_attack: u8,
    /// Pitch EG Decay/Release 速率 (0=最快, 值域 0-3)
    pub pitch_eg_decay: u8,
    /// 滤波器截止频率 (64=中心)
    pub filter_cutoff: u8,
    /// 滤波器共鸣 (64=中心)
    pub filter_resonance: u8,

    // ── 模式/范围/类型 (3 bytes) ══════════════════════════════════
    /// 调音模式 (0=直加, 1-4=查表)
    pub pitch_mode: u8,
    /// 范围基值 (60=中央C)
    pub range_base: u8,
    /// 音色类型 (0=标准poly, 1=默认, 2=合成器, 3=SFX)
    pub voice_type: u8,

    // ── NoteShift + Detune + PEG front (4 bytes) ═════════════════
    /// Note Shift (32-96, 64=中心); 运行时被覆写
    pub note_shift: u8,
    /// Detune (14-114, 64=中心); 运行时被覆写
    pub detune: u8,
    /// PEG Center Note Low
    pub peg_center_low: u8,
    /// PEG Center Note High
    pub peg_center_high: u8,

    // ── PEG 前段 (4 bytes) ═══════════════════════════════════════
    /// PEG Vel Sense Level
    pub peg_vel_sense_level: u8,
    /// PEG Vel Sense Rate
    pub peg_vel_sense_rate: u8,
    /// PEG Rate Scaling
    pub peg_rate_scaling: u8,
    /// PEG Center Note
    pub peg_center_note: u8,

    // ── PEG 速率 (5 bytes, 值域 0-127) ═══════════════════════════
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

    // ── DSP 合成参数 [31..77] ════════════════════════════════════
    /// DSP 参数基索引 (FUN_10013456 读取 → vtable[0x4F0])
    pub dsp_base: u8,
    /// 未使用 (填充)
    pub _pad32: u8,
    /// 查表索引 → 音高缩放 (0x10015887 读取, 2D 查表 0x100473D0)
    pub tbl_index: u8,
    /// 未使用
    pub _pad34: u8,
    /// 音高粗调偏移 (0x10006D4D 读取 → voice[0xB4])
    pub pitch_coarse: u8,
    /// 未使用
    pub _pad36: u8,
    pub _pad37: u8,
    pub _pad38: u8,
    pub _pad39: u8,
    /// Filter EG 使能 (0x10019643: 非0→触发 Filter EG 链)
    pub eg_filt_en: u8,
    /// Amp EG 使能 (0x10007315: 非0→计算 AEG → voice[0x6E])
    pub eg_amp_en: u8,
    /// LFO 使能 (0x10007354)
    pub lfo_en: u8,
    /// Pitch EG 使能 (0x100073E6: 非0→调用 vtable[0x18C] PEG)
    pub eg_pitch_en: u8,
    /// 输出使能 (0x100074FA)
    pub output_en: u8,
    /// 未使用
    pub _pad45: u8,
    /// Cutoff 覆写标志 (0x10007834: 0=默认查表, 非0=覆写)
    pub ovr_cutoff: u8,
    /// Cutoff Scaling 阶段1 使能
    pub cs_en_1: u8,
    /// Cutoff Scaling 阶段2 使能
    pub cs_en_2: u8,
    /// Level Scaling 使能 (0x100196E1 → FUN_1001B630)
    pub ls_en: u8,
    /// Level Scaling 存储
    pub ls_store: u8,
    /// Level Scaling 后处理比较
    pub ls_cmp: u8,
    /// Level Scaling 标志
    pub ls_flag: u8,
    /// 未使用
    pub _pad53: u8,
    /// AEG Decay1 Rate 覆写使能
    pub aeg_d1: u8,
    /// 未使用
    pub _pad55: u8,
    /// AEG Decay2 Rate 覆写使能
    pub aeg_d2: u8,
    /// AEG Release Rate 覆写使能
    pub aeg_rel: u8,
    /// 未使用
    pub _pad58: u8,
    pub _pad59: u8,
    pub _pad60: u8,
    pub _pad61: u8,
    pub _pad62: u8,
    pub _pad63: u8,
    /// EG 速率重映射索引 (0x100142AC: 左移7→2D查表)
    pub rate_idx: u8,
    /// 未使用
    pub _pad65: u8,
    pub _pad66: u8,
    /// 采样格式标志 (0x10038D6C: 0=8bit, 非0=16bit)
    pub fmt_flag: u8,
    /// 查表索引 (0x10015834 → 字表 0x10048134 → voice[0xE])
    pub tbl_68: u8,
    /// EG 阶段偏移/计数器 (0x10012550 → voice[0x1C7])
    pub eg_phase: u8,
    /// 采样层音高微调 (0x100125B0 → voice[0x1CA])
    pub wave_pitch: u8,
    /// EG 总使能 (0x10012600 → voice[0x64])
    pub eg_enable: u8,
    /// EG 延迟 (0x10012670 → voice[0x69][0x66])
    pub eg_delay: u8,
    /// 触发/再触发模式 (0x100127A0)
    pub trig_mode: u8,
    /// 覆写查表值 (0x10014238: 非0→覆写 0x10047AD8)
    pub alt_ovr: u8,
    /// 采样偏移高位 (0x10015691 → voice[0x1CC])
    pub off_hi: u8,
    /// 采样偏移低位 ([75] 低7位)
    pub off_lo: u8,
    /// 灵敏度 signed (0x10013530: elem-64 → 调制 element[31])
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
    /// 组合音高微调: `(elem[9]-8)*256 + elem[10]*16` → 12-bit 值
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
    /// 采样偏移组合: `(off_hi << 7) | off_lo`
    pub fn sample_offset_pair(&self) -> u16 {
        ((self.off_hi as u16) << 7) | self.off_lo as u16
    }
    /// 灵敏度 signed: `value - 64`
    pub fn sensitivity_signed(&self) -> i8 {
        self.sensitivity as i8 - 64
    }
    /// 键位匹配: `min <= note <= max`
    pub fn matches_key(&self, note: u8) -> bool {
        note >= self.key_min && note <= self.key_max
    }
    /// 力度匹配: `min <= vel <= max`
    pub fn matches_vel(&self, vel: u8) -> bool {
        vel >= self.vel_min && vel <= self.vel_max
    }
}
