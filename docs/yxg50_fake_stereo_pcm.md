# S-YXG50 "Fake Stereo" PCM（channel_flag=0x00）合成过程

## 1. 背景与误解修正

`seg16` WaveEntry 的 `[12]` 格式标志（`channel_flag` / `sampleFlags`）：

| 值 | 含义 |
| --- | --- |
| `0x80` | 8-bit 单声道 PCM（字节单位，每位置 1 样本） |
| `0x00` | 16-bit PCM（word 单位，每位置 1 个 16-bit 采样）——旧称"Fake Stereo" |

**历史误解**：`flags=0x00` 曾长期被当作"双声道 L/R 交错 8-bit"，实现为取其中一声道
（`b[(start & 1) ^ 1]`）。实际数据是**相邻字节拼成的 16-bit 采样**（word = `lo | hi<<8`，
小端），取单字节会丢失高字节内容（音色 = 8-bit 量化，听感"EP/电子"）。

**验证**：Marimba（prog 12）采样起点 word 合并后 392Hz 基音 `corr=0.99`（清晰）；
8-bit 单字节流仅 `corr=0.88`。引擎内存中的波形 = 解码后数据再 `^0x5C`（防 dump 层），
渲染器还原后与 madaha 解码数据一致。

## 2. 数据格式（word 单位）

WaveEntry 中与采样数据相关的 24-bit 字段，**单位均为 word（16-bit 采样）**：

```text
negOffset[3-5]   采样负偏移（word）    → 攻击段长度
loopStart[6-8]   循环长度（word）      → 循环段长度
baseAddr[9-11]   采样基地址（字节）    → 指向循环段起点（数据流中的字节偏移）
```

- **采样起点（字节）** = `baseAddr − negOffset×2`
- **采样终点（字节）** = `baseAddr + loopStart×2`
- **总长度** = `(negOffset + loopStart)` word（16-bit 采样数）
- **word 合并**（小端）：`word = data[i] | data[i+1]<<8`，中心化 `word − 0x8000`

实例（Marimba 592）：`neg=4075`、`loop=115`、`base=0x3FF790` →
起点 `0x3FF790 − 8150 = 0x3FD7BA`，共 4190 word（392Hz 基音数据）。

## 3. 引擎渲染链

### 渲染器选择

voice 波形初始化（FUN_1001a6e0 / FUN_100191a0）按 wave 结构标志选择渲染器组：

| wave+0x44 & 0xC0 | 渲染器组 | 语义 |
| --- | --- | --- |
| `0x00` | `0x1001a1d0`（有循环）/ `0x1001a270`（无循环） | 16-bit word 插值 |
| `0x80` | `0x1001abb0` / `0x1001ac90` 组 | 双字节加权混合 |
| 其他 | 跳过（保持默认） | — |

实际渲染器指针写入 `voice+0x88`（有循环 → `+0x94` 候选，无循环 → `+0x98`）。

### 16-bit 渲染器公式（FUN_1001ad60 / 0x1001a1d0）

```test
输出 = word[pos] × (0x7FFF − w) + word[pos+1] × w      （word = (lo | hi<<8) − 0x8000）
w    ：权重相位，每样本 += (step 低 16 位)，模 0x8000
frac ：每样本 += step；pos += frac>>15；frac &= 0x7FFF   （位置推进，word 单位）
循环 ：pos ≥ 循环长度 → 回绕（+0x74 步进调整）
step ：FUN_1001a790 计算：(音高×系数 >> 移位)×基础步进>>11 + 基础步进（音高相关）
```

- **word 读取**：`*(ushort*)(data + pos*2)`（小端，2 字节/位置）
- **`^0x5C5C`**：防 dump 层——原版内存数据 = 解码后 ^0x5C，渲染器还原。madaha
  使用已解码数据，**不需要**该层
- **位置初始**：`pos = −攻击长度`（data 指向循环起点，负偏移到攻击段）

### 权重相位（w）的实测

Marimba（note 60）校准：`step=0x5740`、`frac` 循环（0→0x2000→…→0）、
`pos` 从 −4075 递增、`loop=115`、`+0x6c=0xFEB`（攻击长度）。

w 的大步进跳变（阶梯化）对最终谐波结构影响很小（模拟 + 实测一致）——
16-bit 音色主要来自**完整 word 内容**（高字节贡献的泛音/细节），
单字节取流（8-bit 量化）才是音色失真的根源。

## 4. madaha 实现

