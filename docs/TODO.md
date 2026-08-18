# Madaha TODO / 暂不实现清单

决策记录（2026-08-05）：Madaha 是 S-YXG2006LE 的超集实现。
以下项目经评审后决定"暂不实现"或"待办"，按优先级排序。

---

## 一、当前待办（按优先级）

### 1. Element 消费对齐（Phase 2 剩余）

#### PEG 状态机接线（✅ 2026-08-14 已完成）

peg.rs 已重写为引擎 4 电平包络模型（elem[26]→[27]→[28]→[29]→sustain），181 测试全过：
- 速率字表接入（`peg_rate_word` 补 key_in 项 + 完整 88 项 dump，哨兵 0x8000/0x1830）
- 3 段速率比较跳过（elem[27]==[28] 跳 attack、elem[28]==[29] 跳 decay）
- 电平计算（`peg_level`：elem[17] voice_type 倍率 ÷4/÷2/×1/×2 + vel_sense）
- 键跟 C（elem[20]/[21]）+ elem[19] 力度缩放汇入速率索引
- 修正 elem[20] 用途（键跟C depth，非 PEG depth）
- Part 音高 EG（apply_xg_eg）保留原有「init→0」覆盖语义（默认速率 OLD 近似）

**遗留**：Part 音高 EG 的精确速率/电平映射（引擎 part[0x62]/[0x63] = FUN_10015f10 ×37.5）
与 PEG 速率字的块大小（假定 128 样本）待音频 A/B 校准。

#### 键跟随接线（逆向定案 2026-08-14，键跟 B/C 已接线）

公式已实现 + 单测（`key_follow(key, ref, amount) = (key−ref)×(amount−0x40)>>4`，SAR floor）：
- 组 A：ref=elem[45]、amount=elem[44]（→ voice[0x57]，FUN_10013e20）——**⚠ 暂缓**
- 组 B：ref=elem[67]、amount=elem[66]（→ voice[0x65]，FUN_10015770）——**✅ 已接线**（key_on_delay）
- 组 C：ref=elem[21]、amount=elem[20]（→ voice[0x4f]，FUN_10015f60）——**✅ 已接线**（peg.rs 速率）

**消费者定案**（详见 element_alignment.md「键跟随」节）：
- A → FEG（= CS/LS 包络）速率（表 0x10047550[elem[46/47/48/49] + 键跟A + 力度缩放(elem[43])]）
- B → eg_enable[71] + key_on_delay[72]（FUN_10012600/12670：`clamp(...)+键跟B → ×2`）
- C → PEG 速率（已接线）

**键跟 B 精确实现**（2026-08-15）：`key_on_delay_index()`（pre_voice.rs，表 0x10046cd8 +
`clamp(...)+kf_B → ×2`）+ KEY_ON_DELAY_TABLE 按 ×2 重索引（修正原 raw elem[72] 索引）。

**键跟 A 暂缓（关键发现）**：FEG 输出（`voice[0x1a]` = 电平>>2）经 FUN_100146d0 @0x10014778
加到 `voice[0x18]`（[33] 参数）→ vtable[0x4d0] → **DSP 0x400 音高字**——即 S-YXG50「滤波」=
**采样率截止**（经音高字），**非** madaha 的 2006LE LPF。FEG 重写尝试已回退（输出映射不匹配）。
键跟 A 待「采样率截止 / 2006LE 数据文件」阶段统一对齐（FUN_10014200 二维表 0x10047F50 链）。

#### 曲线 A/B 完整接线

- 曲线 A（elem[34..41]）= EG 段 2/3 log 域速率键位缩放（DSP 0x440/0x5c1 链）——H2 已用于 AEG decay/release
- 曲线 B（elem[56..63]）= 音量键缩（FUN_100156c0 → voice[0x70]）——P1 已用于每元素音量平衡
- 完整 voice 布局重建后接线（piecewise_curve 已实现+单测）

#### [31] dsp_base / [33] tbl_index / [77] sensitivity（语义已定案，暂不实现）

- `[31]` = DSP 0x400 音高字**高 5 位（粗调）**；`[33]` = 低 11 位（微调）；二者共同构成 16-bit 音高字
- `[77]` = 力度敏感度（0x40 居中）：`(0x80−vel)×([77]−0x40)×18>>8`（正）/ 负分支
- 链：`elem[31]−sens → clamp → engine[0x1b4]（5-bit）→ DSP 0x400 = 0x1b4×0x800 + 0x1b6`
- 动态实测：Marimba [31]=0x10→0x1b4=0x08、Organ [31]=0x08→0x1b4=0x04（[77] 均 0x40 中性）
- 分布：[31] 0~54（最常见 0×35/8×36）；[77] 118 个 0x40，仅 10 个非中性（钢琴 0-4 等）
- madaha 状态：`dsp_base`/`sensitivity` 已解析（pre_voice.rs，含 `sensitivity_signed()`）但未接入音高公式
- 决策：暂不实现（音高字分量，幅度小、需完整 DSP F-num 公式）。详见 yxg50_fake_stereo_pcm.md §9 与 element_alignment.md

#### 音高公式补全（FUN_10015460，✅ 2026-08-17 大部分完成）

- [x] **scale_tuning[key%12]**（engine[0x3a]，键表）——`get_delta_pitch` 已接（`scale_tuning[note%12]−64`）
- [x] **master_tune**（engine[0x63a6]）——`(System.get_master_tune()−0x0400)/10.0` 分，经 `AudioRender` → `ToneGenerator::play()` 注入。测试 `master_tune_shifts_pitch`
- [x] **elem[7] pitch_offset**（`+ elem[7] − 0x40`，signed）——oscillator ratio 接入
- [ ] **elem[9]/[10] pitch_fine**（12-bit，FUN_10013720 键位缩放）——暂缓：elem[9]/[10] 实测多为 0，
      按公式代入得 −2048（与「音准已验证不应用」矛盾），疑 engine[9]/[10] 非 elem[9]/[10] 直拷，
      需找写入点后再接（见 note_opencode.md §10.4）
