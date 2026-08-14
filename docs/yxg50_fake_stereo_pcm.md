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
