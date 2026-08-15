/// Low-pass filter (DCF, Digital Controlled Filter)
///
/// Implementation: Chamberlin two-pole state-variable filter, aligned with the
/// S-YXG2006LE reference (`CDCFUnit::Generate`, verified in Ghidra x86-32-cpu0x3):
///
/// ```text
///   y1 = (x − K·y1 − y2)·f + y1      (band-pass state)
///   y2 = y1·f + y2                    (low-pass state, output)
///   K  = max(2 − f, 3 − 2f)           (frequency-dependent damping)
///   K  = min(K, k_min)                (k_min from ExchangeResonanceToLinear)
/// ```
///
/// Alignment (2026-08-15, Mac 母本核实)：
/// - **cutoff → f**：`f = ExchangeCutoffToLinear(AdjustRegCutoffToEGCnt(cutoff−0x40)
///   + LPF_CUTOFF_LIMIT_TBL[0x20−resonance]<<8)`（定点 log→linear，f ≈ 0.85~1.0），
///   **非** 旧的 `2·sin(π·fc/fs)`（100Hz-12kHz 对数）。`cutoff_param_to_f` 实现之。
/// - **f 块内滑动**：母本在 64 样本块内逐样本滑 `f += (target−current)/64`。
/// - **共鸣 → k_min**：`ExchangeResonanceToLinear`（0→1.0/64→4.0/127→15.75）。
/// - ⚠ S-YXG50 的 `elem[13] filter_cutoff` 语义（采样率截止）与 2006LE LPF 不同，
///   此处对齐的是 **2006LE LPF** 母本；S-YXG50 cutoff 的映射见 `CutOff` 说明。
use std::f32::consts::PI;

pub mod feg;

pub use feg::FEG;

/// 2006LE `AdjustRegCutoffToEGCnt`：cutoff（signed，0x40 中心）→ 定点
/// `(cutoff << 12) / 75`。
pub fn adjust_reg_cutoff_to_eg_cnt(cutoff: i16) -> i32 {
    ((cutoff as i32) << 12) / 75
}

/// 2006LE `ExchangeCutoffToLinear`：定点（log2 域：byte2=指数、byte1=尾数）→ f。
/// ```text
/// f = (mantissa + 0x100) / 2^(base − exp)，base = 0x18（full）/0x17（symmetric）
/// clamp ≤ 1.0（full）/1.75（symmetric）
/// ```
pub fn exchange_cutoff_to_linear(value: i32, symmetric: bool) -> f32 {
    let exp = (value >> 16) as i8 as i32;
    let mantissa = (value >> 8) & 0xff;
    let base = if symmetric { 0x17 } else { 0x18 };
    let shift = (base - exp) & 0x1f;
    let f = (mantissa + 0x100) as f32 / (1i32 << shift) as f32;
    f.min(if symmetric { 1.75 } else { 1.0 })
}

/// 2006LE LPF cutoff character limit（`_VTG_SWP50_LPF_CUTOFF_LIMIT_TBL`，
/// 索引 `0x20 − resonance`，resonance 0-127）。
fn lpf_cutoff_character_limit(resonance: i16) -> i32 {
    // sVar1 = 0x20 − resonance，0..0x20（resonance 0..32）的 dump 值；resonance > 32 恒 0x0fc0。
    const POS: [u16; 33] = [
        0x0ef8, 0x0efd, 0x0f02, 0x0f07, 0x0f0b, 0x0f11, 0x0f17, 0x0f1c, 0x0f21, 0x0f26, 0x0f2b,
        0x0f2f, 0x0f33, 0x0f3a, 0x0f40, 0x0f46, 0x0f4c, 0x0f53, 0x0f5a, 0x0f61, 0x0f67, 0x0f70,
        0x0f79, 0x0f81, 0x0f89, 0x0f95, 0x0f9e, 0x0fa6, 0x0fb0, 0x0fbc, 0x0fc8, 0x0fd4, 0x0fdf,
    ];
    let svar1 = 0x20 - resonance as i32;
    let v = if svar1 >= 0 {
        POS.get(svar1 as usize).copied().unwrap_or(0x0fc0)
    } else {
        0x0fc0
    };
    (v as i32) << 8
}

