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
      dc_blocker（可关）、alsa_buffer_frames（2 的幂校验）、jack_client_name；
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