- 完整公式：`pitch = (range−baseKey)×100 + tone + scale_tuning[key%12] + master_tune − 0x40 + elem[14] + elem[7] − 0x40 + pitch_fine`

### 2. AEG / sustain 对齐

- [ ] **5 段 AEG 重构**：KeyOnDelay → Attack → Decay1 → Decay2 → Decay3（现仅 Attack/Decay/Sustain/Release）
- [ ] **驱动模型**：rate-based（每周期增量，rate 域 0-0x7f）；`_gfAEGAttackCycle` 表已定位（0x91360，
      rate 0→699050 周期、127→129 周期），Decay/Release 表待提取
- [ ] **D2L（sustain level）真实来源**：2006LE `_utgd_GetAegLvlDrctnToD2l` = decay1/2/3 电平**单调递减**
      检查（已逆向）；madaha 目前用 `wave_pitch[70]`→EG_TARGET_TABLE（0.7/0.014/0.4687）作 sustain，
      aeg_d2 不再用于 sustain（旧 `1−aeg_d2/127` 已移除）
- [ ] **制音器模式（PianoDamperMode）**：2006LE sustain_mode=2 走 ShiftAEGSegmentPFDamper；
      S-YXG50 为 XG 全局 hold（无 PFDamper 段）——Element 已预留 `sustain_mode` 字段
      （0=无/1=恒保持/2=制音器），待 2006LE 数据文件读取时填充实现
- [ ] **force damp 对齐**：SetupForceDampAEG 强制制音时 release 目标 ≥ 0x60（voice+0x190 下限）——
      madaha 的 CC123 直接 release/kill，无电平下限调整
- [ ] **aeg_d1/d2/aeg_rel 语义（✅ 已确认，不接）**：字节级搜索确认 voice[0x60]/[0x61]/[0xc0]/[0xc1]/[0x66]
      全部**只写不读**（S-YXG50 引擎忽略）；仅 voice[0x68]（aeg_rel[57]）被读——作**特殊值条件标记**
      （120/126/127 = 无释音等），非速率/电平。AEG 速率来自 EG 速率表 + Part EG/曲线 A。
      2026-08-17 曾试把 aeg_rel→release、aeg_d1_val→Decay1 速率——音色退化（钢琴像电钢），已回退。
      madaha 现有 curve_a + Part EG 方向正确，不再按速率/电平接这些字段。

### 3. musicbox 音色残余差异

P1/P3 波形对齐后残余（onset 归一化 10ms=88/25ms=75/1s=25/1.5s=0 vs yxg50 68/96/3/1——单峰+方向一致）：
- [ ] 凹陷深度（madaha 100ms 28% vs yxg50 63%）与拍频细节——可能是 elem1 的 decay/sustain
      与引擎 3 段 EG 的段语义差异
- [ ] D2L 真实来源（AEG 初始化 FUN_100103f0 表驱动未暴露）+ 击锤层识别启发式
- 注记：早期"击锤在 elem1"结论已被纠正为"击锤在 elem0"（L234-236 核实）；elem0 持续性
      实验（sustain 0.811 致泛音过强）已确认方向

### 4. 计划（2026-08-10）四阶段未完成项

- [ ] **前置测试框架**：e2e 公共模块（render_sequence/write_wav/断言助手）、鲁棒性
      （128 音色遍历、随机模糊、压力）、录音对比工作流脚本化
- [ ] **阶段 1（非 SysEx 校准）**：CC67 soft pedal 接线核对、CC71-75（谐波/起音/释音/亮度
      → 08 pp 18-1C RAM）断言、每指令端到端断言 + 录音 A/B
- [ ] **阶段 1 HPF 对齐**：HPF 截止映射、与 LPF 的共享 CLFO 调制、多模输出混合
      （2006LE field_0x48-54）；录音 A/B 验证低频保留度
- [ ] **阶段 2（SysEx）**：XG 参数 SysEx 写入→声音变化端到端测试（volume 已测）、
      NRPN 全参数映射核对（nrpn_to_addr 覆盖度）、UniversalRealtime 细节
- [ ] **阶段 2 PLUGIN 区**：`SetPluginForPart` 副作用链未接（part.engine 赋值 → TG 发声按
      engine 路由）；写入检查 + 警告（RcvNote 未关闭/板未分配，只警告一次）；测试
- [x] **阶段 3（鼓）**：普通鼓 PCM 加载（按 start_point_offset/loop_length 从 sxgwave4 取，
      参考 melodic set_wave 路径）；鼓音色端到端（发声、drum_params 生效、alter group 截断）
      —— 2026-08-16：`From<&YXG50DrumSetupEntry>` 的 `pcm: None` 已修（clone）、鼓/SFX
      SampleMeta 按 drum entry 索引用 Vec 共享（避免 OOM）、SFX 路径修复（seg06 字节偏移 ÷2 +
      drum_setup 接线）。测试：voice_manager_get_drum_sample / voice_manager_get_sfx_sample /
      drum_note_produces_sound / drum_alternate_group_cuts_off
- [ ] **阶段 3b（鼓压缩 PCM，暂缓）**：18 个鼓条目 `sampleNegOffset[19] < 4` 使用 DSP 压缩
      格式（`[19]=0` 表 0x100293F0、`[19]=1` ADPCM 0x1002E420/0x1002E8F0、`[19]=2` 循环压缩
      跳表、`[19]=3` 扩展格式 0x10029434）——当前 `set_wave` 按未压缩 8-bit 处理会输出噪声。
      Standard Kit 中受影响键：62（闷康加）、75/76/77（木鱼）等 ~8 键。需逆向 S-YXG50 DSP
      解压器后实现。
