# Madaha — Yamaha S-YXG50 soft synth clone in Rust

## Pitch chain (TODO: FIX ORDER)

The correct XG pitch chain is:

```
MIDI key
  → + (sys.transpose - 64)     // Master Transpose, System 0x06, ±24 semitones
  → + (part.note_shift - 64)   // Note Shift, Part 0x08, ±24 semitones
  → effective_key is used for:
     1. WaveEntry chain scan (layer selection)
     2. (effective_key - baseKey) * 100 + tone
  → + part.coarse_tune * 100   // Coarse Tune, RPN 2, ±64, NOT in SysEx map
  → + part.fine_tune            // RPN 1 or SysEx 09H-0AH
  → + part.detune * 10          // SysEx 09H-0AH (the 2-byte part)
  → + scale_tuning
  → + bend + portamento + PEG + LFO PM + ...
  → total_cents
```

Current bugs:
1. `get_delta_pitch` adds `note_shift` in cents domain — but note_shift should
   affect the MIDI key BEFORE WaveEntry selection. Currently layer selection
   uses the raw key, missing note_shift + transpose shifts.
2. `get_delta_pitch` doesn't include `sys.transpose` or `part.coarse_tune`.
3. `Pitch::step` only has access to `MultiPart` + `Channel`, not `System`.
   Need to pass `&RAM` or `&System` so system transpose is accessible.

Fix: move transpose + note_shift into the note dispatch layer (before
oscillator init), NOT in the cents summing chain.

## Prompt: Spec coverage check

When the user says "spec coverage check", do the following:

1. Read all source files under `src/` (madaha) and cross-reference against
   `dev_docs/XGSpec2.0.md` (XG 2.0 spec translation) and
   `dev_docs/note_opencode.md` (S-YXG50 reverse engineering).

2. Produce a markdown report covering:
   - **Missing features**: SysEx params, CC/NRPN handlers, effect types,
     voice features that exist in spec but not implemented in madaha.
   - **Incorrect behavior**: pitch chain order, unit mismatches, missing
     clamp/center-offset conversions, wrong XG address mapping.
   - **Incomplete stubs**: fields parsed but not wired into rendering
     (e.g. AEG/FEG rates from data file not connected to envelope state
     machines).
   - **Data model gaps**: RAM fields with no corresponding tone-gen read.

3. Be thorough — check every match arm in `Index`/`IndexMut` impls,
   every field in `MultiPart`/`System`/`DrumSetup`, every tone-generator
   `step()` fn, every voice-manager dispatch.

4. Focus on functional correctness, not style or performance.

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
- **`src/config/`** — TOML deserialization, validation. MidiConfig with
  ScoringConfig for polyphony stealing (time_weight, protect_attack,
  penalty_release, per-note/per-volume weights).
- **`src/engine/`** — MIDI engine: 16-channel state, controller/RPN/NRPN
  handling, SysEx (GM/XG/GS/Roland/Yamaha), RAM (XG/GS model with address
  remap). LFO subsystem with sine table and waveform generators. XG tone
  generation model (oscillator, LPF, AEG, PEG, FEG). Effects subsystem has
  data/parameter tables but no DSP.
- **`src/engine/ram/`** — Fully structured XG RAM (multi_part, effects,
  drum_setup, system, display_bitmap, effect2) and GS RAM via address remap.
  Hook system for parameter change callbacks.
- **`src/engine/lfo/`** — LFO implementation with 4096-entry sine table,
  saw/square/triangle/random waveform generators.
- **`src/engine/tone_generator/`** — XG tone generation signal chain
  (Oscillator → LPF → HPF → Amp → EQ → Pan) with PEG, FEG, AEG envelopes.
- **`src/voice_manager/`** — Sound bank loading, instrument caching with
  128×128×128 bank/program/key lookup. Voice stealing algorithm with scoring.
- **`libmadaha/`** — (separate crate) TBL file format parsers for S-YXG50
  and S-YXG2006 LE, designed to support Wingroove in the future.
  SoundModule trait with LoadedModule enum for format dispatch.
- **`src/utils.rs`** — bit manipulation macros + `transform_byte` decrypt fn.

## MIDI input

ALSA sequencer only (regardless of audio engine setting). Creates a port
named "Madaha MIDI input port" with GM/GS/XG capabilities.

## Notable quirks

- `max_polyphony` must be a multiple of 16, else config loading panics.
- Channel 10 (index 9) is the drum channel (`DRUM_CHANNEL_ID`).
- Polyphony stealing uses configurable scoring: protect_* params (< 1000)
  protect voices from being stolen; penalty_* params (> 1000) accelerate
  stealing. Default weights favor bass drum/snare/tom and loud voices.

## Extracted reference data

- `dev_docs/note_opencode.md` — full SYXG-50 TBL binary layout & Note On→audio
  pipeline from Ghidra RE.
- `dev_docs/XG_DATASET.md` — XG instrument/effect/SysEx parameter tables from
  qxgedit source.
