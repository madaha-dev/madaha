use libmadaha::{
    SoundWave, yxg50::pre_voice::Element, yxg50::sample_meta::SampleMeta as YXG50SampleMeta,
};

pub trait SampleMetaFactory<T, O> {
    fn new(params: T, sample_meta: O) -> SampleMeta;
}

#[derive(Debug)]
pub struct SampleMeta {
    pub start_offset: usize,
    pub loop_start: usize,
    pub loop_length: usize,
    // 基准音高
    pub base_note: u8,
    pub end_note: u8,

    /// 键位下限 (FUN_10017060: key_range 匹配)
    pub key_min: u8,
    /// 键位上限
    pub key_max: u8,
    /// 力度下限
    pub vel_min: u8,
    /// 力度上限
    pub vel_max: u8,

    // some parameters
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
    /// 查表索引 → 音高缩放 (0x10015887 读取, 2D 查表 0x100473D0)
    pub tbl_index: u8,
    /// 音高粗调偏移 (0x10006D4D 读取 → voice[0xB4])
    pub pitch_coarse: u8,
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
    /// AEG Decay1 Rate 覆写使能
    pub aeg_d1: u8,
    /// AEG Decay2 Rate 覆写使能
    pub aeg_d2: u8,
    /// AEG Release Rate 覆写使能
    pub aeg_rel: u8,
    /// EG 速率重映射索引 (0x100142AC: 左移7→2D查表)
    pub rate_idx: u8,
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

// For S-YXG50
impl From<&Element> for SampleMeta {
    fn from(value: &Element) -> Self {
        Self {
            start_offset: 0,
            loop_start: 0,
            loop_length: 0,
            base_note: 0,
            end_note: 0,

            key_min: value.key_min,
            key_max: value.key_max,
            vel_max: value.vel_max,
            vel_min: value.vel_min,
            lfo_wave: value.lfo_wave,
            vel_threshold: value.vel_threshold,
            pitch_offset: value.pitch_offset,
            vol_offset: value.vol_offset,
            pitch_fine_h: value.pitch_fine_h,
            pitch_fine_l: value.pitch_fine_l,
            pitch_eg_attack: value.pitch_eg_attack,
            pitch_eg_decay: value.pitch_eg_decay,
            filter_cutoff: value.filter_cutoff,
            filter_resonance: value.filter_resonance,
            pitch_mode: value.pitch_mode,
            range_base: value.range_base,
            voice_type: value.voice_type,
            note_shift: value.note_shift,
            detune: value.detune,
            peg_center_low: value.peg_center_low,
            peg_center_high: value.peg_center_high,
            peg_vel_sense_level: value.peg_vel_sense_level,
            peg_vel_sense_rate: value.peg_vel_sense_rate,
            peg_rate_scaling: value.peg_rate_scaling,
            peg_center_note: value.peg_center_note,
            peg_rate0: value.peg_rate0,
            peg_rate1: value.peg_rate1,
            peg_rate2: value.peg_rate2,
            peg_rate3: value.peg_rate3,
            peg_rate4: value.peg_rate4,
            dsp_base: value.dsp_base,
            tbl_index: value.tbl_index,
            pitch_coarse: value.pitch_coarse,
            eg_filt_en: value.eg_filt_en,
            eg_amp_en: value.eg_amp_en,
            lfo_en: value.lfo_en,
            eg_pitch_en: value.eg_pitch_en,
            output_en: value.output_en,
            ovr_cutoff: value.ovr_cutoff,
            cs_en_1: value.cs_en_1,
            cs_en_2: value.cs_en_2,
            ls_en: value.ls_en,
            ls_store: value.ls_store,
            ls_cmp: value.ls_cmp,
            ls_flag: value.ls_flag,
            aeg_d1: value.aeg_d1,
            aeg_d2: value.aeg_d2,
            aeg_rel: value.aeg_rel,
            rate_idx: value.rate_idx,
            fmt_flag: value.fmt_flag,
            tbl_68: value.tbl_68,
            eg_phase: value.eg_phase,
            wave_pitch: value.wave_pitch,
            eg_enable: value.eg_enable,
            eg_delay: value.eg_delay,
            trig_mode: value.trig_mode,
            alt_ovr: value.alt_ovr,
            off_hi: value.off_hi,
            off_lo: value.off_lo,
            sensitivity: value.sensitivity,
        }
    }
}

impl SampleMetaFactory<&Element, &YXG50SampleMeta> for SampleMeta {
    fn new(params: &Element, sample_meta: &YXG50SampleMeta) -> SampleMeta {
        let mut sm = Self::from(params);
        sm.start_offset = sample_meta.start_point_offset;
        sm.loop_length = sample_meta.loop_length;
        sm.loop_start = sample_meta.loop_start;
        sm.base_note = sample_meta.base_key;
        sm.end_note = sample_meta.key_end;

        sm
    }
}

impl SampleMeta {
    pub fn get_start(&self) -> usize {
        self.loop_start - self.start_offset
    }

    pub fn get_length(&self) -> usize {
        self.start_offset + self.loop_length
    }

    pub fn get_sample(&self, sw: &'static SoundWave) -> Option<&[u8]> {
        let start = self.get_start();
        let end = start + self.get_length();

        sw.get(start..end)
    }

    pub fn check_key(&self, note: u8) -> bool {
        note <= (self.end_note & 0x7F)
    }

    pub fn fill_velocity(&'static self) -> [Option<&'static Self>; 128] {
        let mut data = [None; 128];
        for i in self.vel_min..=self.vel_max {
            data[i as usize] = Some(self);
        }
        data
    }
}