- [ ] **阶段 4（效果器）**：效果器音质端到端（wet/dry、参数变化、send 电平）、
      variation/insertion 参数语义与 2006LE 对齐、multi_part_ext 全参数消费核对、输出断言 + 录音 A/B

---

## 二、暂不实现（用户决策）

### GS 的 2006LE 数据文件

- S-YXG50 数据文件（sxgbin41.tbl / sxgwave4.tbl）已完整支持 GS 音色/鼓组
- S-YXG2006LE 的数据文件（sxgbnw6l.tbl / sxgbnm6l.tbl / sxgdat6l.tbl）暂不读取
- 状态：`voice_manager` 只加载 S-YXG50 系 TBL；2006LE TBL 解析待实现

### XG 插件区（SysEx 0x70/0x71）

- `src/midi/ram/xg/ram.rs:39` TODO——PLG 插件效果区无 RAM 字段
- 2006LE 也不支持；XG Spec 定义存在，暂不实现

### LCD 显示

- RAM 字段（display_letter / display_bitmap）完整，但不接入任何输出
- 2006LE 有 VST GUI 面板；Madaha headless 无 UI，LCD 不实现

### System Common 消息

- MTC QuarterFrame / SongPosition / SongSelect / TimingClock / Start / Continue
  / Stop / SystemReset / TuneRequest 均忽略（engine.rs `_ =>` 分支）
- 决策：Madaha 只接收 MIDI → 出声，不返回/处理任何 MIDI 信号

### VST 插件接口

- 2006LE 是 VST 插件（processReplacing/VST 事件接口）
- Madaha 是 headless 控制台程序（CLI + 后端），不做 VST 宿主接口

### VA 算法

- 已经在它处完成逆向，必要时可以用浮点数实现
- 单独拉出来做 LV2 也许可行，稍微扩展一下木吉他也能搞，填补 Linux 环境没有原生物理建模音源的空白

### DX 模块（PLG-DX，FS1R 作参考）

- 根据 Dexed/FS1R 实现 DX 模块模拟

### OPL3 模块

### WinGroove 模块

- 主引擎不再加载 WinGroove TPD 文件
- 已知：WinGroove 采样格式为有符号 8 位（-128 至 +127），22050Hz
- TPD 文件字段需要进一步分析

---

## 三、已完成历史

### 效果器 / XG2.0 超集（2026-08-05 系列）

- [x] XG2.0 串联组合（阶段 3a）：Distortion/Overdrive/V-Distortion/Wah/Compressor →
      Delay/Rotary 串联（SerialChain）；Tempo 变体用普通延迟（无 tempo 源）
- [x] Dyna 家族（阶段 3b）：DynaFilter（LFO 扫频 LPF）/ DynaFlanger / DynaPhaser / DynaRingModulator
- [x] 调制/空间类（阶段 3c，xg20_effects.rs 近似）：RingModulator、EnsembleDetune、Ambience、
      WideStereo、ThreeDManual/Auto、VibeVibrate、LoFi、Slice、Isolator、LowResolution、
      DigitalTurntable/Scratch、MultiBandComp、TempoFlanger/Phaser、VFlanger、DualRotorSpeaker1/2
- [x] GS 兼容：`gs_xg_addr_remap` 扩展（0x41 鼓 7 参数 + 0x42 Scale Tuning + 0x43 Part Tuning）+ GS reset 分支
- [x] Reverb/Chorus 扩展类型（0x0D-0x12 / 0x10-0x14，2006LE 权威值）
- [x] xg_20_data / xg_distortion_data 恢复（B5b merge 回归）
- [x] DC 偏移修正（DcBlocker 主总线，XG Spec 依据，固定启用）
- [x] 测试盲区补测（config/encode/LFO/voice_manager/XG RAM +16 测试；修复 MidiConfig::check_scoring 无限递归 bug）
- [x] XG2.0 参数布局精化（49 处默认值按 XG Spec 修正；串联类型布局 Delay/Distortion 参数分配修正）
- [x] OSC 绑定（bind_part + is_drum 读 part_mode）
- [x] LFO Key-sync 接入（note-on 相位复位）
- [x] scoring AEG 阶段保护（Attack 期权重加倍）
- [x] Variation 混响类类型切换默认加载（Chorus 布局）
- [x] Dyna 家族包络驱动：DynaEnv（Attack/Release 平滑 + Threshold + Sensitivity）、
      DynaFilter 包络扫频、DynaFlanger/Phaser 包络调制、DynaRingMod 包络
- [x] Wah/Compressor 前级串联：SerialChain 3 级（Wah→Dist→Delay / Comp→Dist→Delay），
      P11-14 附加参数接入
- [x] TWO WAY ROTARY 布局核对：P1 转速、P14-16 失真前级（Drive/LPF/Output）
- [x] xg20_effects.rs 索引核对：Ensemble/LoFi/RingMod/LowRes/Ambience/Isolator
- [x] 多段压缩（MultiBandCompEffect：3 段分频 + 软拐点压缩 + 段增益）
- [x] WSOLA 移调器（core/wsola.rs，共用组件——PitchChange + Harmony 受益）：
      读指针速率扫描 + 追尾对齐跳回（SAD 相似段搜索 + Hann cross-fade）
- [x] 和声类效果（harmony_effect.rs）：ChordDetector + HarmonyEffect（WSOLA×3 声部 + Vibrato +
      Lead/Harmony 平衡）+ Vocoder 简化 + TalkingModulator（formant 滤波）；
      VOCODER_HARMONY/CHORDAL_HARMONY/DETUNE_HARMONY/CHROMATIC_HARMONY/TALKING_MODULATOR
