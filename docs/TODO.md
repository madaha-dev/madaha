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

## 里程碑（2026-08-10）：简单 MIDI 文件可播放

### 双元素音色检查（2026-08-10，Dream LSB=41 prog=0 验证通过）
修复的 4 个深层 bug：
1. **`pre_voice.rs` 双元素判定**：`& 0x3 == 3` → `& 0x2 != 0`（Ghidra FUN_10016fa0：header[1] bit1）
2. **`get_program_index` LSB `+1` 偏移**：Dream（lsb=41）误查 lsb_table[42] → 直接用 lsb（sxgparser 一致）
3. **`controller.rs` rcv_bank_select 的 `part_mode != 0`**：旋律 part（mode=0）的 CC32 被拒 → 移除
4. **`MultiPart hook_check` 0x03 program change**：相同 program（0==0）不触发 ChangeProgram →
   无条件应用（重选音色有效）；顺带 0x01/0x02 去掉 `part_mode == 0`
回归：dual_element_dream_allocates_two_voices（双元素加载+2 voice+发声）/
dual_element_dream_loads_two_tone_generators

### 双元素 NoteOff 组释放（2026-08-10 已修）
- **bug**：release_handler 只释放最早一个 Running TG（min_by_key attack_time）——双元素时
  NoteOff 只释放元素之一，另一元素残留（NoteOn vel=0 同样失效）
- **修复**：TG 加 `note_on_id`（NoteOn 组标识）；audio_render `note_on_counter` 为每个 NoteOn
  分配唯一 id（双元素 voice 共享）；`release_note` 释放最早 note_on_id 的**整组** voice
- 语义保留：stacking（连按同音）每个 NoteOff 释放一组；sustain 挂起/松开、CC123/120 同组语义
- 回归：dual_element_noteoff_releases_both_voices / dual_element_noteon_zero_velocity_releases_both

- [x] **简单 MIDI 文件已可播放**：仅含 NoteOn/NoteOff 的单 channel 文件（441 测试全绿）
- [x] 后端重构：音频 cpal（替换 ALSA/PipeWire）、MIDI ALSA（删 PipeWire）；后端/trait 简化
- [x] 音高链修复（最终版）：`get_sample_rate → 44100`（PCM 22050 录、44100 播 = ×2 trick）、
      `base_note = baseKey`（采样准）、公式 `note - base + tone`（seg16 data[2] = 录制微调音分）、
      移除 wave_pitch/pitch_offset/fine 音分项、detune 默认 0x80 表索引 bug（-50 音分）
- [x] 音准客观验证（numpy）：48/60/69/81 → 130.75/261.66/440.04/879.85Hz（±0.5 音分内）
- [x] LPF/HPF 按 2006LE 重写：Chamberlin 二阶 SVF（`y1=(x−K·y1−y2)·f+y1; y2=y1·f+y2`）、
      `K=clamp(3−2f,·,2.0)`、`ExchangeResonanceToLinear` 忠实移植（Ghidra 核实非梳状）；
      消除 Simper SVF 高 Q 削波（原 64 中心 → Q=5.29）
- [x] PEG 修复（音头变调）：XG 语义 = 初始电平滑向 0（原实现 0→+100 反了）、
      level 参数化（elem[20] depth，64 中性 → 无滑音）、rate 指数近似暂留
- [x] 复音释放修复：`release_handler` 过滤 Running（快速同音连按 attack_time 相同
      导致第二个 NoteOff 选同一 TG → 音符永不释放）；保留"优先释放最早音符"
- [x] 延迟修复：cpal ring 65536→256 帧（渲染 backpressure 钳制到实时，按下即响）
- [x] tone_generator 链式重构：`self.osc().lpf().hpf().amp().eq().pan()`（逐级排查）
- [x] Lanczos 内核 LUT 化（4096 表 + 线性插值，音色不变）

## 计划（2026-08-10）：XG 功能完全覆盖（四阶段）

