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

pub mod yxg50;
pub mod yxg2006le;

/// Used for encrypted wave tbl file
pub fn transform_byte(start_key: u8, data: &mut Box<[u8]>) {
    let mut key = start_key;

    for (i, b) in data.iter_mut().enumerate() {
        let tmp = *b ^ key ^ (i as u8);
        *b = (tmp >> 4) | (tmp << 4);
        key = !key;
    }
}