- [x] 活跃音符采集（audio_render 每块收集 → EffectProcessor::set_active_notes）
- [x] PitchChangeEffect 迁移到 WSOLA
- [x] config 音频参数扩展：master_volume（0.05-4.0 校验）、soft_clip、dc_blocker（可关）、
      alsa_buffer_frames（2 的幂校验）；GainSink（输出增益+软限幅，所有后端统一）、dc_enabled 接线

### 里程碑（2026-08-10）：简单 MIDI 文件可播放

#### 双元素音色检查（2026-08-10，Dream LSB=41 prog=0 验证通过）

修复的 4 个深层 bug：
1. **`pre_voice.rs` 双元素判定**：`& 0x3 == 3` → `& 0x2 != 0`（Ghidra FUN_10016fa0：header[1] bit1）
2. **`get_program_index` LSB `+1` 偏移**：Dream（lsb=41）误查 lsb_table[42] → 直接用 lsb（sxgparser 一致）
3. **`controller.rs` rcv_bank_select 的 `part_mode != 0`**：旋律 part（mode=0）的 CC32 被拒 → 移除
4. **`MultiPart hook_check` 0x03 program change**：相同 program（0==0）不触发 ChangeProgram →
   无条件应用（重选音色有效）；顺带 0x01/0x02 去掉 `part_mode == 0`

回归：dual_element_dream_allocates_two_voices / dual_element_dream_loads_two_tone_generators

#### 双元素 NoteOff 组释放（2026-08-10 已修）

- **bug**：release_handler 只释放最早一个 Running TG——双元素时 NoteOff 只释放元素之一
- **修复**：TG 加 `note_on_id`（NoteOn 组标识）；`release_note` 释放最早 note_on_id 的**整组** voice
- 语义保留：stacking（连按同音）每个 NoteOff 释放一组；sustain 挂起/松开、CC123/120 同组语义
- 回归：dual_element_noteoff_releases_both_voices / dual_element_noteon_zero_velocity_releases_both

#### 播放/后端/音高链

- [x] **简单 MIDI 文件已可播放**：仅含 NoteOn/NoteOff 的单 channel 文件（441 测试全绿）
- [x] 后端重构：音频 cpal（替换 ALSA/PipeWire）、MIDI ALSA（删 PipeWire）；后端/trait 简化
- [x] 音高链修复（最终版）：`get_sample_rate → 44100`（PCM 22050 录、44100 播 = ×2 trick）、
      `base_note = baseKey`（采样准）、公式 `note - base + tone`（seg16 data[2] = 录制微调音分）、
      移除 wave_pitch/pitch_offset/fine 音分项、detune 默认 0x80 表索引 bug（-50 音分）
- [x] 音准客观验证（numpy）：48/60/69/81 → 130.75/261.66/440.04/879.85Hz（±0.5 音分内）
- [x] LPF/HPF 按 2006LE 重写：Chamberlin 二阶 SVF、`K=clamp(3−2f,·,2.0)`、
      `ExchangeResonanceToLinear` 忠实移植；消除 Simper SVF 高 Q 削波（原 64 中心 → Q=5.29）
- [x] PEG 修复（音头变调）：XG 语义 = 初始电平滑向 0（原实现 0→+100 反了）、
      level 参数化（elem[20] depth，64 中性 → 无滑音）、rate 指数近似暂留
- [x] 复音释放修复：`release_handler` 过滤 Running（快速同音连按 attack_time 相同导致音符永不释放）
- [x] 延迟修复：cpal ring 65536→256 帧（渲染 backpressure 钳制到实时，按下即响）
- [x] tone_generator 链式重构：`self.osc().lpf().hpf().amp().eq().pan()`（逐级排查）
- [x] Lanczos 内核 LUT 化（4096 表 + 线性插值，音色不变）

#### CC / sustain / damper / 触发

- [x] **CC64 sustain 挂起释放（2026-08-10）**：NoteOff 踏板踩下挂起（EG 保持）、松开批量 Release；
      **CC#123 释放挂起音符 + 清空队列；CC#120 kill 挂起 + 清空**
- [x] **校准（2026-08-10）**：bend 默认 ±2 半音 ✓、**RPN#0 改 bend 范围**、**CC#7 volume 实时性**、
      **CC#10 pan 实时性**、CC#11 expression ✓、CC#10 极左/极右断言、CAT/PAT 存储断言
- [x] **CC66 sostenuto（2026-08-10）**：只保持踏板踩下时已响的音符（快照机制），
      新音符正常释放；松开批量 Release；CC123/120 同样清空
- [x] **PianoDamperMode 评估（2026-08-10 静态深挖定案，暂不实现）**：S-YXG50 为 XG 全局 hold
      （CC64→engine 状态，所有音色统一），madaha 现有全局挂起已匹配，无 PFDamper；
      `Element` 已预留 `sustain_mode` 字段（解析=0）
- [x] **PortUnsubscribed 核实（2026-08-11）**：已是 release 语义（非 kill）——engine.rs:177 发
      ReleaseAll（CC#123 语义），release_all_handler 对全部 Running TG 调 t.release()
- [x] **damper 策略实现（2026-08-11）**：sustain_mode 字段优先（1=恒保持/2=damper），
      S-YXG50 无字段（0）→ program 0-7（XG 钢琴组）fallback 到 damper；AEG 加 Damp 段（damp_time=3s 近似）
- [x] **program number 决定 sustain 策略评估（2026-08-10，已作为 fallback 实现）**：
      钢琴"明显衰减"来自元素 AEG 参数（sustain level 低），非 damper 段；根因候选是 D2L 映射疑误
