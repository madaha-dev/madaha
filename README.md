# Madaha

**Madaha** is a clone of the Yamaha S-YXG50 / S-YXG2006 LE software synthesizer,
written in Rust. It parses the original Yamaha `.tbl` instrument banks and drives
a full XG audio pipeline, aiming to be a **superset of S-YXG2006 LE**:
everything the 2006LE supports must work, plus features it lacks.

## Status

**Functional.** The complete chain works and is covered by tests:

- **MIDI input**: ALSA Sequencer / JACK / PipeWire ports (real-time note/CC/SysEx)
- **Sound engine**: full XG tone generation (Oscillator → LPF → HPF → Amp → EQ → Pan,
  PEG/FEG/AEG envelopes, dual velocity-layer elements, drum setup)
- **Effects**: all 2006LE effect kernels reverse-engineered from the Mac binary
  (Reverb/Chorus/Delay/ER/Distortion/Phaser/RotarySP) + the full XG2.0 effect
  set (~100 Variation types incl. Harmony/Vocoder/Talking, serial chains, Dyna
  family) — see `dev_docs/effect_dsp.md`
- **MIDI behavior**: controllers, RPN/NRPN, SysEx (GM/GM2/XG/GS/Roland/Yamaha),
  Active Sensing watchdog, XG RAM emulation with GS address remap, voice stealing
  with configurable scoring
- **Audio output**: ALSA / PipeWire / PulseAudio / JACK backends, 4 depths,
  DC-offset correction, soft-clip, master gain
- **Tests**: 123 passing (unit + 6 end-to-end through the real render chain)

## Requirements

- **Rust nightly** (edition `2024` — ci-never 1.95+). Stable will not work.
- **ALSA** (needed for MIDI input). Audio backends: ALSA, PipeWire, PulseAudio, JACK.
- The **original Yamaha `.tbl` files** (not included). Madaha reads the S-YXG50
  family files (`sxgbin41.tbl` + `sxgwave4.tbl`); the 2006LE files
  (`sxgbnw6l.tbl` etc.) are not read yet (see `docs/TODO.md`).

A **Nix flake** provides all system dependencies (`direnv allow` or `nix develop`).

## Quick start

```sh
# copy the TBL files (or point madaha.toml at them with absolute paths)
cp /path/to/sxgbin41.tbl /path/to/sxgwave4.tbl .

nix develop
cargo run -- -D                          # debug logs, reads madaha.toml
cargo run -- -C /path/to/madaha.toml     # custom config
MADAHA_CONFIG_FILE=/path cargo run -- -D # via env var
```

Connect a MIDI source (keyboard/DAW/`aplaymidi`):

```sh
aconnect <source> Madaha:0     # list with: aconnect -l
```

## Configuration

The config file is TOML and **gitignored** — an example lives at the repo root
as `madaha.toml`. Field names follow the config structs:

```toml
log_level = "info"

[sound_module]
module_type = "syxg50"          # auto / syxg50 / syxg2006le / wingroove
tbl_bin_file = "sxgbin41.tbl"   # instrument definition file (absolute path ok)
tbl_data_file = "sxgwave4.tbl"  # wave data file

[audio]
engine = "alsa"                 # alsa / pipewire / pulseaudio / jack
sample_rate = 44100             # 22050 / 44100 / 48000 / 96000 / 192000
depth = "s16"                   # u8 / s16 / s24 / f32
buffer_size = 64                # block size, power of two
master_volume = 1.0             # output gain (0.05..=4.0)
soft_clip = true                # tanh soft clipping on the master bus
dc_blocker = true               # DC offset correction (XG Spec)
jack_client_name = "madaha"     # JACK client name

[midi]
max_polyphony = 512             # multiple of 16, max 2048
poly_replicant = 150            # voice count = max_polyphony × replicant/100
device_id = 16                  # SysEx device id (>= 16)
master_tune = 440.0
input_engine = "alsa"           # alsa / jack / pipewire
# scoring = { ... }             # voice-stealing weights (see src/config/midi.rs)
```

## Architecture

```
src/
├── main.rs            — entrypoint: args → config → check → Synth::run
├── args.rs            — clap CLI (-D, -C / MADAHA_CONFIG_FILE)
├── synth.rs           — MIDI event loop + audio render thread
├── config/            — TOML deserialization & validation (audio/midi/sound_module)
├── audio/             — rendering
│   ├── audio_render.rs — master bus, effect chain, DC correction, modulation
│   ├── wrapper.rs      — AudioRender (voices, effects, sinks)
│   ├── tone_generator/ — XG voice chain (oscillator/LPF/HPF/amp/EQ/pan + EGs)
│   └── dsp/            — effects: reverb/chorus/delay/ER/distortion/phaser/
│                         rotary/harmony/vocoder/talking + XG2.0 misc (xg20_effects)
│                         + core (biquad/delay/dc_blocker/wsola)
├── midi/              — engine, part, ram (XG/GS), sysex (GM/XG/GS/Roland),
│                         source (alsa/jack/pipewire), active_sensing
├── voice_manager/     — TBL sound bank loading, instrument cache, drum setup
├── lfo/               — LFO (DDS + 13 waveforms)
├── double_buffer.rs   — lock-free-ish parameter sharing (Mutex + Arc snapshot)
└── libmadaha/         — (separate crate) TBL file format parsers
```

## Tests

```sh
nix develop --command cargo test -- --test-threads=1
```

123 tests: DSP kernels, effect parameter tables (verified against the 2006LE
binary), GS address remap, MIDI/SysEx parsing, config validation, and 6
end-to-end tests driving the real render chain (NoteOn → voice → audio buffer).

## Reference docs

- `dev_docs/effect_dsp.md` — 2006LE effect DSP reverse engineering (kernels,
  coefficient flow, type dispatch, parameter tables)
- `dev_docs/xg20_params.md` — XG2.0 type parameter layouts (XG Spec)
- `dev_docs/wsola_harmony.md` — WSOLA pitch shifter + harmony/vocoder design
- `dev_docs/gm2.md` — GM2 SysEx implementation
- `dev_docs/note_opencode.md` — S-YXG50 `.tbl` binary layout (Ghidra RE)
- `dev_docs/XGSpec2.0.md` — XG 2.0 spec translation
- `docs/TODO.md` — deferred features and decisions

## License

GNU General Public License v3.0 or later. See [LICENSE](LICENSE).