### set_wave（libmadaha/src/yxg50/sample_meta.rs、drum_setup.rs）

```rust
// channel_flag=0x00: 16-bit PCM（word 单位）。neg/loop 单位 word → 字节 ×2
let scale = if self.channel_flag & 0x80 == 0 { 2 } else { 1 };
let start = self.loop_start - self.start_point_offset * scale;
let end = self.loop_start + self.loop_length * scale;
// pcm = word 合并（小端 lo | hi<<8）中心化
wp.chunks_exact(2)
    .map(|b| ((b[0] as i32 | (b[1] as i32) << 8) - 0x8000) as f32 / 0x8000 as f32)
    .collect()
```

- **不除 2**：`loop_point = negOffset`（word 数）保持原值（旧"双声道"实现 `/2` 错误）
- **8-bit（flags=0x80）**：`scale=1`、逐字节 `u8_to_f32`（原路径不变）

### oscillator（src/audio/tone_generator/oscillator/）

- `interpolate_xg`（interpolating.rs）：引擎公式结构（word 相邻加权 + w 相位累加器）
- 16-bit（`channel_flag & 0x80 == 0`）走 `interpolate_xg`；8-bit 走标准插值
- w 相位每音符重置（`xg_w_phase`）

## 5. 8-bit（flags=0x80）对照

- 字节单位（每位置 1 样本）、标准线性插值（madaha 现有路径）
- musicbox、钢琴、Dream 等 flags=0x80 音色不受本次修复影响（回归 177 全过）

## 6. 鼓（DrumSetupEntry）

`drum_setup.rs` 的 `channel_flag=0x00` 分支同样为 16-bit（word 单位），
修复逻辑与 sample_meta 一致（165 个 channel_flag=0 的鼓采样）。

## 7. 验证记录

- **数据**：Marimba 592 word 合并 392Hz `corr=0.99`；引擎内存 = 解码后 ^0x5C（还原一致）
- **听感**：Marimba（prog 12）音色正确（用户确认）；Tubular Bell 等 16-bit 音色正确
- **回归**：177 测试全过（musicbox/8-bit 零影响）

## 8. 持续/打击语义（短采样分支废弃）

**问题**：曾用"采样总长 < 200ms → sustain 0"判定打击乐（为 Marimba 的衰减引入）。
但这**误伤持续音色**——Organ（11.7ms）、Harmonica（102ms）、Accordion（56ms）
都是短采样但**持续**（key 按住一直发音，靠循环 + EG sustain 保持）。

**根因**：采样长度不能区分"打击 vs 持续"——Marimba（短采样但衰减）与
Organ（短采样但持续）长度相近，行为不同。

**修复**（2026-08-13）：移除短采样分支——sustain 恢复 0.7：
- Organ/Harmonica/Accordion 延音恢复（RMS 恒定：Organ 5.0%、Harmonica 5.2%、Accordion 10.2%）
- Marimba 持续（sustain 0.7）——**音色无影响（用户确认）**

**遗留**：元素驱动的持续/打击区分（如 `[31] dsp_base/[33] tbl_index` 的 DSP 配置
语义）尚未实现——当前 sustain 0.7 硬编码对现有音色可接受。

## 9. element 字段语义补充（2026-08-14 动态验证）

### [70] wave_pitch —— 持续/打击控制字段（已实现）

引擎机制（动态验证 Marimba vs Organ）：
- **元素 [70] × 2 = EG 段目标表索引**（wave 结构 +0x24）
- 段目标表（128 项 log 域 uint32，动态 dump 自引擎 0x01dc2f48）：
  - Marimba [70]=0x1a → 表[52]=0x600 → sustain ≈ 0.014（**衰减到近 0**）
  - Organ [70]=0x3f → 表[126]=0xf83e0 → sustain 0.4687（**保持**）
- 幅度 EG（voice+0xe0/+0xe4）实测：Marimba 1.0→0.25→0.06→0.014（衰减）、
  Organ 1.0→0.4687 恒定（保持）
- madaha 实现：`sustain_level = f(EG_TARGET_TABLE[[70]×2])`（exp 映射 0→0.014、15→0.4687）
  ——替换硬编码 0.7（tone_generator.rs 内建表）

### [33] tbl_index —— DSP 调制参数表索引（完整链）