- [x] **电钢感根因 = HPF（2026-08-18，✅ 汇编确认 + 修复）**：madaha 总是 `lpf().hpf()` 串联，
      而 2006LE `CSTGChannel::FilterSingle` 只用**一个 DCF**（双滤波仅 Double 路由音色用）、
      S-YXG50 链无 HPF。默认 HPF 0x40→~458Hz 切掉钢琴基频（DFT：261Hz 3.42→19.25 恢复）。
      **修复**：`MultiPartExt::new()` `hpf_cutoff_freq` 默认 0x40→0（≈20Hz 中性）。详见 note_opencode §10.10。
      回归：pitchbend 阈值 40%→90%（Damp 态自然衰减 76%）；串行基线 189 过。
      仍待：用户听音确认。
- [x] **顶八度发软/没音头 = 旋律 key_on_delay 误应用（2026-08-18，✅ 修复）**：elem[72]=31 经
      (31+kf)×2→62→延迟表 81-139ms，AEG 停在 Delay(level=0) 把音头（锤击瞬态）静音 →
      顶八度（kd 随音高增大，139ms）没音头+发软。母机 voice[0x66] 只写不读 → 旋律不应用
      key_on_delay（鼓保留）。实测 C7 攻击峰值 0.188→0.850、全音区攻击一致；串行 191 过。
      详见 note_opencode §10.11。
- [x] **拖动滑音音头小 = 限幅器过猛 + 增益过热（2026-08-18，✅）**：密集滑音总线 2-3.4× 阈值，
      原限幅器(0.85/50ms)增益压到 0.25-0.40 → 新音头 -8~-12dB。修改：限幅器阈值 0.95、
      释放 20ms；LUFS_GAIN 6.756→4.2（-4.1dB，单音攻击 ~0.9 不削，密集滑音增益恢复
      0.28→0.45-0.70，音头保留）。代价 loudness -14→≈-18 LUFS（动态优先）。串行 192 过。
- [x] **输出侧动态响度 -14 LUFS（2026-08-18，✅）**：BS.1770 K-weighting + 慢速 AGC
      （`dsp/core/loudness.rs`，0.4s 块短时 3s 门控，±0.5dB/块、范围 [0.15,8.0]、静音冻结）
      挂在 master（计量 pre-gain、限幅器前施加）。实时路径 `[audio] loudness_norm=true`。
      验证：5 音色探针收敛 projected **-14.28 LUFS**（基准 -3.22 + 切 -11.06dB）；
      限幅器阈值按用户选择回 **0.85**。串行 195 过 + libmadaha 14 过。详见 note_opencode §10.12。
- [x] **音乐盒（prog 10）延音太短（2026-08-18，✅）**：wave_pitch→EG_TARGET_TABLE 分类让
      musicbox 落 0.014 类 → ~30ms 塌成静音。特判 prog 10 sustain=0.55（长鸣，release 收尾），
      实测按住包络 0.30-0.55 持续 3s。串行 196 过 + libmadaha 14 过。详见 note_opencode §10.13。
      待定：mid 桶共 ~50 program（含钢琴 1/3/4 等）同样偏短——需逐个试听决定是否整体细化。
- [x] **overdrive 修复 = master 峰值限幅器（2026-08-18，✅）**：低频恢复后单音攻击瞬态峰值 1.481
      超 ±1.0，生产链 soft_clip=false + 无 tanh → DAC 硬削波 = 过载。加 `dsp/core/limiter.rs`
      （快攻击/50ms 释放/阈值 0.85，阈值下透明不改响度），攻击峰值 1.481→0.850。详见 note_opencode §10.10。
      回归：串行 189 过 + libmadaha 14 过。

#### 采样/音质修复（2026-08-07 ~ 08-11）

- [x] **采样加载（08-07）**：`0x80` 位=1 连续逐字节（22050Hz）；`0x80` 位=0 每 2 字节取第 1 字节
      拼 8-bit 波形（第 2 字节 SMID 索引），start_point_offset/loop_length 字节→样本（÷2）
- [x] **调制深度公式错误（08-07）**：filter/amp 控制改 `(v-64)/64`（中心 0）；MW/CAT/PAT pitch 改
      `(v-64)` 半音；`mod_bend_pitch` 固定 1.0（避免双重应用）。回归：pitchbend_changes_pitch_not_volume
- [x] **音高偏移（08-07）**：MIDI 输入 note 映射 -12（源 +12 修复）；base_note 改回 seg16 baseKey；
      coarse 符号移回 note 侧；baseKey→频率 +440 cents。141 测试通过
- [x] **ALSA MIDI pitchbend 中心值（08-07）**：`(pitch.value + 0x2000) as u16`
- [x] **元素级 AEG 时长映射系统性修复（08-11）**：`eg_time_ms` 改 S-YXG2006LE `_gfAEGAttackCycle`
      指数表语义（rate 0→15.85s、127→2.92ms）；key_assign=0 单音替换 `release()`→`kill()`。
      回归：musicbox_element_release_uses_xg_rate_table（173 全过）
- [x] **cents_to_ratio 表扩展（08-11）**：±4096 → ±11520 音分（±8 八度，23041 项）——note 62 及以下
      恢复精确音高。测试：cents_to_ratio_bounds 扩展（171 全过）
- [x] **切换音色 + 滑音音色混乱修复（08-11）**：修复 A（脏 voice 防御，play 绑定链加 else kill）、
      修复 B（xorshift 分配 + idle_buffer 缓冲轮）。171 全过
- [x] **震音卡顿修复（08-11）**：`fast_sine.rs` 的 `pub const SINE_TABLE: LazyLock` 每次 deref 重新
      初始化（~65us/次）→ `const`→`static`（一次初始化）。实测 x6.01 → x0.21（28 倍余量）
