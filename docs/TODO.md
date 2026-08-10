# Madaha TODO / 暂不实现清单

决策记录（2026-08-05）：Madaha 是 S-YXG2006LE 的超集实现。
以下项目经评审后决定"暂不实现"或"待办"，按优先级排序。

## 暂不实现（用户决策）

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

### 和声类效果（XG2.0 Harmony/Vocoder）——已完成（2026-08-05 四轮）

- VOCODER_HARMONY（简化：移调 + band-pass 染色）/ CHORDAL_HARMONY /
  DETUNE_HARMONY / CHROMATIC_HARMONY / TALKING_MODULATOR（formant 滤波）
- 和声判断：活跃音符（pitch-class）→ ChordDetector（major/minor 模板）
- 和声部生成：WSOLA 移调器（共用组件）

## 已完成（2026-08-05）

- [x] XG2.0 串联组合（阶段 3a）：Distortion/Overdrive/V-Distortion/Wah/Compressor →
      Delay/Rotary 串联（SerialChain）；Tempo 变体用普通延迟（无 tempo 源）
- [x] Dyna 家族（阶段 3b）：DynaFilter（LFO 扫频 LPF）/ DynaFlanger / DynaPhaser /
      DynaRingModulator
- [x] 调制/空间类（阶段 3c，xg20_effects.rs 近似）：RingModulator、EnsembleDetune、
      Ambience、WideStereo、ThreeDManual/Auto、VibeVibrate、LoFi、Slice、Isolator、
      LowResolution、DigitalTurntable/Scratch、MultiBandComp、TempoFlanger/Phaser、
      VFlanger、DualRotorSpeaker1/2
- [x] GS 兼容：`gs_xg_addr_remap` 扩展（0x41 鼓 7 参数 + 0x42 Scale Tuning +
      0x43 Part Tuning）+ GS reset 分支
- [x] Reverb/Chorus 扩展类型（0x0D-0x12 / 0x10-0x14，2006LE 权威值）
- [x] xg_20_data / xg_distortion_data 恢复（B5b merge 回归）

## 已完成（2026-08-05 二轮）

- [x] DC 偏移修正（DcBlocker 主总线，XG Spec 依据，固定启用）
- [x] 测试盲区补测（config/encode/LFO/voice_manager/XG RAM +16 测试；
      修复 MidiConfig::check_scoring 无限递归 bug）
- [x] XG2.0 参数布局精化（49 处默认值按 XG Spec 修正；串联类型布局
      Delay/Distortion 参数分配修正——Dist+Delay=P1-3 延迟/P4-9 失真/P10 Dry/Wet、
      V-Dist=P1-5/P6-11 延迟、Wah/Comp 串联=P1-3 延迟/P4-7 失真；
      dev_docs/xg20_params.md）
- [x] OSC 绑定（bind_part + is_drum 读 part_mode）
- [x] LFO Key-sync 接入（note-on 相位复位）
- [x] scoring AEG 阶段保护（Attack 期权重加倍）
- [x] Variation 混响类类型切换默认加载（Chorus 布局）

## 已完成（2026-08-05 三轮）

- [x] Dyna 家族包络驱动：DynaEnv（Attack/Release 平滑 + Threshold + Sensitivity），
      DynaFilter 包络扫频、DynaFlanger/Phaser 包络调制（DynaChorusEffect/DynaPhaserEffect）、
      DynaRingMod 包络
- [x] Wah/Compressor 前级串联：SerialChain 3 级（Wah→Dist→Delay / Comp→Dist→Delay），
      P11-14 附加参数接入（Wah Sensitivity/Cutoff/Resonance/Release、Comp Attack/Release/Threshold/Ratio）
- [x] TWO WAY ROTARY 布局核对：P1 转速、P14-16 失真前级（Drive/LPF/Output）
- [x] xg20_effects.rs 索引核对：Ensemble（P1 detune/P2-3 delay）、LoFi（P2 word/P4 LPF）、
      RingMod（P5 LFO freq）、LowRes（P4 resolution）、Ambience（P1 delay/fb 固定）、
      Isolator（3 段电平模型）