FUN_10015880（vtable+0x590 转换器）→ DSP 上传：
```
元素[33] → SHORT_ARRAY_100473d0[索引]（12-bit 递增参数表，0x600 起）
         → + 键位曲线（FUN_100164c0）修正 → clamp 0..0xfff → voice+0x18（ushort）
voice+0x1a（另一参数）+ voice+0x18 → eax
voice+0x4d = 0xf0（DSP 命令）→ vtable+0x4d0（运行时 0x100131e0）：
  参数变换（+0x70 DSP 映射表、+0x63 字段、+0x4d 命令偏移）
  → clamp → vtable DSP 写函数（寄存器由 0xf0 命令决定）
```
表 0x100473d0：0x600, 0x61a, 0x635, 0x64f...（递增，12-bit 域）。
Organ 实测：+0x18 = 0x0e58（3664）、+0x1a = 0。

**最终语义（DSP 寄存器已确认）**：
- voice+0x1a（主音高高 9 位，FUN_10015460 主公式）+ voice+0x18（[33] 参数 12-bit）
  → 组合 → voice+0x1b6
- FUN_10005140（DSP 上传器）：voice+0x1b4/+0x1b6 → **DSP 0x400（21-bit 音高字）**
- 即 **[33] = 音高（21-bit）的低 12 位微调分量**（表 + 键位曲线驱动）——
  cent 级微调，与主音高组合成完整音高字
- 0xf0 命令（FUN_10015880 的第二个操作）→ voice+0x1b8 = 0x7f（DSP 0x440 曲线 A
  寄存器重置）——与 [33] 无关

madaha 对应：音高公式（cents→ratio）可加 [33] 分量（表 0x473d0 值 → 微调 cents）——
幅度极小（21-bit 低 12 位），暂不实现（记录备查）。

### [31] dsp_base —— DSP 0x400 音高字高 5 位（粗调）【2026-08-14 修正】

**勘误**：上文「[31] 未消费」结论有误——之前的调查只覆盖了 FUN_10015a20 的 9 个转换器，
漏掉了 `FUN_10013440`（读 [8]/[9]/[10] 的 FUN_100130a0 同一片区域）。`dev_docs/note_opencode.md`
第 601/639 行早已记录 `[31]` 读于 0x10013456、`[77]` 读于 0x10013530。

**实际消费链**（FUN_10013440，读 elem[31]/[77]）：
```
FUN_10013530(engine, note, element):            ; [77] sensitivity → 力度调制量
  raw = elem[77]（signed）- 0x40
  raw == 0 → 0
  raw > 0 → ((0x80 - vel) * raw * 18) >> 8     ; vel = note[0x73]（力度）
  raw < 0 → (vel * raw * -18) >> 8
result = clamp(elem[31] - sens, 0, 0xffff)
if (note.mode == 2)  result = FUN_100133f0(...)   ; 鼓模式加 part 偏移
FUN_100134e0(engine, note, result):
  part_off = part[0x19] - 0x40                   ; part 字段 +0x19（居中）
  combined = (part_off < 0) ? clamp(part_off + result, 0, ..) : max(part_off, result)
  engine[0x1b4] = (combined >> 1) & 0x1f         ; 5-bit，同时写 note[9]

FUN_10005140（DSP 上传器）:
  DSP 0x400 = FUN_100050d0(field_0x1b6, field_0x1b4)
            = field_0x1b4 * 0x800 + field_0x1b6   ; 16-bit
```

**语义**：`[31] dsp_base` = DSP 0x400 音高字的**高 5 位（粗调/整数部分）**，被
`[77] sensitivity`（力度敏感度，0x40 居中）调制后写入 `engine[0x1b4]`；主音高 + `[33]`
构成低 11 位（微调）。即 **[31] 粗调 + [33] 微调 共同构成 DSP 0x400 音高字**。

**动态实测**（KeyOn，vel=100）：
- Marimba：elem[31]=0x10 → engine[0x1b4]=0x08；elem[77]=0x40（中性）
- Organ：elem[31]=0x08 → engine[0x1b4]=0x04；elem[77]=0x40（中性）

**128 音色分布**：[31] 0~54（最常见 0×35 个、8×36 个）；[77] 大多 0x40（118 个），
仅 10 个音色非中性（66/67/69/71，如钢琴 0-4）——力度敏感度仅少数音色启用。

madaha 对应：字段已解析（`dsp_base`/`sensitivity` + `sensitivity_signed()`）但**未接入
音高公式**。与 [33] 同类（音高字分量），幅度小、需完整 DSP F-num 公式才可精确映射，
暂不实现（记录备查）。