- [x] **渲染线程 watchdog 休眠（08-11）**：所有 TG Idle 且静音窗口过后 sleep_idle 阻塞等待事件；
      事件到达立即唤醒。测试：watchdog_sleeps_after_silence_window / sleep_idle_wakes_on_event（169 全过）
- [x] **TG 分配：去 xorshift 随机 → 关联元素连续分配（08-11）**：find_idle_voice 顺序扫描 +
      idle_buffer 缓冲轮；find_adjacent_idle 双元素相邻放置；TG 加 element_index 字段
- [x] **attack+decay 保护 + osc/AEG 同步（08-11）**：AEG 加 pending_release（短按键走完 decay）；
      Oscillator 加 finished（one-shot 采样播完 → kill voice）。174 全过
- [x] **musicbox 击锤修复（08-11）**：核实击锤在 elem0（非 elem1）；根因 AEG attack 中性基准
      `param_to_ms(64,5)`=5ms 削弱击锤瞬态 → 改 1ms。173 全过
- [x] **未应用元素参数排查 + vol_offset 应用（08-11）**：amp 加 element_gain（+0.1dB/单位，
      VOL_OFFSET_GAIN[256] 查表）；musicbox 两元素 vol_offset=0 不受影响。173 全过
  - 08-11 快照的"未应用参数清单"（解析但 0 处应用；多数已在此后被 Element 消费对齐解决）：
    vel_threshold[6]、pitch_fine[9..10]（音准已验证不应用，tone 已覆盖）、pitch_eg_attack/decay[11/12]、
    pitch_mode[15]、range_base[16]、voice_type[17]、peg_center_high[21]、peg_rate3[29]、
    dsp_base[31]、tbl_index[33]、ovr_cutoff[46]、cs_en_1/2[47/48]、ls_en/store/cmp/flag[49-52]、
    rate_idx[64]、tbl_68[68]、eg_phase[69]、wave_pitch[70]、**eg_delay[72]（100% 非 0，18-50——
    语义待确认，应用会延迟所有音色起音——谨慎）**、trig_mode[73]、alt_ovr[74]、off_hi/lo[75/76]、
    fmt_flag[67]（PCM 已转 f32 无需）
  - 注：其中 [70] wave_pitch 已于 08-14 接入 sustain 映射；[14] 于 Phase 2-1 接入 pitch_comp；
    [31]/[33]/[77] 已定案（见"当前待办"）；[11]/[12]/[15]/[16]/[20]/[21] 等已部分对齐
- [x] **musicbox 音色逆向排查（08-11）**：循环语义确认（loopStart=start_point_offset、
      loopEnd=loopStart+loop_length——madaha 原语义正确，曾尝试改后钢琴 bend 音高异常已恢复，179 全过）；
      musicbox 剩余问题（741Hz 非谐波泛音）待 elem0 音量/插值/LPF 排查
- [x] **elem0 持续性实验（08-11）**：musicbox note60 elem0 sustain_level=0 → 频谱从 1044Hz 主导 →
      256Hz 基频主导（接近原版）；elem0 持续保持（sustain 0.811）是泛音过强主因

#### musicbox 音色逆向受阻（2026-08-11 挂起）

- **静态受阻**：S-YXG50 voice 结构布局不可靠（voice[0x68]/[0x69]/[0x6a] 在 FUN_10011f60 用于
  音色库检查 FUN_10016c30——XG_BANK_LSB/GM2_DRUM_TABLE，与 aeg_rel/eg_delay 语义冲突）——
  **需重建 voice 布局**（大工程）；AEG 段推进（voice[0x66] KeyOnDelay 消费）未定位（voice[0x66]
  仅 2 处引用：0x12ea1 复制到 0x69、0xcd00 钳制 ≤voice[0x64]——无渲染递减，疑经初始化 vtable[0x58]
  设入 AEG 结构但参数被忽略）
- **动态受阻**：winedbg 在无 GUI 环境启动卡住（wine 初始化慢/交互；GDB 此前亦受阻 SIGUSR1）——
  需真实 GUI 环境（用户）或 xvfb 才能断点观察 voice[0x66] 行为
- **audit 确认（重要，已移入"当前待办 AEG 对齐"）**：aeg_d1（[54]）→voice[0x60] 标志（≥64→1）、
  voice[0x61]=参数值（疑 Decay1 rate——elem[55] 非"未使用"）；aeg_d2（[56]）→voice[0xc0] 标志
  （≥64→1）**非 sustain**（madaha `1-aeg_d2/127` 映射错）、voice[0xc1]=参数值（疑 Decay2 rate——
  elem[58] 非"未使用"）；aeg_rel（[57]）→voice[0x68]=参数值（release rate）
- **2006LE 逆向进展**（Ghidra x86-32-cpu0x3——S-YXG2006LE.vst，有完整符号）：AEG 段状态机
  `field_0x278`：Attack(0/1)→Decay1(2/3)→Decay2(4/5)→Decay3(6/7)→Release(8/9)→PFDamper(10/b/c)；
  SetupParameterAEGAttack（field_0x280 rate）；CDCFUnit::GetCoefK→ExchangeCutoffToLinear（cutoff 定点→K）
- **S-YXG50 逆向进展（08-11）**：
  - FUN_10012210（元素[69..72] 参数处理）反编译确认：eg_delay → voice[0x66]/voice[0x69] 双写
    （每元素一个 voice——musicbox 两 voice 各自 22/21，KeyOnDelay 错位基础）；voice[0x67]=eg_phase
  - 前期记录错误修正（note_opencode.md）：[69] eg_phase→engine field_0x1c7、[70] wave_pitch→
    engine field_0x1ca（非 voice[0x1C7]/[0x1CA]）
  - vtable[0x4ac]（FUN_10007290 每元素调用）＝ FUN_100120e0（参数查表——非 AEG 渲染）
