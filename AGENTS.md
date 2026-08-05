# Madaha — Yamaha S-YXG50 / S-YXG2006 LE soft synth clone in Rust

Madaha is a **superset of S-YXG2006 LE**: everything the 2006LE supports must
work, plus features it lacks (Flanger, XG2.0 full set, GS, multi-backend).

## Quick start

```sh
nix develop
cargo test -- --test-threads=1     # 123 tests, must stay green
cargo run -- -D                    # debug logs, reads madaha.toml
cargo run -- -C /path/to/madaha.toml
```

## Rust edition

`edition = "2024"` — requires **nightly** Rust (ci-never 1.95+).
`src/main.rs` has `#![deny(warnings)]` — every warning is a build error.
Keep real functionality behind `#[allow(dead_code)]` (never delete).

## Build & dev

- **Nix flake** is the primary dev environment: `direnv allow` or `nix develop`.
- `.envrc` sets `CARGO_HOME=.cargo` (gitignored).
- Tests are slow/long: run `cargo test -- --test-threads=1`; e2e tests need a
  big stack (`run_on_big_stack`, 256 MB) and are wrapped already.
- Long test runs: `setsid nohup … > /tmp/opencode/x.log &` then poll.

## Config (madaha.toml)

**Gitignored** — example at repo root. Sections match the config structs:

- `log_level`: debug/info/warn/error
- `[sound_module]`: `module_type` (auto/syxg50/syxg2006le), `tbl_bin_file`,
  `tbl_data_file` — point at real TBL files (absolute paths ok)
- `[audio]`: `engine` (alsa/pipewire/pulseaudio/jack), `sample_rate`,
  `depth` (u8/s16/s24/f32), `buffer_size`, `master_volume` (0.05..=4.0),
  `soft_clip`, `dc_blocker`, `alsa_buffer_frames`, `jack_client_name`
- `[midi]`: `max_polyphony` (multiple of 16), `poly_replicant`, `device_id`,
  `master_tune`, `input_engine`, `scoring` (voice stealing weights)

Config path via `-C` flag or `MADAHA_CONFIG_FILE` env var (default `madaha.toml`).

## Pitch chain (TODO: verify against 2006LE)

The XG pitch chain (from `dev_docs/note_opencode.md` RE):

```
MIDI key
  → + (sys.transpose - 64)
  → + (part.note_shift - 64)
  → effective_key: WaveEntry chain scan (layer selection)
  → + coarse_tune / fine_tune / detune / scale_tuning
  → + bend + portamento + PEG + LFO PM + ...
  → total_cents
```

Current implementation: `MultiPart::get_delta_pitch` = scale_tuning + detune;
`note_shift` is applied at the voice pitch step (tone_generator.rs).
Open question: note_shift/transpose should affect key-based layer selection —
verify against the 2006LE program (see `dev_docs/note_opencode.md`).

## Prompt: Spec coverage check

When the user says "spec coverage check", do the following:

1. Read the source under `src/` and cross-reference against
   `dev_docs/XGSpec2.0.md` (XG 2.0 spec translation) and
   `dev_docs/effect_dsp.md` (2006LE reverse engineering).

2. Produce a markdown report covering:
   - **Missing features**: SysEx params, CC/NRPN handlers, effect types,
     voice features that exist in spec but not implemented in madaha.
   - **Incorrect behavior**: pitch chain order, unit mismatches, missing
     clamp/center-offset conversions, wrong XG address mapping.
   - **Incomplete wiring**: fields parsed but not connected to rendering.
   - **Data model gaps**: RAM fields with no corresponding tone-gen read.

3. Be thorough — check every match arm in `Index`/`IndexMut` impls,
   every field in `MultiPart`/`System`/`DrumSetup`, every tone-generator
   step fn, every voice-manager dispatch.

4. Focus on functional correctness, not style or performance.

## Source layout