> 目标：当前 MIDI 实现功能的完全覆盖；验证手段以录音 A/B 对比为主（人耳），
> 客观断言（频率/RMS/包络/无 NaN）只做回归保护。当前只能播放单 channel
> 简单 MIDI（NoteOn/NoteOff），先单 channel 做深，多 channel 后置。

### 前置：测试框架
- [ ] e2e 公共模块：`render_sequence(events)→音频`、`write_wav`（录音对比基础）、
      断言助手（dft_freq_in / RMS / 包络时间 / 无 NaN）
- [ ] 鲁棒性：音色库遍历（128 音色 × 代表键 × vel 层 → 无 panic/NaN/超幅）、
      确定性随机事件模糊（LCG）、压力（快速事件/polyphony 打满）
- [ ] 录音对比工作流脚本化：确定性音序 → wav → 与原版 yxg50 录音 A/B

### 阶段 1：非 SysEx MIDI 指令校准
现状：PitchBend ✓（1 测试）；CC 覆盖较全（1/5/7/10/11/32/64/65/66/67/71-75/84/91/93/94/98-101/120/121/123）；
RPN 0-4 ✓ 无测试；NRPN→RAM ✓ 无测试；CAT/PAT ✓ 无测试
- [x] **CC64 sustain 挂起释放（2026-08-10 已实现）**：NoteOff 踏板踩下挂起（EG 保持）、松开批量 Release；
      **CC#123 释放挂起音符 + 清空队列；CC#120 kill 挂起 + 清空**（用户报告的 bug 已修）；
      回归：sustain_pedal_holds_noteoff_until_release / cc123_releases_sustain_held_notes /
      cc120_kills_sustain_held_notes
- [x] **校准（2026-08-10）**：bend 默认 ±2 半音 ✓（已有测试）、**RPN#0 改 bend 范围**（rpn0_sets_pitchbend_range）、
      **CC#7 volume 实时性修复**（amp.update 每块传 volume——原 note-on 快照）、**CC#10 pan 实时性修复**
      （update_amp_and_sends 每块 set——原 play 快照）、CC#11 expression ✓、CC#10 极左/极右断言
      （pan=1 极左/127 极右；0=random）、CAT/PAT 存储断言
      **用户实测验收（2026-08-10）**：pitchbend 默认行为符合预期（不考虑 RPN 时）
- [x] **CC66 sostenuto（2026-08-10 已实现）**：只保持踏板踩下时已响的音符（快照机制），
      新音符正常释放；松开批量 Release；CC123/120 同样清空（sostenuto_holds_only_pedal_time_notes）
- [ ] CC67 soft pedal 接线核对、
      CC71-75（谐波/起音/释音/亮度 → 08 pp 18-1C RAM）断言测试
- [ ] 每指令端到端断言 + 录音 A/B

#### AEG 2006LE 对齐（逆向进展 2026-08-10，实现待续）
- [ ] **sustain 机制已逆向**：sustain_mode（VPRM +0xf 高 4 位：0=无/1=恒保持/2=制音器）、
      KeyOffMode（mode1 或 mode2+踏板>0x3F → 保持）、PianoDamperMode（mode2+踏板）
- [ ] **段结构**：KeyOnDelay → Attack → Decay1 → Decay2 → Decay3（现仅 Attack/Decay/Sustain/Release）
- [ ] **驱动模型**：rate-based（每周期增量，rate 域 0-0x7f）；`_gfAEGAttackCycle` 表
      （0x91360，rate 0→699050 周期、127→129 周期）已定位，Decay/Release 表待提取
- [ ] **D2L（sustain level）证据**：S-YXG50 elem[56]（aeg_d2）是"覆写使能+二值化"标志
      （FUN_10006e80：非 0 时 value>0x3F → voice[0xc0]=1）——**非 D2L**；
      madaha 的 `sustain_level = 1 - aeg_d2/127` 映射疑误（钢琴 aeg_d2=106 → 0.165），
      D2L 真正来源待确认（元素缺失 → 默认/固定？）
