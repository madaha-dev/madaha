# Madaha — Yamaha S-YXG50 soft synth clone in Rust

## Quick start

```sh
cargo run -- -D                         # debug mode, reads madaha.toml
cargo run -- -C /path/to/madaha.toml    # custom config
```

## Rust edition

`edition = "2024"` — requires **nightly** Rust (ci-never 1.95+).
Check `Cargo.toml` before assuming stable works.

## Build & dev

- **Nix flake** is the primary dev environment: `direnv allow` or `nix develop`.
  The flake provides ALSA, PipeWire, libclang, and JACK2 libs.
- `.envrc` sets `CARGO_HOME=.cargo` (gitignored) and calls `use flake`.
- `.cargo/config.toml` overrides crates.io with mirror (aliyun). Cargo.lock is
  checked in.
- No tests exist yet. No CI. No lint config beyond treefmt.

## Formatting

`treefmt` (runs nixfmt + rustfmt + mdformat + yamlfmt + shellcheck).
Excludes: `*.rasi`, `*.jpg`, `*.png`, `Makefile`, `.vscode/*`, `.zed/*`,
`.opencode/*`, `.cargo`, `.envrc`.

## Config (madaha.toml)

**Gitignored.** Example at repo root. Fields:
- `log_level`: debug/info/warn/error
- `tbl`: type (auto/s-yxg50/syxg2006le), bin_file, data_file
- `audio`: engine (alsa/pipewire/pulseaudio/jack), max_polyphony (multiple of
  16, max 2048), sample_rate (22050/44100/48000/96000/192000), depth
  (u8/s16/s24/f32), master_tune

Config file path via `-C` flag or `MADAHA_CONFIG_FILE` env var (default:
`madaha.toml`).

## Sound engine architecture

- **`src/main.rs`** — entrypoint: parse args → load config → new Synth → run
- **`src/args.rs`** — clap CLI: `-D` (debug), `-C` (config path)
- **`src/config/`** — TOML deserialization, validation
- **`src/engine/`** — MIDI engine: 16-channel state, controller/RPN/NRPN
  handling, SysEx (GM/XG/GS/Roland/Yamaha), RAM (XG model with GS address
  remap). Effects subsystem has data/parameter tables but no DSP.
  **`engine/voice/` is empty** — audio rendering not implemented.
  Many event handlers are `todo!()` — only Controller and SysEx work.
- **`src/tbl/`** — Yamaha SYXG `.tbl` file parsers (s-yxg50 + syxg2006le
  variants). Decryption: `transform_byte()` XOR + nibble swap.
- **`src/utils.rs`** — bit manipulation macros + `transform_byte` decrypt fn.

## MIDI input

ALSA sequencer only (regardless of audio engine setting). Creates a port
named "Madaha MIDI input port" with GM/GS/XG capabilities.

## Notable quirks

- `max_polyphony` must be a multiple of 16, else config loading panics.
- Channel 10 (index 9) is the drum channel (`DRUM_CHANNEL_ID`).

## Extracted reference data

- `dev_docs/note_opencode.md` — full SYXG-50 TBL binary layout & Note On→audio
  pipeline from Ghidra RE.
- `dev_docs/XG_DATASET.md` — XG instrument/effect/SysEx parameter tables from
  qxgedit source.
