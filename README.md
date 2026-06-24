# Madaha

**Madaha** is a clone of the Yamaha S-YXG50 software synthesizer, written in
Rust. It aims to faithfully reproduce the sound and MIDI behavior of the
original S-YXG50 / S-YXG2006 LE soft synths by parsing their proprietary `.tbl`
instrument banks and driving the same audio pipeline.

## Status

**Early development.** The MIDI engine handles controllers, RPN/NRPN, and SysEx
(GM/XG/GS/Roland/Yamaha). The effects subsystem has data/parameter tables but
no DSP. **Audio rendering is not yet implemented** (`src/engine/voice/` is
empty). Many event handlers are stubbed with `todo!()`.

## Requirements

- **Rust nightly** (edition `2024` — ci-never 1.95+). Stable will not work.
- **ALSA** (mandatory for MIDI input). Audio backends: ALSA, PipeWire,
  PulseAudio, JACK.
- The **original Yamaha `.tbl` files** from S-YXG50 or S-YXG2006 LE (not
  included).

A **Nix flake** provides all system dependencies (`direnv allow` or
`nix develop`).

## Quick start

```sh
cargo run -- -D                              # debug mode, reads madaha.toml
cargo run -- -C /path/to/madaha.toml         # custom config
MADAHA_CONFIG_FILE=/path cargo run -- -D     # via env var
```

## Configuration

The config file is TOML and **gitignored** — an example lives at the repo root
as `madaha.toml`.

```toml
log_level = "info"

[tbl]
tbl_type = "auto"              # auto / s-yxg50 / syxg2006le
tbl_bin_file = "sxgbin21.tbl"  # instrument definition file
tbl_data_file = "sxg2006le.tbl" # wave data file

[audio]
engine = "alsa"                # alsa / pipewire / pulseaudio / jack
max_polyphony = 512            # multiple of 16, max 2048
sample_rate = 44100            # 22050 / 44100 / 48000 / 96000 / 192000
depth = "s16"                  # u8 / s16 / s24 / f32
master_tune = 440.0
```

## Architecture

```
src/
├── main.rs      — entrypoint: args → config → synth → run
├── args.rs      — clap CLI (-D, -C / MADAHA_CONFIG_FILE)
├── config/       — TOML deserialization & validation
├── engine/       — MIDI engine (16-ch state, controllers, SysEx, RAM, effects)
│   ├── voice/    — (empty) audio rendering not yet implemented
│   ├── sysex/    — GM, XG, GS, Roland, Yamaha SysEx parsers
│   ├── ram/      — XG parameter RAM with GS address remap
│   └── effects/  — parameter tables only, no DSP
├── tbl/          — .tbl file parsers (s-yxg50 & syxg2006le variants)
└── utils.rs      — bit manipulation macros & transform_byte decrypt
```

### Notable quirks

- **Channel 10** (index 9) is the drum channel.
- **`max_polyphony`** must be a multiple of 16, or config loading panics.
- **TBL wave data** is decrypted with `transform_byte()` (XOR + nibble swap).
- **MIDI input** uses ALSA sequencer regardless of the configured audio engine.

## MIDI

Creates an ALSA sequencer port named "Madaha MIDI input port" advertising
GM/GS/XG capabilities. Connect your MIDI keyboard or DAW via `aconnect`.

## Reference docs

- `dev_docs/note_opencode.md` — full S-YXG50 `.tbl` binary layout and Note On
  → audio pipeline (Ghidra RE).
- `dev_docs/XG_DATASET.md` — XG instrument, effect, and SysEx parameter tables
  from qxgedit.

## License

GNU General Public License v3.0 or later. See [LICENSE](LICENSE).