- [ ] 实现：aeg.rs 重构（5 段 + rate 驱动 + D2L + sustain_mode），待续逆向（rate 表/驱动/段目标链）
- [ ] **sustain 保持行为差异（2026-08-10 逆向确认）**：
  - 核心机制一致：wave loop 循环 + AEG 保持（state 9 停住不衰减）
  - **制音器模式未实现**：2006LE sustain_mode=2 时走 `ShiftAEGSegmentPFDamper`（state 10 段切换，
    钢琴踩踏板专用）——madaha 当前统一挂起，无 PFDamper 段
  - **force damp 未对齐**：`SetupForceDampAEG` 强制制音时 release 目标 ≥ 0x60（voice+0x190 下限）——
    madaha 的 CC123 直接 release/kill，无电平下限调整
- [x] **PianoDamperMode 评估（2026-08-10 静态深挖定案，暂不实现）**：
  - 概念：钢琴制音器模拟——CC64 踩下音符保持（制音器抬起），松开走制音器专用释放段
  - madaha 已有基础：挂起机制（踩下保持 + 松开标准 Release）——基本制音器效果 ✓
  - **静态深挖结论（S-YXG50.dll）**：NoteOff 链（FUN_1000c3a0→vtable[0x3d0] 释放前检查→
    元素循环 vtable[0x3c4] 释放）与 AEG 阶段机（FUN_10007560→PTR_FUN_10041bb0 表）均无
    sustain_mode/damper 段；元素参数表（0-77）无 damper 字段——**S-YXG50 为 XG 全局
    hold（CC64→engine 状态，所有音色统一）**，madaha 现有全局挂起已匹配，无 PFDamper
  - **决策**：暂不实现 PianoDamperMode；`Element` 已预留 `sustain_mode` 字段
    （pre_voice.rs，S-YXG50 解析=0，语义 0=无/1=恒保持/2=制音器），
    待 2006LE 数据文件读取时填充并实现——**有就用，没有就不用**
- [x] **PortUnsubscribed 核实（2026-08-11）**：已是 release 语义（非 kill）——
      engine.rs:177 发 ReleaseAll（CC#123 语义），release_all_handler 对全部 Running TG
      调 t.release()（AEG release 段衰减，非立即静音），sustain 队列清空，测试断言 Releasing ✓
- [x] **elem0 持续性实验（2026-08-11，确认方向）**：
  - **实验**：musicbox note60——elem0 AEG sustain_level=0（decay 衰减到静音）——
    **mix 频谱从 1044Hz 主导 → 256Hz 基频主导**（接近原版 259 主导、泛音弱）——
    **elem0 的持续保持（sustain 0.811）是泛音过强（音色不对）的主因**
  - **静态边界**：elem0/elem1 元素参数全同（aeg_d1/d2/rel、[11][12]、filter、eg_amp_en
    80/64 只是使能、aeg_d2→voice[0xc0] 标志非 D2L）——**无参数可区分"击锤层衰减停"**
  - **实现待定**：① 听感确认（/tmp/madaha_musicbox_s0_mix.wav vs 原版）② D2L 真实
    来源（AEG 初始化 FUN_100103f0 表驱动未暴露）③ 击锤层识别启发式
  - 175 全过
