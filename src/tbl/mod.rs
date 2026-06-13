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

pub mod yxg2006le;
pub mod yxg50;
