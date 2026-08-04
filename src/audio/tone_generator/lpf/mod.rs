/// 低通滤波器 (DCF, Digital Controlled Filter)
///
/// 实现: 双极点状态变量滤波器 (SVF, Andrew Simper 稳定版)
/// 参数:
///   cutoff:  截止频率 (Hz), 由 VCE base + Part 相对偏移 + FEG 调制而来
///   resonance: Q 值 (0.5 - 10)
///
/// 对齐说明 (S-YXG50 数据):
/// - `SampleMeta.filter_cutoff` (Element[13], 64=中心) → base cutoff
/// - `SampleMeta.filter_resonance` (Element[14], 64=中心) → Q
/// - Part 08 pp 18/19 (Filter Cutoff/Resonance 相对偏移) → note-on 快照
/// - FEG (Filter EG) + LFO.lpf 输出 → 调制 cutoff 参数
pub mod feg;

pub use feg::{FEG, FEGStage};

#[derive(Debug)]
pub struct LPF {
    /// 截止频率 (Hz)
    pub cutoff: f32,
    /// Q 值
    pub resonance: f32,

    // SVF 状态
    ic1eq: f32,
    ic2eq: f32,
    // 系数缓存 (cutoff/resonance 变化时重算)
    a0: f32,
    a1: f32,
    k: f32,
}

impl LPF {
    pub fn new() -> Self {
        Self {
            cutoff: 1000.0,
            resonance: 1.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            a0: 0.0,
            a1: 0.0,
            k: 0.0,
        }
    }

    /// 设置参数并重算系数 (cutoff/resonance 变化时调用)
    pub fn set_params(&mut self, cutoff_hz: f32, q: f32, sample_rate: f32) {
        self.cutoff = cutoff_hz.max(1.0);
        self.resonance = q.max(0.1);
        let w = 2.0 * std::f32::consts::PI * self.cutoff / sample_rate;
        let g = w.tan();
        self.k = 1.0 / self.resonance;
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a0 = g;
    }

    /// 处理一个采样, 返回低通输出
    pub fn tick(&mut self, input: f32) -> f32 {
        let v3 = input - self.ic2eq;
        let v1 = self.a1 * self.ic1eq + self.a0 * v3;
        let v2 = self.ic2eq + self.a1 * (self.a0 * v1);
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        v2
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// 0-127 参数 → 截止频率 (Hz), 对数映射 100Hz - 12kHz
    pub fn cutoff_param_to_hz(param: u8) -> f32 {
        let t = (param & 0x7F) as f32 / 127.0;
        100.0 * (120.0f32).powf(t)
    }

    /// 0-127 参数 → Q 值 (0.5 - 10)
    pub fn resonance_param_to_q(param: u8) -> f32 {
        0.5 + (param & 0x7F) as f32 / 127.0 * 9.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_mapping() {
        assert!((LPF::cutoff_param_to_hz(0) - 100.0).abs() < 0.1);
        assert!((LPF::cutoff_param_to_hz(127) - 12000.0).abs() < 1.0);
        assert!((LPF::cutoff_param_to_hz(64) - 100.0 * 120.0f32.powf(64.0 / 127.0)).abs() < 0.1);
    }

    #[test]
    fn lpf_passes_dc() {
        // DC 信号应无损通过 (截止 10kHz)
        let mut lpf = LPF::new();
        lpf.set_params(10000.0, 1.0, 44100.0);
        let mut out = 0.0;
        for _ in 0..100 {
            out = lpf.tick(1.0);
        }
        assert!((out - 1.0).abs() < 1e-3, "DC gain = {out}");
    }

    #[test]
    fn lpf_attenuates_high_freq() {
        // 10kHz 方波过 100Hz 低通 → 大幅衰减
        let mut lpf = LPF::new();
        lpf.set_params(100.0, 0.5, 44100.0);
        let mut out = 0.0;
        for i in 0..4410 {
            let input = if (i / 4) % 2 == 0 { 1.0 } else { -1.0 };
            out = lpf.tick(input);
        }
        assert!(out.abs() < 0.1, "output={out}");
    }
}

/// CutOff 截止频率计算
///
/// 参数域 (0-127) 加法 → 对数频率 (乘性调制, 音频上自然):
/// ```
/// cutoff_param = base (VCE filter_cutoff)
///              + part_offset (08 pp 18, 64=中心)
///              + FEG.level × feg_depth (Filter EG Depth, 08 pp 71)
///              + LFO.lpf.output (LFO FMOD)
/// cutoff_hz = param_to_hz(clamp(cutoff_param, 0, 127))
/// ```
#[derive(Debug)]
pub struct CutOff {
    /// VCE base cutoff 参数 (0-127)
    pub base: f32,
    /// Part 08 pp 18 相对偏移 (64=0)
    pub part_offset: f32,
    /// FEG 调制深度 (08 pp 71, 64=0 → 无影响)
    pub feg_depth: f32,
    /// LFO 对 cutoff 的调制深度 (0-127, 0=无影响)
    pub lfo_depth: f32,
    /// 外部调制 (MW/Bend/CAT/PAT filter control), param 单位, 每块更新
    pub mod_offset: f32,
}

impl CutOff {
    pub fn new() -> Self {
        Self {
            base: 64.0,
            part_offset: 0.0,
            feg_depth: 0.0,
            lfo_depth: 0.0,
            mod_offset: 0.0,
        }
    }

    /// 计算截止频率 (Hz), 每 block 调用
    pub fn compute_hz(&self, feg_level: f32, lfo_lpf: f32) -> f32 {
        let mut param = self.base + self.part_offset;
        param += feg_level * self.feg_depth;
        param += lfo_lpf * self.lfo_depth;
        param += self.mod_offset;
        LPF::cutoff_param_to_hz(param.round().clamp(0.0, 127.0) as u8)
    }

    /// FEG 深度参数 (08 pp 71, 0-127, 64=0) → 调制范围 (param 单位)
    pub fn feg_depth_param(param: u8) -> f32 {
        param as f32 - 64.0 // -64..+63 param 单位
    }

    /// LFO FMOD 深度 (0-127) → cutoff param 调制范围 (±40 param ≈ 频率 ~8 倍跨度)
    pub fn lfo_depth_param(param: u8) -> f32 {
        param as f32 / 127.0 * 40.0
    }
}
