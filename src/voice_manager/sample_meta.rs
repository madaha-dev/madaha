use libmadaha::{
    yxg50::drum_setup::DrumSetupEntry as YXG50DrumSetupEntry, yxg50::pre_voice::Element,
    yxg50::sample_meta::SampleMeta as YXG50SampleMeta,
};

pub trait SampleMetaFactory<T, O> {
    fn new(params: T, sample_meta: O) -> SampleMeta;
}

#[derive(Debug)]
pub struct SampleMeta {
    // 省点内存吧，本来波形文件就很大了，直接用指针就得了
    pub pcm: Option<&'static [f32]>,
    pub loop_point: usize,
    pub loop_length: usize,
    // 基准音高
    base_note: u8,
    // 音高微调
    tone: i8,
    pub end_note: u8,
    pub sample_rate: u8,
    pub base_cent: f32,

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
    pub pitch_fine_h: i8,
    pub pitch_fine_l: i8,

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
            pcm: None,
            loop_point: 0,
            loop_length: 0,
            base_note: 0,
            end_note: 0,
            tone: 0,
            sample_rate: 0x80, // 22050 Hz
            base_cent: 0.0,

            key_min: value.key_min,
            key_max: value.key_max,
            vel_max: value.vel_max,
            vel_min: value.vel_min,
            lfo_wave: value.lfo_wave,
            vel_threshold: value.vel_threshold,
            pitch_offset: value.pitch_offset,
            vol_offset: value.vol_offset,
            pitch_fine_h: value.pitch_fine_h as i8,
            pitch_fine_l: value.pitch_fine_l as i8,
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

impl From<&YXG50DrumSetupEntry> for SampleMeta {
    fn from(value: &YXG50DrumSetupEntry) -> Self {
        Self {
            pcm: None,
            loop_point: value.start_point_offset,
            loop_length: value.loop_length,
            base_note: value.base_key,
            base_cent: to_cent(value.base_key, 0),
            sample_rate: value.sample_rate,
            end_note: value.base_key,
            tone: 0,

            key_min: 0x0D,
            key_max: 0x5B,
            vel_min: 1,
            vel_max: 127,

            lfo_wave: 0,
            vel_threshold: 0,
            pitch_offset: 0,
            vol_offset: 0,
            pitch_fine_h: 0x40,
            pitch_fine_l: 0,
            pitch_eg_attack: 0x40,
            pitch_eg_decay: 0x40,
            filter_cutoff: 0x40,
            filter_resonance: 0x40,
            pitch_mode: 0,
            range_base: 64,
            voice_type: 0,
            note_shift: 0x40,
            detune: 0x40,
            peg_center_low: 0,
            peg_center_high: 0,
            peg_vel_sense_level: 0x40,
            peg_vel_sense_rate: 0x40,
            peg_rate_scaling: 0x40,
            peg_center_note: 64,
            peg_rate0: 0x40,
            peg_rate1: 0x40,
            peg_rate2: 0x40,
            peg_rate3: 0x40,
            peg_rate4: 0x40,
            dsp_base: 0,
            tbl_index: 0,
            pitch_coarse: value.pitch_coarse,
            eg_filt_en: 0,
            eg_amp_en: 1,
            lfo_en: 0,
            eg_pitch_en: 0,
            output_en: 1,
            ovr_cutoff: 0,
            cs_en_1: 0,
            cs_en_2: 0,
            ls_en: 0,
            ls_store: 0,
            ls_cmp: 0,
            ls_flag: 0,
            aeg_d1: 0,
            aeg_d2: 0,
            aeg_rel: 0,
            rate_idx: 0,
            fmt_flag: 0,
            tbl_68: 0,
            eg_phase: 0,
            wave_pitch: 0,
            eg_enable: 1,
            eg_delay: 0,
            trig_mode: 0,
            alt_ovr: 0,
            off_hi: 0,
            off_lo: 0,
            sensitivity: 0,
        }
    }
}

impl SampleMetaFactory<&Element, &YXG50SampleMeta> for SampleMeta {
    fn new(params: &Element, sample_meta: &YXG50SampleMeta) -> SampleMeta {
        let mut sm = Self::from(params);
        sm.pcm = sample_meta.pcm;

        sm.loop_point = sample_meta.start_point_offset;
        sm.loop_length = sample_meta.loop_length;
        sm.base_note = sample_meta.base_key;
        sm.end_note = sample_meta.key_end;
        sm.sample_rate = sample_meta.sample_rate_for_sample;
        sm.tone = sample_meta.tone as i8;
        sm.base_cent = to_cent(sm.base_note, sm.tone);

        sm
    }
}

impl SampleMeta {
    pub fn get_length(&self) -> usize {
        self.loop_point + self.loop_length
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

    pub fn get_coarse_in_cent(&self) -> f32 {
        ((self.pitch_coarse as i32 - 64) * 100) as f32
    }

    /// 采样的基准音分（不含 tone）: base_note × 100
    pub fn get_base_note_cent(&self) -> f32 {
        self.base_note as f32 * 100.0
    }

    /// 采样微调 (WaveEntry[2] tone, 音分)
    pub fn get_tone(&self) -> f32 {
        self.tone as f32
    }

    /// element 音高偏移 (elem[7], signed, 音分)
    pub fn get_pitch_offset(&self) -> f32 {
        self.pitch_offset as f32
    }

    pub fn get_fine_in_cent(&self, vel: u8) -> f32 {
        let d = ((self.pitch_fine_h - 8) as i32) << 8 + (self.pitch_fine_l as i32) << 4;
        if d > 0 {
            ((PITCH_FINE_TABLE_POS[vel.min(127) as usize] as i32 * d) >> 16) as f32
        } else if d < 0 {
            ((PITCH_FINE_TABLE_NEG[vel.min(127) as usize] as i32 * d) >> 16) as f32
        } else {
            0.0
        }
    }
}

#[inline(always)]
fn to_cent(base_note: u8, tone: i8) -> f32 {
    (base_note as i8 * 100 + tone) as f32
}

pub const PITCH_FINE_TABLE_POS: [u16; 128] = [
    0xE019, 0xD737, 0xCE93, 0xC62C, 0xBE03, 0xB618, 0xAE6A, 0xA6F9, 0x9FC4, 0x98CC, 0x9210, 0x8B8E,
    0x8547, 0x7F3A, 0x7964, 0x73C7, 0x6E60, 0x692E, 0x6430, 0x5F66, 0x5ACE, 0x5666, 0x522D, 0x4E22,
    0x4A44, 0x4691, 0x4309, 0x3FA8, 0x3C6F, 0x395B, 0x366C, 0x33A0, 0x30F6, 0x2E6C, 0x2C01, 0x29B4,
    0x2784, 0x2570, 0x2375, 0x2194, 0x1FCC, 0x1E1A, 0x1C7E, 0x1AF7, 0x1984, 0x1825, 0x16D7, 0x159B,
    0x146F, 0x1353, 0x1246, 0x1147, 0x1056, 0x0F71, 0x0E99, 0x0DCC, 0x0D0A, 0x0C53, 0x0BA5, 0x0B01,
    0x0A66, 0x09D3, 0x0948, 0x08C5, 0x0849, 0x07D3, 0x0764, 0x06FB, 0x0698, 0x063A, 0x05E2, 0x058E,
    0x053F, 0x04F4, 0x04AE, 0x046B, 0x042C, 0x03F0, 0x03B8, 0x0383, 0x0351, 0x0321, 0x02F5, 0x02CA,
    0x02A3, 0x027D, 0x0259, 0x0238, 0x0218, 0x01FA, 0x01DE, 0x01C3, 0x01AA, 0x0192, 0x017B, 0x0166,
    0x0152, 0x013F, 0x012D, 0x011C, 0x010C, 0x00FD, 0x00EF, 0x00E2, 0x00D5, 0x00C9, 0x00BE, 0x00B3,
    0x00A9, 0x00A0, 0x0097, 0x008E, 0x0086, 0x007F, 0x0078, 0x0071, 0x006B, 0x0065, 0x005F, 0x005A,
    0x0055, 0x0050, 0x004B, 0x0047, 0x0043, 0x003F, 0x003C, 0x0039,
];

pub const PITCH_FINE_TABLE_NEG: [u16; 128] = [
    0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF, 0xF465,
    0xE187, 0xD08D, 0xC134, 0xB346, 0xA695, 0x9AFD, 0x905D, 0x869B, 0x7D9F, 0x7556, 0x6DAE, 0x6699,
    0x6007, 0x59EF, 0x5446, 0x4F01, 0x4A1A, 0x4587, 0x4144, 0x3D49, 0x3991, 0x3617, 0x32D6, 0x2FCB,
    0x2CF2, 0x2A46, 0x27C5, 0x256C, 0x2338, 0x2127, 0x1F36, 0x1D64, 0x1BAD, 0x1A12, 0x188E, 0x1722,
    0x15CC, 0x148A, 0x135B, 0x123E, 0x1132, 0x1036, 0x0F48, 0x0E68, 0x0D95, 0x0CCF, 0x0C14, 0x0B63,
    0x0ABD, 0x0A21, 0x098E, 0x0903, 0x0880, 0x0804, 0x0790, 0x0722, 0x06BB, 0x0659, 0x05FD, 0x05A7,
    0x0555, 0x0508, 0x04BF, 0x047A, 0x043A, 0x03FD, 0x03C3, 0x038D, 0x035A, 0x0329, 0x02FC, 0x02D1,
    0x02A8, 0x0282, 0x025E, 0x023B, 0x021B, 0x01FD, 0x01E0, 0x01C5, 0x01AC, 0x0194, 0x017D, 0x0168,
    0x0153, 0x0140, 0x012E, 0x011D, 0x010D, 0x00FE, 0x00F0, 0x00E2, 0x00D6, 0x00CA, 0x00BE, 0x00B4,
    0x00AA, 0x00A0, 0x0097, 0x008F, 0x0087, 0x007F, 0x0078, 0x0071, 0x006B, 0x0065, 0x005F, 0x005A,
    0x0055, 0x0050, 0x004B, 0x0047, 0x0043, 0x003F, 0x003C, 0x0039,
];