- 实验结论（无效/撤销）：cutoff 全局映射（64→454/350——破坏钢琴/Dream）、elem1 掐音头（去第二锤）、
  elem1 慢 attack+增益（RMS 匹配但音色不像）——musicbox 暗色非 cutoff 参数（64）直接导致
- **待续**：重建 voice 布局 → AEG 段推进 → KeyOnDelay（eg_delay 22/21）映射/方向 → 实现

#### 响应度 / 竞态

- [x] **A1 竞态修复（on_event 重排，08-13）**：`src/midi/engine.rs` — 动作收集到 pending，hook_exec
      写 back 后统一 swap 再批量发送——ProgramChange 后立即 NoteOn 的竞态窗口消除
- [x] **A2 压力测试**（dual_element_program_change_race_stress，177 项之一）：600 次 0 异常
- 遗留：A1 是正确时序修复，但 1/10 是否完全由该竞态引起待用户真机验证；若仍复现下位嫌疑
      watchdog 睡眠/唤醒时序、key_assign==0 的 kill 路径、复音偷声

#### 其他

- [x] **响度统一至 -14 LUFS（08-13）**：探针实测 -30.59 LUFS → ×6.756（+16.59dB）；
      1kHz 满幅校验 -2.79 vs 参考 -3.01。验证：增益后 -14.00 LUFS 精确命中（181 全过）
- [x] **偶发声音不清晰 bug（TG 未完全复位，08-14 修复）**：ToneGenerator::kill → reset() 完整复位
      （amp.kill + mod_gain/element_gain/lfo_depth 重置 + oscillator.reset（pos/finished/xg_w_phase/PEG）+
      LFO/FEG 复位 + release_elapsed）。验证：tg_reuse_after_steal_has_consistent_volume（ratio=0.978 ✓）
- [x] **移调器优化**（~~LPC/相位声码器~~——当前 WSOLA 足够，已划掉）

### Element 消费对齐（2026-08-12 ~ 08-14 逆向时间线）

#### 疑似字段 → 动态复核（08-12）

- ~~静态搜索未找到 element 直接读取点~~——**已被动态 rwatch 推翻**（下条）。
  注记：pre_voice.rs 曾保持 `_pad` 命名；专利 US-5955691 确认 S-YXG50 架构（ADSR/DCF/波形表）
- [x] **动态复核完成（rwatch 批次）**：全部疑似字段定位——
  - [34..41] 曲线 A（FUN_100164f0，×32 → 12-bit voice[0x18]）
  - [56..63] 曲线 B（FUN_10016490，×2 → 音量键缩 voice[0x70]）
  - [44]/[45] 键跟 A（FUN_10013e20）；[66]/[67] 键跟 B（FUN_10015770）
  - [65] 表索引（FUN_10015940 → 0x100480A0）；[11]/[12] Pitch EG 速率
  - 曲线输入 = note key（变键实验证明）
  - pre_voice.rs 改名：curve_a_*/keyfol_ref/curve_b_*/tbl65_idx/keyfollow_depth
  - 引擎架构：vtable+0x574（FUN_10015a20）= 元素参数转换调度器；运行时元素块 = 每音符 scratch

#### 曲线 A DSP 身份 + 剩余字段裁决（08-12 晚）

- [x] **曲线 A = 包络速率键位缩放参数（EG 段 2/3 log 域速率）**：12-bit → 平滑 FUN_100132a0 →
      engine[0x1b8] → DSP 0x440 → 查表系数 voice+0xa8；FUN_10013580（5 层调制）→ engine[0x1dc] →
      DSP 0x5c1 → 高低 nibble×0x4000 → +0x100/+0x10c（EG 段 2/3 速率）
- [x] **DSP 寄存器映射表已全解**：0x400(音高)/0x440(曲线A)/0x4c0(eg_phase)/0x500(wave_pitch)/
      0x5c0/0x5c1 等，vtable[0x58] 解码器 FUN_10017340
- [x] **剩余字段动态裁决（rwatch 批次 A-E）**：
  - 新确认：[7] 双读取、[9]/[10] 直读、[14] 音高分量（非共鸣）、[18]/[19] 力度缩放、
    [20]/[21] 键跟 C、[22]/[23]/[24]/[27]/[28]/[29] PEG 状态机、[32] 截止调制深度（非 pad）
  - 注记：早期"[17] voice_type 未读"结论已被 08-14 发现推翻（PEG 推进时读，见下）
- 踩坑记录：gdb 挂载偶发被反调试踢掉；批量 pkill 会挂死——改用按 PID kill

#### Phase 0/1（08-12 晚）

- [x] **Phase 0**：element_alignment.md 建立 78 字段消费对齐矩阵（✅12 / ⚠6 / 🔶3 / ❌~25 / —4）
- [x] **Phase 1.5 决议**：S-YXG50 渲染链无共振滤波环节（8-bit 渲染器纯插值）——LPF 是 2006LE
      模型，共鸣参数固定中性 64
- [x] **Phase 1 修正**（176 全过）：[14]→LPF 中性、[25]/[26]/[30]→PEG 停用、[54]→移除 decay 覆盖、
      [56]→移除 sustain 近似、[57]→保留 release；注释修正
- Phase 2 待办（已分解到"一、当前待办"）

#### Phase 2-1（08-12 晚，181 全过）

- [x] **[14] pitch_comp**：重命名（原 filter_resonance），接入 oscillator ratio（`pitch_comp − 64` 分）
- [x] **element_range**（FUN_100140f0）：pitch_mode 0→key；1-4→`range_base + 表0x10047750[mode]×(key−range_base)/100`；
      ≥5→range_base；接入 note_in_cent