- [x] **musicbox 音色逆向排查（2026-08-11，用户要求直接逆向）**：
  - **循环语义确认（重要）**：逆向 S-YXG50 渲染器 FUN_1001a7c0——循环回绕
    `samplePos >= loopEnd → samplePos -= loopEnd; fracPos += loopStart`——**循环段
    [loopStart, loopEnd]**；**loopStart = start_point_offset（文件 data[9-11]
    loop_start 位置对应采样内 spo——drum_setup.rs:109 PCM 切片 `loop_start-spo..
    loop_start+loop_length` 证实）、loopEnd = loopStart + loop_length（data[6-8]）**
    ——**madaha 原语义（loop_point=spo、loop_length=data[6-8]）正确**；
    曾尝试改 [data[6-8], len]（新语义）导致钢琴 bend 测试音高异常（399.8→385.8
    不单调）——**已恢复**（179 全过）
  - **musicbox 音色不对的剩余问题**（非循环）：elem0 输出含 741Hz 非谐波泛音
    （原版频谱完全没有——740/1815/1440）——**741 出现在循环前（击锤区变速产物）**，
    与循环区无关；候选：① elem0 音量/持续性（原版击锤层弱/音头后停）② 8-bit 插值
    伪影（madaha 线性插值 vs S-YXG50 点采样插值 0x5C XOR）③ LPF cutoff 映射
    （中性 64 → 1120Hz，原版泛音衰减更强 260:1043 ≈ 6.5:1）
  - **原版录音数据**（/tmp/yxg50-musicbox_note60.wav）：3 个 note60、单音 ~1.7s 余音、
    260Hz 主体主导（击锤与主体同时 0ms 起）——madaha release（610ms）仍短于原版
- [x] **attack+decay 保护 + osc/AEG 同步（2026-08-11，用户观察）**：
  - **attack+decay 保护**：AEG 加 `pending_release`——noteoff 在 Attack/Decay 阶段
    不打断，走完 decay 到 sustain 才 release（XG 短按键行为——musicbox 极短按键
    仍完整发出击锤+叮）
  - **osc/AEG 同步**：Oscillator 加 `finished`（one-shot 采样播完）→ advance_runtime
    kill voice——采样耗尽即结束（AEG 不再在静音源上走流程，省渲染/复音）
  - **元素非同时查证**：eg_delay（KeyOnDelay）链确认（0x100122DF→FUN_10012670→
    voice[0x66]/[0x69]，上限 0x37=55）——musicbox 两元素 22/21 几乎同（同时延迟，
    非先后）；trig_mode/vel_threshold 两元素相同——静态无"选择/交替"证据——
    **最可能：听感时序**（击锤短促衰减快、主体持续——同时开始感知"咔→叮"）
  - 测试：short_note_preserves_attack_decay + aeg_release_to_zero 适配——174 全过
- [x] **musicbox 击锤修复（2026-08-11，用户纠正：击锤在 elem0）**：
  - **核实**：elem0（base 80、len 4460、非循环区 3613）开头 0-0.9ms 即击锤瞬态
    （20 样本窗峰值 0.41，持续 0.4-0.78 金属衰减）；elem1（base 73、len 6614）
    是音调主体（开头 0.94 为"叮"的起振——非击锤）
  - **根因**：AEG attack 中性基准 `param_to_ms(64, 5)` = **5ms**——击锤瞬态
    （前 1ms）只爬升到 18%（削弱 82%）→ 击锤弱
  - **修复**：attack 中性基准 5ms → **1ms**（XG 中性 attack 必须足够快以保留
    打击乐瞬态）——elem0 第 1ms 输出提升 ~5.5 倍；所有音色音头更锐利
  - 测试：173 全过；**待用户实机验证**（musicbox 击锤明显度）
- [x] **未应用元素参数排查 + vol_offset 应用（2026-08-11，用户发现）**：
  - **vol_offset（element[8]，音量偏移）已应用**：amp 加 `element_gain` 字段（note-on
    快照，+0.1dB/单位——**查表** VOL_OFFSET_GAIN[256]，避免每 noteon 的 powf）；
    数据分布：25% 元素非 0（4..56 为主，127 少数）——musicbox 两元素为 0 不受影响
  - **其余未应用参数清单**（解析但 0 处应用）：vel_threshold[6]（99% 非 0）、
    pitch_fine[9..10]（99.4% 非 0——**音准已验证不应用**，tone 已覆盖）、
    pitch_eg_attack/decay[11/12]、pitch_mode[15]、range_base[16]、voice_type[17]、
    peg_center_high[21]、peg_rate3[29]、dsp_base[31]、tbl_index[33]、ovr_cutoff[46]、
    cs_en_1/2[47/48]、ls_en/store/cmp/flag[49-52]、rate_idx[64]、tbl_68[68]、
    eg_phase[69]、wave_pitch[70]、**eg_delay[72]（100% 非 0，18-50——语义待确认，
    应用会延迟所有音色起音——谨慎）**、trig_mode[73]、alt_ovr[74]、off_hi/lo[75/76]
    （6% 非 0）、fmt_flag[67]（PCM 已转 f32 无需）
  - musicbox 击锤定位：**击锤瞬态在 elem1 开头**（rms[0..40]=0.457 强），elem0 是
    主体循环层（开头弱 0.188）——5ms AEG attack 削弱前 1ms 瞬态（候选修复：attack
    参数映射，待用户验证 eg_time_ms 修复后听感再定）
  - 测试：amp element_gain_applies——173 全过
