# Element 消费对齐矩阵（S-YXG50 引擎 vs madaha）

状态：2026-08-12。依据 note_opencode.md 逐项核实表（78 字段全部裁决）。

## 对齐分类

| 分类 | 含义 | 数量 |
|------|------|------|
| ✅ 已对齐 | madaha 用法 = 引擎行为 | 12 |
| ⚠ 错误 | madaha 用法与引擎证据冲突 | 6 |
| 🔶 部分 | 双重用途字段，madaha 用对一半 | 3 |
| ❌ 缺失 | 引擎读取，madaha 未实现 | ~25 |
| — 未使用 | 引擎不读，madaha 也不该用 | 5（[17]/[25]/[26]/[30] + 无）|

## 逐字段矩阵

### ✅ 已对齐（12）

| 偏移 | 字段 | 引擎行为（证据） | madaha 用法 |
|:----:|------|------|------|
| [0] | seg15_idx | FUN_10004DF0 波形索引 | wave lookup |
| [1..4] | key/vel 区 | FUN_10017060 匹配 | Element 匹配 |
| [5] | lfo_wave | FUN_100155B0 mask 7 → LFO 表 | lfo_wave & 0x07 |
| [6] | vel_threshold | FUN_100155B0 → engine 0x1c6 | 力度阈值（另：音高公式分量，见 [6] 注）|
| [8] | vol_offset | FUN_100130a0 音量偏移 | vol_offset_gain |
| [13] | filter_cutoff | FUN_10014190 → note final_note | CutOff.base |
| [40] | eg_filt_en | 0x10019643 FEG 使能 | feg.enabled |
| [41] | eg_amp_en | 0x10007315 AEG 使能 | aeg.enabled |
| [42] | lfo_en | 0x10007354 | lfo.enable |
| [43] | eg_pitch_en | 0x100073E6 | peg.enabled |
| [71] | eg_enable | FUN_10012210 | eg_total |
| [72] | key_on_delay | FUN_10012210 | aeg delay |

### ⚠ 错误（6，Phase 1 修正）

| 偏移 | 字段 | 引擎实际 | madaha 当前 | 修正 |
|:----:|------|------|------|------|
| [14] | (曾名 filter_resonance) | **音高分量**（0x100154cd，FUN_10015460 公式）| LPF 共鸣 K bound | 共鸣 → 中性 64；音高 → Phase 2 |
| [25] | peg_center_note | **未读**（12+ KeyOn）| PEG 键位缩放基准 | 停用 |
| [26] | peg_rate0 | **未读** | PEG stage1 速率 | 停用（stage1 ← [27]）|
| [30] | peg_rate4 | **未读** | PEG release 速率 | 停用（release → 中性）|
| [54] | aeg_d1 | voice[0x60] **flag ≥64** | 覆盖 decay_time | flag 化 |
| [56] | aeg_d2 | flag + 曲线B x0（非 sustain）| `1−d2/127` sustain | 移除近似 |

### 🔶 部分（3）

| 偏移 | 字段 | 引擎双重用途 | madaha 取用 |
|:----:|------|------|------|
| [44] | output_en | 输出使能（0x100074FA）+ 键跟深度（0x10013e24）| 输出使能 ✓ |
| [55] | aeg_d1_val | voice[0x61] 参数 + 音量力度缩放（0x100156e7）| 未用（缺）|
| [57] | aeg_rel | 曲线B x1 + voice[0x68] release 参数（0x100061A3）| release_time ✓-ish |
| [67] | fmt_flag | 格式标志（0x10038D6C）+ 键跟基准键（0x1001577e）| 未用 |

### ❌ 缺失（引擎读、madaha 未实现，Phase 2）

| 偏移 | 字段 | 引擎行为 |
|:----:|------|------|
| [7] | pitch_offset | 音高公式 signed（0x100154c0）+ 速率阈值（0x1001494d）|
| [9..10] | pitch_fine | 音高微调（FUN_100130a0 @0x10013120/4b；12-bit 组合）|
| [11] | pitch_eg_attack | EG 速率比较（0x1001488c，表 0x10047BD8）|
| [12] | pitch_eg_decay | EG 速率钳制（0x10014a18）|
| [15..16] | pitch_mode/range_base | ElementCalc_Pitch（0x100140f0）|
| [18] | note_shift | 力度缩放双极性偏移（0x10016044）→ voice[0x51] |
| [19] | detune | 力度缩放双极性偏移（0x10015f94）→ voice[0x50] |
| [20..21] | keyfol_c/d | 键跟随深度/基准（FUN_10015f60）→ voice[0x4f] |
| [22] | peg_vel_sense_level | 0x10015d2f |
| [23] | peg_vel_sense_rate | PEG 状态机条件读（0x10015b3f）|
| [24] | peg_rate_scaling | PEG 状态机条件读（0x10015b79）|
| [27] | peg_rate1 | PEG 状态机（0x10015d82/0x10015b35）|
| [28] | peg_rate2 | PEG 状态机（0x10015b38/0x10015b51）|
| [29] | peg_rate3 | PEG 状态机（0x10015b69）|
| [31] | dsp_base | 0x10013456 |
| [32] | cutoff_mod | 截止调制深度（0x1001427d）|
| [33] | tbl_index | 表 0x100473D0 基值（0x10015887）|
| [34..41] | 曲线 A | EG 段 2/3 log 域速率（DSP reg 0x440/0x5c1）|
| [45] | keyfol_ref | 键跟随基准（0x10013e34）|
| [46] | ovr_cutoff | 截止覆写（0x10007834）|
| [47..53] | cs/ls 链 | FUN_10013bb0/0x100196E1 等 |
| [58..63] | 曲线 B | 音量键缩（FUN_10016490）|
| [64] | rate_idx | 2D 表索引（0x100142AC）|
| [65] | tbl65_idx | 表 0x100480A0（FUN_10015940）|
| [66] | keyfollow_depth | 键跟深度（0x10015783）|
| [68] | tbl_68 | 表 0x10048134 → voice[0xE]（0x10015834）|
| [69] | eg_phase | FUN_10012210 → engine 0x1c7 |
| [70] | wave_pitch | FUN_10012210 → engine 0x1ca |
| [73] | trig_mode | 触发比率（0x100127A0）|
| [74] | alt_ovr | 截止表覆写（0x10014238）|
| [75..76] | off_hi/lo | 14-bit 组合（0x10015690）|
| [77] | sensitivity | 调制灵敏度（0x10013530）|