- [x] **piecewise_curve**（FUN_100164f0/165b0）：4 断点分段线性曲线，clamp [−0x40,0x3f]，5 单测
- 公式勘误：FUN_10015460 的 EDI=p4=note 结构（非 element）；音高公式 =
  `(note.range−baseKey)×100 + tone + 键表[key%12] + masterTune − 0x40 + elem[14] + note[7] − 0x40 + 键缩放12bit`
  → voice[0x10] → FUN_100161b0 → engine[0x1c4] → DSP reg 0x680

#### Phase 2-2/2-3/2-4（08-12 深夜，185 全过）

- [x] **2-2 截止路径**：elem[13] 用法确认正确（rwatch @0x100141a2）；公式修正 elem[6]=clamp(...)
      为运行时截止累积器；madaha 无需改代码（[13] 已对齐）
- [x] **2-3 键跟随**：`key_follow(key,ref,amount)=(key−ref)×(amount−0x40)>>4`（SAR floor）
      三组同公式；助手+2 单测
- [x] **2-4 PEG 速率核心**：`peg_rate_word`（FUN_10015fe0）——表 0x10048134 查表；
      键跟C 和 elem[19] 汇入速率索引；助手+2 单测
  - 注记：08-12 的表描述（哨兵 0x7E/0x7C、线性 2(N−18)）已被 08-14 完整 88 项 dump 修正
    （哨兵 0x8000/0x1830）

#### H2/H3 对齐 + 程序→音色映射 bug（08-13）

- [x] **H2（双元素 AEG）**：每元素 AEG decay/release 由曲线 A（elem[34..41]）驱动；
      [54] 引擎不读忽略；[56]/[57]=曲线 B x0/x1（非 sustain/release）
- [x] **H3（时长）**：音乐盒长余音（5.37s）来自曲线 A 驱动 release；钢琴快速停止
- [x] **重大发现（程序→音色映射 bug）**：madaha 测试直写 program_number 不触发 program_entry 重建——
      musicbox 测试一直测的是 GS 钢琴（0x98A2）而非真音乐盒（0x9D48）；已修正为走 ProgramChange
      （program 10）。用户 madaha-note72.wav vs yxg50 差异主因 = 播放了错误音色
- 测试：176（bin）+ 9（lib）全过

#### P1/P3 波形对齐（08-13，176+9 全过）

- [x] **P1 每元素音量平衡**（FUN_100156c0）：`volume_param = clamp(vel × [55]/99 + 曲线B(key)×2, 0, 0x80)`
      → element_gain。音乐盒 elem0 0.56 / elem1 0.47 → 尾音 = elem0 正弦主导 ✓
- [x] **P3 包络衰减**：release 修正 `eg_time_ms(max(curve_a,0x18)+0x10)` → 音乐盒 ≈1.06s
      （yxg50 实测 note-off 0.3s 后 ~1.7s 衰减完）
- **P2（elem0 延后）实验结论**：慢攻击产生第二峰（yxg50 无）——驳回；听感差异来自 P1 音量平衡 +
      旧版无 release 衰减
- 残余差异（凹陷深度 28% vs 63%、拍频）→ 见"一、当前待办 3"

#### [31]/[77] 语义定案（08-14）

- [x] **勘误**：ses_130a 曾下结论「[31] 未消费」——错误（只查了 9 个转换器，漏掉 FUN_10013440）。
      note_opencode.md 第 601/639 行早有 [31]@0x10013456、[77]@0x10013530
- 实际语义（动态+静态定案）：见"一、当前待办 1 的 [31]/[33]/[77]"

#### PEG 状态机全链逆向 + 接线（08-14）

- [x] 速率转换定案（FUN_10015ac0）：速率字直接每块累加，不经 FUN_10019580
- [x] helper 修正（peg_rate_table 完整 88 项 dump + 哨兵 0x8000/0x1830）
- [x] **⚠ 发现 elem[17] 实际被读**（FUN_10015e50 读 element[0x11] 选电平倍率，PEG 推进时读）
- [x] **⚠ 发现 elem[26] 实际被读**（初始速率基，FUN_10015c90 区；非"未读"）
- [x] **⚠ 发现 elem[20] 误作 PEG 深度**（实为键跟C amount；初始电平来自 elem[26]/[27]）
- [x] **peg.rs 重写**：4 电平包络（elem[26]→[27]→[28]→[29]→sustain）+ 速率表 + 跳过逻辑 +
      键跟C + elem[19] + elem[17] 倍率；`peg_level`/`peg_vel_sense_*` 助手（11 libmadaha 单测）；
      181 测试全过（Part 音高 EG 保留 OLD 覆盖语义）

---

## 四、已知问题（待查）

### pitchbend 仍轻微改变音量（待查，2026-08-10）

症状：`pitchbend_changes_pitch_not_volume` 回归测试仍失败——
`bend 不应改变音量: 前 0.00082826195 后 0.0006454304`（bend +2 半音后音量 -22%，超过阈值 20%）。
音高部分断言通过（ratio = 2^(2/12) ✓），仅音量受影响。

背景：调制深度公式已修过（`d(v)=(v-64)/64`、`mod_bend_pitch=1.0`），修复前 bend 管音量 ±24dB（×15），
修复后残留 ~-2.3dB 的小幅音量联动。

待查方向：
1. bend 的 filter_control / amplitude_control 调制路径是否仍泄漏到音量（`mod_bend_filter/amp` 计算）
2. 测试测量窗口（sustain 尾段包络自然衰减可能被误判为音量变化）——确认是测量窗口问题还是真实调制
3. 8-bit 采样量化电平差异（bend 改变采样步进 → 输出电平微变，属正常，需排除）
