pub mod decrypt;
pub mod drum_setup;
pub mod sample_meta;
pub mod check_header;
pub mod pre_voice;
pub mod bintbl;
//pub mod xg_lite;

pub use decrypt::decrypt;
pub use check_header::check_header;
pub use bintbl::BinTbl;

pub const DRUM_BANK_OFFSET_GS: usize = 0x00;
pub const DRUM_BANK_OFFSET_XG: usize = 0x0B; // MSB=0x7F
pub const DRUM_BANK_OFFSET_GM2: usize = 0x16; // MSB=0x78