```
src/
├── main.rs            — entrypoint: args → config → check → Synth::run
├── args.rs            — clap CLI
├── synth.rs           — MIDI event loop + audio render thread
├── config/            — TOML deserialization & validation
├── audio/
│   ├── audio_render.rs — master bus, effect chain, DC correction, modulation
│   ├── wrapper.rs      — AudioRender (voices, system effects, insertion cache)
│   ├── backend/        — sinks: alsa/pulse/jack/pipewire/ringbuf
│   ├── tone_generator/ — XG voice chain + oscillator/lpf/hpf/amp/eq/pan
│   └── dsp/            — effects (reverb/chorus/delay/ER/distortion/phaser/
│                         rotary/harmony/vocoder/talking/xg20_effects + core)
├── midi/
│   ├── engine.rs       — event dispatch, reset, tuning
│   ├── part/           — Part, controller, RPN/NRPN, backup
│   ├── ram/            — XG RAM (multi_part/effects/drum/system/display) + GS remap
│   ├── sysex/          — GM/GM2/XG/GS/Roland parsers
│   ├── source/         — MIDI input (alsa/jack/pipewire)
│   ├── effect_params/  — effect types, parameter tables, default data
│   └── active_sensing.rs — 0xFE watchdog
├── voice_manager/      — TBL sound bank, instrument cache, drum setup
├── lfo/                — LFO (DDS, 13 waveforms)
├── double_buffer.rs    — Mutex + Arc snapshot parameter sharing
├── audit_tests.rs      — config/encode/LFO/voice_manager/RAM coverage
└── e2e_tests.rs        — end-to-end (NoteOn → voice → audio buffer)
```

## Effects (2006LE reverse engineering + XG2.0 superset)

- All 2006LE DSP kernels were reverse-engineered from the Mac binary
  (`S-YXG2006LE.vst`) — see `dev_docs/effect_dsp.md` for the extracted
  algorithms, coefficient flow, and type dispatch.
- Reverb = 131072-sample ring + 7 feedback lines + 2×1st-order IIR input.
- Chorus/Chorus3 = 32768 ring + XOR-table LFO (23-bit phase).
- Delay family = 4-tap + cross feedback; ER = 3 lines + 7+7 taps.
- Phaser = 5-stage allpass; Rotary = 4 modulated lines + dual-speed LFO.
- WSOLA pitch shifter (`core/wsola.rs`) is shared by PitchChange and Harmony.
- XG2.0: serial chains (Dist/Comp/Wah → Delay/Rotary), Dyna family
  (envelope-driven), misc effects (`xg20_effects.rs`), Harmony family
  (active-notes chord detection + WSOLA voices), Talking (formant filter).
- Type defaults: `xg_reverb_data`/`xg_chorus_data`/`xg_variation_data`/
  `xg_20_data` verified against the 2006LE binary tables and XG Spec.

## MIDI input

Creates a sequencer port "Madaha" (ALSA) or JACK/PW port with GM/GS/XG
capabilities. Connect via `aconnect <source> Madaha:0`.

## Notable quirks

- Channel 10 (index 9) is the drum channel (`DRUM_CHANNEL_ID`).
- `max_polyphony` must be a multiple of 16, else config loading panics.
- Voice stealing uses configurable scoring (`scoring` in `[midi]`).
- Drum parts ignore bank LSB/portamento/soft pedal/mono-poly/scale tuning/
  poly AT/pitch EG (YAMAHA list); sustain/modulation respond.
- Flanger types are implemented even though 2006LE passes them through
  (superset behavior).
- DC offset correction on the master bus is always sensible (config off-able);
  XG Spec warns serial chains introduce DC.
- System-common MIDI messages (MTC/SongPosition/etc.) are ignored by design
  (pure MIDI-in → audio-out, no MIDI out).
- No built-in .mid player: connect an external MIDI source.

## Reference docs

- `dev_docs/effect_dsp.md` — 2006LE effect DSP RE (kernels, coefficient flow,
  type dispatch, parameter tables)
- `dev_docs/xg20_params.md` — XG2.0 type parameter layouts
- `dev_docs/wsola_harmony.md` — WSOLA + harmony/vocoder
- `dev_docs/gm2.md` — GM2 SysEx
- `dev_docs/note_opencode.md` — S-YXG50 `.tbl` binary layout (Ghidra RE)
- `dev_docs/XGSpec2.0.md` — XG 2.0 spec translation
- `docs/TODO.md` — deferred features and decisions