- [x] 多段压缩（MultiBandCompEffect：3 段分频 + 软拐点压缩 + 段增益）

## 已完成（2026-08-05 四轮）

- [x] WSOLA 移调器（core/wsola.rs，共用组件——PitchChange + Harmony 受益）：
      读指针速率扫描 + 追尾对齐跳回（SAD 相似段搜索 + Hann cross-fade）
- [x] 和声类效果（harmony_effect.rs）：ChordDetector + HarmonyEffect
      （WSOLA×3 声部 + Vibrato + Lead/Harmony 平衡）+ Vocoder 简化 +
      TalkingModulator（formant 滤波）
- [x] 活跃音符采集（audio_render 每块收集 → EffectProcessor::set_active_notes）
- [x] PitchChangeEffect 迁移到 WSOLA

## 已完成（2026-08-05 五轮）

- [x] config 音频参数扩展：master_volume（0.05-4.0 校验）、soft_clip、
      dc_blocker（可关）、alsa_buffer_frames（2 的幂校验）；
      GainSink（输出增益+软限幅，所有后端统一）、dc_enabled 接线

## 待办（计划内）

- [ ] ~~移调器进一步优化（若音质需要）：LPC/相位声码器（当前 WSOLA 足够）~~
- [ ] GS 的 2006LE 数据文件读取
- [ ] 实现 VA 算法
  - 已经在它处完成逆向，必要时可以用浮点数实现
  - 这玩意我感觉单独拉出来做LV2还行，也许稍微扩展一下木吉他也能搞了，
    填补了 Linux 环境没有原生物理建模音源的空白
- [ ] 根据 Dexed 实现 DX 模块模拟
- [ ] OPL3 模块
- [ ] WinGroove 模块，主引擎不再加载 WinGroove TPD 文件
  - 已知：WinGroove 采样格式为有符号 8 位（-128 至 +127），22050Hz
  - [ ] TPD 文件字段需要进一步分析

## 已知问题

### 采样/音质综合问题（2026-08-07，待修）

用户实际听感（钢琴音色）：
1. 整体音质像电话声（窄带/高频感），切换插值算法（linear/hermite/lanczos）无效果
2. sustain 不生效（长音不持续）
3. 声音有飘忽感
4. pitchbend 不正确，听起来像在控制音量大小
5. 整体有破音感（弹奏 dump 峰值达 3.689，远超 1.0；soft_clip=false）

已确认事实：
- 波形本身是干净的正弦（dump 分析），不是 PCM 字节破坏
- S-YXG50 采样为 8-bit；set_wave 的 0x80（22050 连续读）/0x00（44100 跳字节读）解析正确
- 弹奏 dump：基频 320Hz（弹 C4 时 ≈ +3.4 半音偏移）

### 采样加载（已修，2026-08-07 用户）

`libmadaha/src/yxg50/sample_meta.rs` set_wave / `drum_setup.rs`：`sample_rate_for_sample` 是 flag——
- `0x80` 位为 1：连续逐字节读取（22050Hz）
- `0x80` 位为 0：**每 2 字节取第 1 个字节**拼出 8-bit 波形（第 2 字节是 SMID 索引，madaha 不使用），同时 `start_point_offset/loop_length` 字节→样本（÷2）
当前实现：`wp.chunks_exact(2).map(|b| u8_to_f32(b[0]))`（两个文件一致），已正确。

### 调制深度公式错误（已修，2026-08-07）