- [x] **TG 分配：去 xorshift 随机 → 关联元素连续分配（2026-08-11，用户要求）**：
  - `find_idle_voice` 去掉 xorshift 随机起点（顺序扫描 + 保留 idle_buffer 缓冲轮）
  - 新增 `find_adjacent_idle(prev)`：双元素第二个元素优先落在第一个元素相邻槽
    （关联放置——缓存友好、元素顺序可预测）；`random_state` 字段移除
  - TG 加 `element_index` 字段（play 时记录，供角色识别）
- [ ] **musicbox 音色（4 次谐波/击锤/双击锤）——2006LE 对照（逆向中）**：
  - **2006LE 逆向进展**（Ghidra x86-32-cpu0x3——S-YXG2006LE.vst，有完整符号）：
    AEG 段状态机 `field_0x278`：Attack(0/1)→Decay1(2/3)→Decay2(4/5)→Decay3(6/7)→
    Release(8/9)→PFDamper(10/b/c)；SetupParameterAEGAttack（field_0x280 rate）；
    CDCFUnit::GetCoefK→ExchangeCutoffToLinear（cutoff 定点→K，16.16 定点 2^(0x17-int)）
  - **待续**：① 元素→cutoff 定点映射（SetupKeyOn 调用者链）——精确 cutoff 表
    （musicbox 4 次谐波 0.06 的 fc）；② attack rate（field_0x280）来源——每元素
    attack（musicbox elem1 慢起/双击锤）；③ KeyOnDelay（eg_delay[72] ↔ 2006LE）
  - **渲染 LPF 矛盾未解**：musicbox elem1 cutoff 恒定 351 时 4 次谐波仅 -4.7dB
    （LPF 单测同参数 -21dB）——需专项调试（渲染路径 vs 单测）
  - 实验结论（无效/撤销）：cutoff 全局映射（64→454/350——破坏钢琴/Dream）、
    elem1 掐音头（去第二锤）、elem1 慢 attack+增益（RMS 匹配但音色不像）——
    musicbox 暗色非 cutoff 参数（64）直接导致
- [x] **元素级 AEG 时长映射系统性修复（2026-08-11，用户发现 musicbox 长 release 缺失）**：
  - **根因**：`eg_time_ms(v) = 2000×2^(-v/8)` 曲线过陡——**全部 22474 个元素都有
    aeg_d1/aeg_rel 覆写**，旧映射使元素级 attack/decay/release 普遍过短 20-50 倍
    （aeg_rel=48 主流值：31ms→**610ms**；musicbox aeg_rel=58：13ms→**312ms**——
    "长 release 缺失"、"击锤弱"（decay 7.8ms→209ms 音头保持更久）均由此引起）
  - **修复**：`eg_time_ms` 改为 S-YXG2006LE `_gfAEGAttackCycle` 指数表语义：
    `cycles = 699050 × (129/699050)^(rate/127)`，÷44100Hz（rate 0→15.85s、127→2.92ms）
  - **连带**：key_assign=0 单音替换 `release()`→`kill()`（XG mono 语义——长 release 下
    旧音不得与新音头叠音）；watchdog/note_assign 测试适配长 release 时序
  - 测试：musicbox_element_release_uses_xg_rate_table（release ≥250ms + 尾音 >100ms）
    ——173 全过；**待用户实机验证**（musicbox 余音/击锤 + 其他乐器）
  - 现象：note 62 及以下音高不变化（VST/Yamaha 数据：prog 9 为单采样 base_note=103
    全键覆盖——note 62 与 base 差 -4920 音分，超出 `cents_to_ratio` 表 ±4096 音分
    （±3.4 八度）被 clamp → 62 及以下全部同一 ratio → 同一音高）
  - **修复**：`cents_to_ratio` 表 ±4096 → **±11520 音分（±8 八度，23041 项）**——
    note 62/60 恢复精确音高（293.6/261.6Hz）；更低键因短采样（5636 样本无循环）
    慢播放呈瞬态"叮"，但各键频率已不同（不再同一音）
  - 测试：cents_to_ratio_bounds 扩展（±12000 边界 + 低端单调性断言）——171 全过
