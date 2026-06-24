# Madaha — 雅马哈 S-YXG50 软件合成器 Rust 复刻

**Madaha** 是用 Rust 编写的 Yamaha S-YXG50 软件合成器的复刻项目。
目标是忠实地还原原始 S-YXG50 / S-YXG2006 LE 软波表的音色和 MIDI 行为，
通过解析其专有的 `.tbl` 音色库文件并驱动相同的音频流水线。

## 项目状态

**早期开发阶段。** MIDI 引擎已处理控制器、RPN/NRPN 和 SysEx（GM/XG/GS/Roland/Yamaha）。
效果子系统已有数据/参数表，但尚未实现 DSP。
**音频渲染尚未实现**（`src/engine/voice/` 为空）。许多事件处理器为 `todo!()`。

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

[tbl]
tbl_type = "auto"              # auto / s-yxg50 / syxg2006le
tbl_bin_file = "sxgbin21.tbl"  # 乐器定义文件
tbl_data_file = "sxg2006le.tbl" # 波形数据文件

[audio]
engine = "alsa"                # alsa / pipewire / pulseaudio / jack
max_polyphony = 512            # 16 的倍数，最大 2048
sample_rate = 44100            # 22050 / 44100 / 48000 / 96000 / 192000
depth = "s16"                  # u8 / s16 / s24 / f32
master_tune = 440.0
```

## 架构说明

```
src/
├── main.rs      — 入口：参数解析 → 配置加载 → 合成器 → 运行
├── args.rs      — clap 命令行参数（-D, -C / MADAHA_CONFIG_FILE）
├── config/       — TOML 反序列化与验证
├── engine/       — MIDI 引擎（16 通道状态、控制器、SysEx、RAM、效果器）
│   ├── voice/    —（空）音频渲染尚未实现
│   ├── sysex/    — GM/XG/GS/Roland/Yamaha SysEx 解析器
│   ├── ram/      — XG 参数 RAM，含 GS 地址重映射
│   └── effects/  — 仅有参数表，无 DSP
├── tbl/          — .tbl 文件解析器（s-yxg50 和 syxg2006le 两种格式）
└── utils.rs      — 位操作宏及 transform_byte 解密函数
```

### 注意事项

- **第 10 通道**（索引 9）为打击乐通道。
- **`max_polyphony`** 必须是 16 的倍数，否则配置加载会 panic。
- **TBL 波形数据**：使用 `transform_byte()`（XOR + 半字节交换）解密。
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