`src/audio/tone_generator/tone_generator.rs:434-436`：`d(v) = v/64`
XG 规范（docs/XGSpec2.0.md）：Filter/Amplitude Control 为 `-100%..+100%`、中心 `0x40=0`（无调制）；MW Pitch Control 为 `-24..+24` 半音、`0x40=0`。
代码 `d(0x40)=1.0` 使默认参数变成"全量调制"：
- bend/mw 全量调制音量 ±24dB（"pitchbend 像管音量"）
- `mod_gain = 10^(db/20)`，+24dB = ×15.8 增益（破音）
- 调制信号驱动增益/音高大幅波动（飘忽）

修复（2026-08-07）：filter/amp 控制改 `(v-64)/64`（中心 0）；MW/CAT/PAT/AC1/AC2/CBC1/CBC2 pitch 控制改 `(v-64)` 半音（×100 cents）；`mod_bend_pitch` 固定 1.0（bend_cent 已含 08 pp 23 范围，避免双重应用）；HPF 深度同 filter 映射。offset level（0A pp 3F-44）公式原本正确未动。
回归测试：`pitchbend_changes_pitch_not_volume`（bend +2 半音 → 音高比 2^(2/12) ✓、音量变化 <20% ✓，修复前 ±24dB ≈ ×15）。


### 音高偏移（已修，2026-08-07）

根因链（e2e 实测 + Ghidra 验证）：
1. **MIDI 输入 note 映射 -12**：Note 枚举是 MIDI+12 键系（C4=72 内部 = MIDI 60），但 ALSA/pipewire 源直接 `try_from(MIDI key)` → MIDI 48 变成内部 48（= MIDI 36），**输入音符整体低 12 半音**。修复：两个源均 `+12`（src/midi/source/alsa.rs、src/midi/source.rs）。回归：`midi_48_maps_to_internal_60_and_pitch`
2. **base_note 基准错误**：此前用 `range_base`（elem[16]，仅音符区间中心）→ 改回 **seg16 baseKey**（采样 PCM 录制于 baseKey 音高）。修复后音程正确（内部 60→72 输出比 3.95→1.99）
3. **coarse 符号**：移回 note 侧（`+coarse`）；Ghidra `FUN_10006d00` 确认 `voice[0xB4] = elem[35]-64` 为加法
4. **baseKey→频率映射校准**：+440 cents（seg16 baseKey 是 MIDI 键而音符是 +12 内部键系）

验证（e2e 自相关基频，期望 MIDI 键频率）：内部 48/60/69/72 → 64.9/129.3/218.3/257.9Hz（-0.13~-0.25 半音）。对照：wine 中真实 S-YXG50（ALSA 输入）音高正确，与 madaha 修复后一致。141 测试通过。

### ALSA MIDI pitchbend 中心值（已修，2026-08-07 用户，待回归）

`src/midi/source/alsa.rs`：ALSA `EvCtrl.value` 为有符号 i32（-8192..+8191，中心 0），
此前直接 `as u16` 导致负值变 65535 附近。已改为 `(pitch.value + 0x2000) as u16`。

pipewire MIDI 源（parse_midi_bytes）中心值解析正确，无需改动。


### pitchbend 仍轻微改变音量（待查，2026-08-10）

症状：`pitchbend_changes_pitch_not_volume` 回归测试仍失败——
`bend 不应改变音量: 前 0.00082826195 后 0.0006454304`（bend +2 半音后音量 -22%，超过阈值 20%）。
音高部分断言通过（ratio = 2^(2/12) ✓），仅音量受影响。

背景：调制深度公式已修过（见上"调制深度公式错误"节：`d(v)=(v-64)/64`、`mod_bend_pitch=1.0`），
修复前 bend 管音量 ±24dB（×15），修复后残留 ~-2.3dB 的小幅音量联动。

待查方向：
1. bend 的 filter_control / amplitude_control 调制路径是否仍泄漏到音量（`mod_bend_filter/amp` 计算）
2. 测试测量窗口（sustain 尾段包络自然衰减可能被误判为音量变化）——确认是测量窗口问题还是真实调制
3. 8-bit 采样量化电平差异（bend 改变采样步进 → 输出电平微变，属正常，需排除）