- [x] **切换音色 + 滑音音色混乱修复（2026-08-11）**：
  - **修复 A（脏 voice 防御）**：`play()` 的 sample 绑定链三处 if（program_entry / key /
    sample_at）原无 else——复用 TG 绑定失败时保留旧音色 sample 仍置 Running 发声
    → 加 else 分支 `kill() + return`（无 key / vel 范围不匹配 / 无 program 均静音）
  - **修复 B（xorshift 分配 + 缓冲，用户方案）**：空闲 voice 分配不再固定取列表头——
    `find_idle_voice()` 从 xorshift 随机起点环形两轮扫描：① 优先"空闲已久"（idle_since
    超过 idle_buffer=50ms 的 voice——刚释放的获得缓冲空间）；② 缓冲池耗尽时退化任意
    Idle；全忙才 steal。TG 加 `idle_since`（kill 时更新）
  - 测试：program_switch_reused_voice_stays_silent_on_missing_key（S-YXG50 数据全键覆盖
    时 SKIP）/ idle_allocation_prefers_buffered_voice（确定性：老空闲必被第一轮选中）
    ——171 全过
- [x] **震音卡顿修复（2026-08-11）**：Midi3.mid 后半段快速震音（42 note 事件/s）导致
      渲染欠载——**根因**：`fast_sine.rs` 的 `pub const SINE_TABLE: LazyLock` —— **const +
      LazyLock 组合使每次 deref 重新初始化表**（实测 ~65us/次）；`pan.set`（update_block_
      parameters 每 64 帧 × 每 TG）调用 fast_cos/fast_sin（各 1 次 deref）→ 满复音时
      每帧 ~131us 额外开销 → x6 欠载
  - **修复**：`const` → `static`（一次初始化）——pan.set 65us → 5.6ns（24000 倍）
  - **实测**：64 复音 1s 渲染 release 基准 x6.01 → **x0.21**（28 倍余量）
  - 二分定位过程：TG 链手动 x0.14 vs render_frame x6 → update_block_parameters
    ~127us/次 → update_amp_and_sends → pan.set → fast_sin/cos → const LazyLock deref
  - 同类检查：其余 LazyLock 均为 static（LANCZOS/CENTS_TO_RATIO/TG_GAIN/XG_GAIN/PAN_GAIN）✓
- [x] **渲染线程 watchdog 休眠（2026-08-11）**：所有 TG Idle 且静音窗口（sleep_delay_ms
      默认 200ms——效果器尾音窗口）过后，渲染线程 sleep_idle 阻塞等待事件（recv_timeout
      200ms 轮询）；MIDI/音频事件到达立即处理并唤醒（事件驱动）。cpal 回调欠载补静音
  - AudioRender：needs_render() / sleep_idle() / tick_idle_watchdog()（idle_elapsed 累计）
  - config：audio.sleep_delay_ms（0=禁用休眠）；synth.rs 循环按 needs_render 分支
  - 测试：watchdog_sleeps_after_silence_window / sleep_idle_wakes_on_event（169 全过）
