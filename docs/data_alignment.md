# S-YXG50 数据 vs 2006LE 程序对齐

逆向目标：S-YXG2006LE 程序（Mac i386 全功能版）。
数据来源：S-YXG50 数据文件（sxgbin41.tbl，78 字节 Element + WaveEntry + DrumData）。
2006LE 的 tbl 文件 (xgbnw6l.tbl) 尚未完全解析，先用 S-YXG50 数据喂 2006LE 算法。

## 对齐总原则

- 能对上的字段 → 直接对应
- 对不上的（转换表未解析）→ 填充中性值（0/64），使该阶段不产生影响
- 不在本次范围（AEG/FEG/DSP/效果）→ TODO 占位，等 2006LE 数据文件支持

## 已对齐（`src/voice_manager/sample_meta.rs` → `oscillator`）

| SampleMeta | 来源 (S-YXG50) | 2006LE 消费 | 实现 |
|---|---|---|---|
| `pcm/loop_point/loop_length` | WaveEntry 波形 | 波形数据 | `Oscillator` DDS ✓ |
| `base_note` + `tone` | WaveEntry[1]/[2] | 采样基准音高 | `get_base_note_cent()` + `get_tone()` ✓ |
| `pitch_coarse` | Element[35] | element 粗调 | `get_coarse_in_cent()` ✓ |
| `pitch_fine_h/l` | Element[9..10] | element 精调 | `get_fine_in_cent(vel)` 查 PITCH_FINE_TABLE ✓ |
| `pitch_offset` | Element[7] | 音高偏移 | `get_pitch_offset()` ✓ |
| `lfo_wave` (0-12) | Element[5] | LFO 波形 | 与 2006LE `WaveType` 0-12 一致 ✓ |
| `velocity` | MIDI | 力度 | `Oscillator.velocity` ✓ |

Pitch 公式（Oscillator::tick）:

```
cents = delay + peg + portamento + pitch(note×100)
      + coarse_in_cent + fine_in_cent(vel) + pitch_offset
ratio_cents = cents - base_note×100 + tone        // (key-base)×100 + tone + 偏移
```

## 中性占位（转换表未解析，填充不影响的值）

| SampleMeta | 2006LE 对应 | 处理 |
|---|---|---|
| `peg_rate0-4` | PEG 各段速率 (EGCnt/sample) | `PEG::setup` 填 rate=0, level=0 → 输出恒 0 |
| `peg_vel_sense_level/rate` | PEG 力度缩放 | 同上（未接入） |
| `peg_rate_scaling/center_note` | PEG 键盘缩放 | 同上（未接入） |
| `pitch_eg_attack/decay` | PEG Attack/Decay | 同上（未接入） |

## TODO（AEG/FEG/DSP，未实现，字段已就位）

| SampleMeta | 2006LE 对应 |
|---|---|
| `filter_cutoff/resonance`, `eg_filt_en`, `cs_en_1/2`, `ovr_cutoff` | FEG/LPF |
| `eg_amp_en`, `aeg_d1/d2/rel`, `rate_idx`, `eg_delay`, `trig_mode` | AEG |
| `dsp_base`, `tbl_index`, `tbl_68`, `alt_ovr` | DSP 参数表 |
| `ls_*`, `output_en` | 音量缩放/输出 |
| `fmt_flag`, `wave_pitch`, `off_hi/lo` | 采样格式/偏移（PCM 已预转 f32）|
| `note_shift`, `detune` | Part 级 (08 pp 08/09) —— 应由 2006LE 程序从 MultiPart 读，S-YXG50 文件值仅作参考 |

## Part 级参数（2006LE 程序读 MultiPart 08 pp，与数据文件无关）

- LFO: Vibrato Rate/Depth/Delay (08 pp 15-17)
- 调制深度: MW/CAT/PAT/AC pitch control (08 pp 1D/4D/53/5A/61)
- AEG 基准: EG Attack/Decay/Release (08 pp 1A-1C)
- Portamento: 08 pp 67/68
- 这些已由 MIDI 侧解析到 `MultiPart`（Arc 绑定），待 voice 实时调制接入

## 接入点

- `ToneGenerator::play(note, vel, part)` → `part.program_entry[note].samples[vel][0]` → `Oscillator::setup(sample, note, vel)`
- `Oscillator::setup` → `set_sample` + `peg.setup` + `lfo_wave`
