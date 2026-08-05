# Madaha — 雅马哈 S-YXG50 软件合成器 Rust 复刻

**Madaha** 是用 Rust 编写的 Yamaha S-YXG50 软件合成器的复刻项目。
目标是忠实地还原原始 S-YXG50 / S-YXG2006 LE 软波表的音色和 MIDI 行为，
通过解析其专有的 `.tbl` 音色库文件并驱动相同的音频流水线。

## 项目状态

**功能完整。** 完整链路可用并通过测试：

- **MIDI 输入**：ALSA Sequencer / JACK / PipeWire 端口（实时音符/控制器/SysEx）
- **发声引擎**：完整 XG 音色生成链（振荡器 → LPF → HPF → 音量 → EQ → 声像，
  PEG/FEG/AEG 包络、双力度层元素、鼓组设置）
- **效果器**：从 Mac 版二进制逆向的 2006LE 全部效果内核（混响/合唱/延迟/早期反射/
  失真/移相/旋转音箱）+ 完整 XG2.0 效果集（~100 种变化类型，含和声/声码器/
  说话调制器、串联链、Dyna 家族）——见 `dev_docs/effect_dsp.md`
- **MIDI 行为**：控制器、RPN/NRPN、SysEx（GM/GM2/XG/GS/Roland/Yamaha）、
  Active Sensing 看门狗、XG RAM 模拟（GS 地址重映射）、可配置评分的复音偷取
- **音频输出**：ALSA / PipeWire / PulseAudio / JACK 后端、4 种位深、
  DC 偏移修正、软限幅、主增益
- **测试**：123 个全部通过（单元 + 6 个端到端走真实渲染链）

## 环境要求

- **Rust nightly**（edition `2024`，ci-never 1.95+）。Stable 无法编译。
- **ALSA**（MIDI 输入必需）。音频后端：ALSA、PipeWire、PulseAudio、JACK。
- 需要原始的 **Yamaha `.tbl` 文件**（S-YXG50 或 S-YXG2006 LE，不随项目提供）。

推荐使用 **Nix flake** 提供所有系统依赖（`direnv allow` 或 `nix develop`）。

## 快速开始

```sh
cargo run -- -D                              # 调试模式，读取 madaha.toml
cargo run -- -C /path/to/madaha.toml         # 指定配置文件
MADAHA_CONFIG_FILE=/path cargo run -- -D     # 通过环境变量指定
```

## 配置说明

配置文件为 TOML 格式且 **被 gitignore**，仓库根目录有示例文件 `madaha.toml`。

```toml
log_level = "info"

[sound_module]
module_type = "syxg50"          # auto / syxg50 / syxg2006le / wingroove
tbl_bin_file = "sxgbin41.tbl"   # 乐器定义文件（可用绝对路径）
tbl_data_file = "sxgwave4.tbl"  # 波形数据文件

[audio]
engine = "alsa"                 # alsa / pipewire / pulseaudio / jack
sample_rate = 44100             # 22050 / 44100 / 48000 / 96000 / 192000
depth = "s16"                   # u8 / s16 / s24 / f32
buffer_size = 64                # 块大小（2 的幂）
master_volume = 1.0             # 输出增益（0.05..=4.0）
soft_clip = true                # 主总线 tanh 软限幅
dc_blocker = true               # DC 偏移修正（XG Spec）

[midi]
max_polyphony = 512             # 16 的倍数，最大 2048
poly_replicant = 150            # 复音数 = max_polyphony × replicant/100
device_id = 16                  # SysEx 设备号（>= 16）
master_tune = 440.0
input_engine = "alsa"           # alsa / jack / pipewire
```

## 架构说明

```
src/
├── main.rs          — 入口：参数解析 → 配置加载 → 合成器 → 运行
├── args.rs          — clap 命令行参数（-D, -C / MADAHA_CONFIG_FILE）
├── config/          — TOML 反序列化与验证（含评分配置）
├── audio/           — 渲染：audio_render（主总线/效果链/DC 修正）、
│   │                 tone_generator（XG 发声链）、dsp（效果器）、backend（输出）
├── midi/            — 引擎、声部、ram（XG/GS）、sysex（GM/XG/GS/Roland）、
│   │                 source（alsa/jack/pipewire）、active_sensing
├── voice_manager/   — TBL 音色库加载、乐器缓存、鼓组设置
├── lfo/             — LFO（DDS，13 种波形）
└── libmadaha/       —（独立 crate）TBL 文件格式解析器
    └── yxg50/       — .tbl 文件解析器（bintbl、pre-voice、sample_meta、drum）
```

### 注意事项

- **第 10 通道**（索引 9）为打击乐通道。
- **`max_polyphony`** 必须是 16 的倍数，否则配置加载会 panic。
- **复音偷取评分**：`protect_*` 参数（< 1000）保护音符不被偷取，
  `penalty_*` 参数（> 1000）加速偷取。默认权重偏向低音鼓/军鼓/通鼓和大音量音符。
- **TBL 波形数据**使用 `transform_byte()`（XOR + 半字节交换）解密。
- **MIDI 输入**固定使用 ALSA 时序器，与配置的音频后端无关。

## MIDI 使用

程序会创建一个名为 "Madaha MIDI input port" 的 ALSA 时序器端口，
声明 GM/GS/XG 能力。使用 `aconnect` 连接 MIDI 键盘或 DAW。

## 参考文档

- `dev_docs/note_opencode.md` — S-YXG50 `.tbl` 完整二进制布局和 Note On 到
  音频的流水线分析（Ghidra 逆向工程）。
- `dev_docs/XG_DATASET.md` — 来自 qxgedit 的 XG 乐器、效果器和 SysEx 参数表。

## 许可证

GNU General Public License v3.0 或更高版本。详见 [LICENSE](LICENSE)。