- [x] **damper 策略实现（2026-08-11）**：sustain_mode 字段优先（1=恒保持/2=damper），
      S-YXG50 无字段（0）→ program 0-7（XG 钢琴组）fallback 到 damper
  - AEG 加 `Damp` 段（aeg.rs）：挂起时从 sustain_level 慢衰减到 0（damp_time=3s 近似，
    **待 2006LE 数据对齐 PFDamper 参数**）；Release 改为从 note_off 当前电平衰减
  - SampleMeta/TG 传递 sustain_mode；release_handler 挂起时按策略设 damper_hold
  - 测试：sustain_damper_decays_piano_while_held / sustain_hold_freezes_non_piano /
    sustain_mode_wins_over_program_fallback + AEG 单测（167 全过）
- [x] **program number 决定 sustain 策略评估（2026-08-10，结论：已作为 fallback 实现）**：
  - 用户观察：S-YXG50 钢琴踩踏板（damper）波形衰减明显——考虑用 program 0-7 判断 damper 策略
  - **结论**：衰减机制已存在——挂起时 TG 仍 Running、`advance_runtime`/AEG 每帧推进
    （tone_generator.rs:701-734）→ AEG 自然衰减到 sustain level——**钢琴"明显衰减"来自
    元素 AEG 参数**（sustain level 低），非 damper 段；**根因候选是 D2L 映射疑误**
    （`1 - aeg_d2/127`，见上）——先修 D2L，再看听感
  - **鼓**：YAMAHA 鼓忽略列表（part.rs:147 `5|32|65|67|84|126|127`）不含 CC64/66 →
    鼓响应 sustain——保持统一挂起（鼓 AEG 参数天然短衰减）；曾尝试鼓 gate 后撤销
  - **预留**：若 D2L 修后钢琴听感仍差 → 用 program 0-7 触发 damper 释放段
    （sustain_mode 字段的触发源，release_handler 加特判）

#### HPF 2006LE 对齐（2026-08-10 用户反馈：低频过滤过狠）
- [ ] 逆向 2006LE 的 HPF 行为（Mac 版 CDCFUnit 的 high 输出已确认；但 HPF 参数映射/
      cutoff 范围需核对——当前 `cutoff_param_to_hz` 20Hz-10kHz 与 cutoff 计算可能过狠）
- [ ] 对齐：HPF 截止映射、与 LPF 的共享 CLFO 调制、多模输出混合（2006LE field_0x48-54）
- [ ] 录音 A/B 验证（低频保留度）

### 阶段 2：SysEx（含 NRPN）
现状：Yamaha single/bulk→RAM ✓、Master Tuning ✓、XG System On ✓、GM/GS/Roland/UniversalRealtime ✓；
测试仅 reset/gm2 映射
- [x] **`bulk_write` 逻辑 bug（2026-08-10 已修）**：`||` → `&&`（原逻辑导致 bulk dump 永不生效）
- [x] **SysEx 格式深层修复（2026-08-10，测试驱动发现）**：
  1. **`XGWriteMode` 映射反**：sub-status 0x00=单址、0x10=Bulk（原 0x00→Bulk/0x10→Single）
  2. **sub-status 字节缺失**：XG 标准 `F0 43 1n 4C ss aH aM aL`——原解析把 ss 当 addrH（错位 1 字节）
  3. **`RAM::set` GM 模式拒绝一切**：XG System On（00 00 7E）也被拒（鸡生蛋）——任意模式放行 System On
  - 回归：xg_bulk_write_applies_parameters / xg_sysex_writes_affect_sound / nrpn_writes_xg_ram
- [ ] XG 参数 SysEx 写入 → 声音变化端到端测试（volume 已测；其余参数后续）
- [ ] NRPN 全参数映射核对（nrpn_to_addr 覆盖度）——已有端到端，覆盖度核对后续
- [ ] UniversalRealtime 细节（GM2/GS/XG reset 完整、Master Volume/Transpose）