### — 引擎未读（madaha 也不该用）

| 偏移 | 字段 | 证据 |
|:----:|------|------|
| [17] | voice_type | rwatch 12+ KeyOn 无读取 |
| [25] | peg_center_note | 同（madaha 错误使用中，Phase 1 停用）|
| [26] | peg_rate0 | 同（madaha 错误使用中，Phase 1 停用）|
| [30] | peg_rate4 | 同（madaha 错误使用中，Phase 1 停用）|

## 引擎关键公式（Phase 2 实现依据）

### 音高公式（FUN_10015460，vtable+0x578）
```
pitch = (elem[6] − wave.baseKey) × 0x64 + wave.tone        ; elem[6] signed
      + 表值（elem[1]%12 → voice[0x20]+0x3a 表；engine[0x63a6]）
      − 0x40
      + elem[14]                                            ; 无符号加
      + elem[7] − 0x40                                      ; signed
      → voice[0x10]（word）→ engine[0x1c4]
```

### 键跟随
```
A: voice[0x57] = (key − elem[45]) × (elem[44] − 0x40) / 16      ; FUN_10013e20
B: voice[0x65] = (key − elem[67]) × (elem[66] − 0x40) × 16 >> 8 ; FUN_10015770
C: voice[0x4f] = (key − elem[21]) × (elem[20] − 0x40) × 16 / 256 ; FUN_10015f60
```

### PEG 状态机（FUN_10015b10）
```
stage 计数器 voice[0x5]：1 → CMP elem[27] vs elem[28]（相等 → stage2）
                        2 → CMP elem[28] vs elem[29]
不等 → FUN_10015fe0(voice, elem[23], 0x40) → voice[0x3c]
      FUN_10016080(voice, element, elem[28]) → voice[0x3c]
```

### 曲线 A → DSP（EG 速率键缩）
```
12-bit = clamp(elem[33]→表0x100473D0 + 曲线A(key)×32, 0, 0xFFF)
  → engine[0x1b8] → DSP reg 0x440 → voice 块 +0x0C → 查表系数 voice+0xa8
FUN_10013580 结果（5 层调制）→ engine[0x1dc] → DSP reg 0x5c1 → voice 块 +0x40
  → nibble×0x4000 → +0x100/+0x10c（EG 段 2/3 log 域速率）
```

### 曲线 B → 音量键缩（FUN_100156c0）
```
EBX（力度基值）+ 曲线B(key)×2 → clamp[0,0x80] → voice[0x70]
```

## 实施状态

- [x] Phase 0：本文档 + 注释修正
- [x] Phase 1：错误修正（[14]→LPF 中性、[25]/[26]/[30]→PEG 停用、[54]→移除覆盖、[56]→移除 sustain 近似）
- [x] Phase 1.5：共鸣来源调研 → **S-YXG50 渲染链无共振滤波环节**（8-bit 渲染器纯插值；
  每块链=噪声/步进/LFO/包络/插值），LPF 为 2006LE 模型，共鸣固定中性
- [🔄] Phase 2：
  - [x] 2-1 音高：elem[14] pitch_comp（FUN_10015460 0x40 中心偏移）→ oscillator ratio；
         element_range（FUN_100140f0：pitch_mode 1-4 缩放，表 0x10047750 50/20/10/5%）→ note_in_cent
  - [x] piecewise_curve（FUN_100164f0/FUN_100165b0 4 断点插值）→ 已实现+单测，曲线 A/B 接线待消费者链
  - [x] 2-2 截止路径：**elem[13] 用法确认正确**（rwatch @0x100141a2，FUN_10014190）。
        公式修正：`elem[6] = clamp(engine[8] + elem[13] + elem[6] − 0x80 + engine[0x6394]
        − 0x40 + engine[0xb7], 0, 0x7f)`——elem[6] 为运行时截止累积器（非只读阈值）；
        engine[8]/[0xb7]/[0x6394] = 通道/主截止偏移（madaha 以 Part 08 pp 18 近似）
  - [x] 2-3 键跟随：`key_follow(key, ref, amount) = (key−ref)×(amount−0x40)>>4`（三组同公式，
        FUN_10013e20/FUN_10015770/FUN_10015f60）——助手+单测；接线待 voice 布局
  - [x] 2-4 PEG 速率核心：`peg_rate_word`（FUN_10015fe0：`表0x10048134[clamp(elem[23],0,0x3f)
        + 键跟C + elem[19]缩放 + 0x12]`，哨兵 0x7E/0x7C；表=指数16项+2零+线性）——助手+单测；
        状态机（FUN_10015b10 键位分段选择 voice[0x3a-c]）接线待音频验证
  - [ ] 2-5 曲线 A/B 接线：EG 速率键缩 + 音量键缩——待 voice 布局
- [x] Phase 3：回归 181 全过（176 + 5 新）
