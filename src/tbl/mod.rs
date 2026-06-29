// TBL (Table) File Parser
//
// This module handles parsing Yamaha SYXG sound bank files (.tbl).
// TBL files contain the instrument wavetable data used by the SYXG
// software synthesizer series (SYXG-50, SYXG-2006 LE, etc.).
//
// File structure overview:
//   - Header: magic bytes, version, offsets
//   - Instrument data: waveform definitions, envelope parameters
//   - Sample data: PCM waveform data

pub mod errors;
pub mod interface;
pub mod transform_byte;

pub mod yxg2006le;
pub mod yxg50;

pub use transform_byte::transform_byte;

use crate::config::TBLType;

pub fn check_header(header: &[u8]) -> TBLType {
    // TODO check header magic here.
}
