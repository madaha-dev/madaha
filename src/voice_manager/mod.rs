// Voice Module — Sound Bank Data Loader
//
// This module provides a unified interface for loading instrument wavetable data
// from different sound bank file formats:
//   - Yamaha SYXG (.tbl) — SYXG-50, SYXG-2006 LE, etc.
//   - Wingroove (.bin) — Wingroove sound bank files
//
// The module abstracts the underlying format differences and exposes a single
// interface: given (msb, lsb, program, note, velocity), it returns the
// corresponding waveform object for audio rendering.
//
// Internal structure:
//   - Header parsing and format detection
//   - Sample data loading and decryption
//   - Instrument/envelope parameter extraction

pub mod voice;
pub mod voice_manager;

mod bank;
mod drum_setup;
mod keys;
mod program;
mod sample_meta;
mod parser;

pub use drum_setup::DrumSetupEntry;

pub use voice_manager::DRUM_BANK_MSB_GS;
pub use voice_manager::DRUM_BANK_MSB_XG;
pub use voice_manager::DRUM_BANK_MSB_GM2;
pub use voice_manager::SFX_BANK_MSB_XG;
pub use voice_manager::VoiceManager;
pub use voice_manager::Instruments;