#### PLUGIN 区（0x70/0x71）实现（语义定稿 2026-08-10）
- [x] **内存模型（用户已实现）**：`PluginPartAssign`（70 nn mm→part，hook_pre_exec 发 `SetPluginForPart`）、
      `PluginNoteFilter`（71 nn pp→data）、RAM 0x70/0x71 接线、`PluginType` 枚举
      （OFF/VL/SG/DX/PF/AN/DR/PC/XG/OPL3/RP2A03/WinGroove）、`PartEngine`→`PluginType`（part.engine）
- [ ] **语义定稿**（已与用户核对）：
  - `70 nn mm <data>`：nn=板类型（0=PLG100-VL,1=SG,2=DX…），**mm=Board identifier**（同型号多板序号，
    用户引用的 spec 原文），**data=Part Number**（00-0F=Part1-16，7F=OFF）
  - `71 nn pp <data>`：nn=板类型，**pp=Part Number**（前置条件判断：板已分配给 pp），
    **data=00-0F=被忽略 NoteOn 的 part**（pp≠data 时过滤另一 part，多 part 共享通道防双音）
  - **Note Filter 不独立执行过滤**：真正生效的是 MultiPart 的 RcvNote（08 pp 02）——
    spec "must be turned off"；PLUGIN 区只是配置记录（可读回）
- [ ] **副作用链未接**：`SetPluginForPart` 无消费者——`part.engine` 不会被更新、发声侧不读；
      需接：事件 → part.engine 赋值 → TG 发声时按 engine 路由（VL/DX/OPL3/WinGroove 低优先，
      SG/PF/DR/PC 静音）
- [ ] **写入检查 + 警告**（用户设计）：
  - 写 `71 nn pp <data>` 时：① 板 nn 未分配给 pp → warn"Note Filter 指定的板未分配，参数无效"；
    ② **data part** 的 RcvNote（08 pp 02）未关闭 → warn"part X 的 RcvNote 未关闭，
    Note Filter 不生效（spec: must be turned off）"
  - **只警告一次**：记录上次警告 key (nn,pp,data)，状态变化才重新警告（防编辑器重复写入刷屏）
  - 参数照常保存（记录性质，发声行为由 RcvNote 决定）
- [ ] 测试：写 71 + RcvNote 开 → 断言警告出现 + 参数记录；RcvNote 关后写 → 无警告；
      重复写相同配置 → 不重复警告；Part Assign 副作用（SetPluginForPart → part.engine）
- [ ] **待核查：`Memory::hook_check` 默认实现全局改动**（"值变化才执行"，原为恒 true）——
      影响所有用默认 hook_check 的 Memory 实现，需回归确认无副作用；异常则询问是否回退

### 阶段 3：打击乐/SFX
现状：SFX 鼓（drum_key_type=0）走 sfx_key → PCM 已绑 ✓；普通鼓（type≠0）走
`SampleMeta::from(ds)` → **pcm: None（PCM 从未加载）**；drum kit 表 ✓；drum_params→TG ✓
- [ ] **普通鼓 PCM 加载**（按 start_point_offset/loop_length 从 sxgwave4 取，参考 melodic set_wave 路径）
- [ ] 鼓音色端到端：发声、drum_params（level/pan/sends/filter/EG）生效、alter group 截断

### 阶段 4：效果器 + multi_part_ext
现状：效果器 DSP 15 文件（45 参数层测试）；system（reverb/chorus/variation）+ insertion
接线 ✓；multi_part_ext（0A pp）HPF/CBC 已接线
- [ ] 效果器音质端到端（wet/dry、参数变化响应、send 电平）
- [ ] variation/insertion 参数语义与 2006LE 对齐（Ghidra 核对参数表）
- [ ] **multi_part_ext 全参数消费核对**（0A pp 各偏移：HPF ✓、CBC ✓、其余是否被读）
- [ ] 效果器输出断言（回归）+ 录音 A/B（验收）

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