/// 2006LE cutoff param（0-127，0x40 中心）→ f（LPF）。
/// `f = ExchangeCutoffToLinear(AdjustRegCutoffToEGCnt(param−0x40) + limit(resonance))`
pub fn cutoff_param_to_f(param: f32, resonance: f32) -> f32 {
    let cutoff = (param - 64.0).round() as i16;
    let value =
        adjust_reg_cutoff_to_eg_cnt(cutoff) + lpf_cutoff_character_limit(resonance.round() as i16);
    exchange_cutoff_to_linear(value, false)
}

#[derive(Debug)]
pub struct LPF {
    /// 当前 f 系数（Chamberlin）
    pub cutoff: f32,
    /// Resonance 参数 (0-127, 64 = 中心)
    pub resonance: f32,

    // Chamberlin SVF 状态
    ic1eq: f32, // band-pass state (y1)
    ic2eq: f32, // low-pass state (y2, output)

    // 块内滑动（母本 64 样本块）
    f_target: f32,
    f_step: f32,
    sweep_left: u32,
    k_min: f32, // 阻尼 K 上限（ExchangeResonanceToLinear）
}

impl LPF {
    pub fn new() -> Self {
        Self {
            cutoff: 0.875,
            resonance: 64.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            f_target: 0.875,
            f_step: 0.0,
            sweep_left: 0,
            k_min: 4.0,
        }
    }

    /// 按 cutoff 参数（0-127）+ resonance（0-127）设置目标 f 与 k_min。
    /// f 在后续 64 个样本内逐样本滑到目标（母本 Generate 语义）。
    pub fn set_params(&mut self, cutoff_param: f32, resonance: f32) {
        self.resonance = resonance;
        let f_target = cutoff_param_to_f(cutoff_param, resonance);
        self.f_target = f_target;
        self.f_step = (f_target - self.cutoff) / 64.0;
        self.sweep_left = 64;
        self.k_min = exchange_resonance_to_linear(resonance as i16);
    }

    /// 处理一个样本，返回低通输出
    pub fn tick(&mut self, input: f32) -> f32 {
        // f 块内滑动
        if self.sweep_left > 0 {
            self.cutoff += self.f_step;
            self.sweep_left -= 1;
        } else {
            self.cutoff = self.f_target;
        }
        let f = self.cutoff;
        // K = max(2 − f, 3 − 2f)，再 min(K, k_min)
        let k_auto = (2.0 - f).max(3.0 - 2.0 * f);
        let k = k_auto.min(self.k_min).max(0.1);
        self.ic1eq = (input - k * self.ic1eq - self.ic2eq) * f + self.ic1eq;
        self.ic2eq = self.ic1eq * f + self.ic2eq;
        self.ic2eq
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
        self.cutoff = self.f_target;
        self.sweep_left = 0;
    }

    /// 0-127 参数 → cutoff 频率 (Hz)，对数映射 100Hz - 12kHz。
    /// 保留给 HPF 等旧路径；LPF 已改用 `cutoff_param_to_f`。
    pub fn cutoff_param_to_hz(param: u8) -> f32 {
        let t = (param & 0x7F) as f32 / 127.0;
        100.0 * (120.0f32).powf(t)
    }

    /// 0-127 参数 → resonance（0-127，保留 API 兼容）
    pub fn resonance_param_to_q(param: u8) -> f32 {
        (param & 0x7F) as f32
    }
}

