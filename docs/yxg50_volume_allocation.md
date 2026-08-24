# S-YXG50 音量对齐与复音分配（2026-08-13）

## 1. 响度校准（-14 LUFS）

### 目标
不跟 yxg50 对齐——按 TODO 的"响度统一至 -14 LUFS"（EBU R128 节目响度）。

### 测量方法
- **探针**：渲染代表性多音色音频（钢琴/Organ/Marimba/Musicbox/弦乐 × 多音符，15s）
  保存 wav（`dbg_render_lufs_probe`）
- **LUFS 计算**（python）：BS.1770-4 K-weighting（二阶高通 38Hz + 二阶高架 1500Hz/+4dB）
  + 0.4s 块 + 绝对门限（-70 LUFS）+ 相对门限（-10 LUFS）
- **校验**：1kHz 满幅正弦 → -2.79 LUFS（参考 -3.01，误差 0.22dB ✓）

### 结果
- 探针实测（无增益）：**-30.59 LUFS** → 目标 -14 需 +16.59dB（×6.756）
- 峰值 <0.1 × 6.756 ≈ 0.68——GainSink 的 tanh 软削波在 <0.7 基本线性（无压）

### 实施（audio_render.rs master bus）
```rust
const LUFS_GAIN: f32 = 6.756;
let vol = xg_level_gain(sys.master_volume) * gm2_vol * LUFS_GAIN;
```
- 与 config `master_volume`（GainSink 增益 0.05-4.0）叠加——默认总增益 6.756（-14 LUFS 基准）
- GainSink soft_clip（tanh）防强音硬削波

### 验证
- 增益后探针：**-14.00 LUFS**（精确命中）
- 181 测试全过

## 2. 复音分配（冗余池 + 事件级评分释放）

### 设计
- **池大小**：`count = max_polyphony × poly_replicant / 100`（synth.rs）
  - 默认 `max_polyphony = 512`、`poly_replicant = 150%` → 池 768（冗余 256）
- **冗余池作用**：分配缓冲——NoteOn 无条件分配不阻塞；复音限制由事件级释放维护

### 分配逻辑（audio_render.rs note_handler）

```
1. 无条件分配（元素循环）：
   池（count）找空闲 → play(...)
   （仅池满 active ≥ count 兜底：评分最高 kill 腾位）

2. 事件级检查（每个 NoteOn 事件完成后）：
   while active > max_polyphony(512):
       释放评分最高者（max_by_key(scoring)——高分）
       同 note_on_id 组一起 kill（双元素同组释放）
       → 直到 active ≤ 512
```

- **steal 触发 = 超过设定复音数（512）**——不是池满（768）
- **评分语义**：低分 = 保护（新音符×0.1 / sustained×0.1 / drum×0.05）；
  高分 = 优先释放（Releasing×1.5、旧音符累积 time_weight）
- **组释放**：双元素（musicbox/Dream）同 `note_on_id` 一起 kill（避免单边残留）

### 验证（测试）
- `polyphony_redundant_pool_then_event_steal`：count=12/max=8——弹 10 音符 →
  active 2→4→6→8→8（第 9/10 事件后压回 8）；最旧音符（note 60，评分最高）被释放
- `polyphony_steal_releases_dual_element_group`：musicbox 双元素——active ≤ 6；
  同 note 的 voice 活跃数恒为 0 或 2（同组释放 ✓）

## 3. 回归
180 测试全过（177 原 + 2 分配测试 + 1 诊断）