/// 2006LE `ExchangeResonanceToLinear`: resonance 参数 → 阻尼上限。
/// 定点 log→linear（x86-32-cpu0x3 @ 0005bef2 核实）：
///
/// ```text
/// t = 0x20 − param
/// result = (0x40 − (t & 0x1f)) / 2^(((t >> 5) as i8 + 5) & 0x1f)
/// ```
pub(crate) fn exchange_resonance_to_linear(param: i16) -> f32 {
    let t = 0x20i32 - param as i32;
    if param as i32 <= -0xdf {
        return 0.0;
    }
    let man = 0x40 - (t & 0x1f);
    let exp = ((((t >> 5) as i8 as i32) + 5) & 0x1f) as u32;
    man as f32 / (1i32 << exp) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_param_to_f_neutral() {
        // 中性（param 64、resonance 64）→ limit = 0x0fc000 → f = 0.875
        assert!((cutoff_param_to_f(64.0, 64.0) - 0.875).abs() < 1e-3);
        // param 0（最低）→ 更暗
        assert!(cutoff_param_to_f(0.0, 64.0) < 0.875);
        // param 127（最高）→ 更亮
        assert!(cutoff_param_to_f(127.0, 64.0) > 0.875);
        // 单调
        assert!(cutoff_param_to_f(0.0, 64.0) < cutoff_param_to_f(64.0, 64.0));
        assert!(cutoff_param_to_f(64.0, 64.0) < cutoff_param_to_f(127.0, 64.0));
    }

    #[test]
    fn exchange_cutoff_to_linear_full() {
        // 0x0fc000 → exp 15、mantissa 0xc0 → f = (192+256)/2^9 = 0.875
        assert!((exchange_cutoff_to_linear(0x0fc000, false) - 0.875).abs() < 1e-3);
        // 0x10c000 → exp 16、mantissa 0xc0 → f = 448/256 = 1.75 → clamp 1.0（full）
        assert!((exchange_cutoff_to_linear(0x10c000, false) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn lpf_passes_dc() {
        // DC 信号应无损通过
        let mut lpf = LPF::new();
        lpf.set_params(127.0, 64.0);
        lpf.reset();
        let mut out = 0.0;
        for _ in 0..200 {
            out = lpf.tick(1.0);
        }
        assert!((out - 1.0).abs() < 1e-3, "DC gain = {out}");
    }

    #[test]
    fn lpf_attenuates_high_freq() {
        // 低频（接近 DC）几乎无损，高频（Nyquist）被衰减
        let mut lpf = LPF::new();
        lpf.set_params(64.0, 64.0);
        lpf.reset();
        for _ in 0..100 {
            lpf.tick(1.0);
        }
        let dc = lpf.tick(1.0);
        lpf.reset();
        let mut high = 0.0;
        for i in 0..4410 {
            let input = if i % 2 == 0 { 1.0 } else { -1.0 }; // Nyquist
            high = lpf.tick(input);
        }
        assert!(high.abs() < dc.abs() - 0.1, "dc={dc} high={high}");
    }
}

/// CutOff cutoff 参数计算
///
/// 参数域（0-127）加法 → cutoff 参数（0-127）：
/// ```
/// cutoff_param = base (VCE filter_cutoff)
///              + part_offset (08 pp 18, 64 = center)
///              + FEG.level × feg_depth (Filter EG Depth, 08 pp 71)
///              + LFO.lpf.output (LFO FMOD)
/// cutoff_f = LPF::cutoff_param_to_f(clamp(cutoff_param, 0, 127), resonance)
/// ```
#[derive(Debug)]
pub struct CutOff {
    /// VCE base cutoff parameter (0-127)
    pub base: f32,
    /// Part 08 pp 18 relative offset (64=0)
    pub part_offset: f32,
    /// FEG modulation depth (08 pp 71, 64=0 → no effect)
    pub feg_depth: f32,
    /// LFO modulation depth on cutoff (0-127, 0=no effect)
    pub lfo_depth: f32,
    /// External modulation (MW/Bend/CAT/PAT filter control), in param units, updated each block
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

    /// Compute cutoff 参数 (0-127)，调用者经 `LPF::cutoff_param_to_f` 转 f。
    pub fn compute_param(&self, feg_level: f32, lfo_lpf: f32) -> f32 {
        let mut param = self.base + self.part_offset;
        param += feg_level * self.feg_depth;
        param += lfo_lpf * self.lfo_depth;
        param += self.mod_offset;
        param.round().clamp(0.0, 127.0)
    }

    /// FEG depth 参数 (08 pp 71, 0-127, 64=0) → 调制范围（参数单位）
    pub fn feg_depth_param(param: u8) -> f32 {
        param as f32 - 64.0 // -64..+63 param units
    }

    /// LFO FMOD depth (0-127) → cutoff param 调制范围（±40 param ≈ frequency ~8x span）
    pub fn lfo_depth_param(param: u8) -> f32 {
        param as f32 / 127.0 * 40.0
    }
}

// 保留 PI 引用（cutoff_param_to_hz 不再用 sin，但 HPF 等路径可能用）
#[allow(dead_code)]
fn _pi_used() -> f32 {
    PI
}
