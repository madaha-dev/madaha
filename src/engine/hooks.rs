use crate::engine::Engine;
use crate::engine::ram::MemoryAddr;
use crate::voice_manager::{DRUM_BANK_MSB_GM2, DRUM_BANK_MSB_GS, DRUM_BANK_MSB_XG};

use super::engine::MidiResetMode;
use super::ram::xg::drum_setup::DrumSetup;

/// Register hook rules for RAM address changes.
///
/// Rules are checked in order: first match wins.
/// `0xFF` in any position acts as a wildcard (matches any value).
///
/// # Example
/// ```
/// hook_rules!(self, addr, [
///     (0x08, 0xFF, 0x07) => Engine::backup_part_mode,
///     (0x08, 0xFF, 0x03) => |engine, h, m, l| { engine.on_program_changed(m as usize); },
/// ]);
/// ```
#[macro_export]
macro_rules! hook_rules {
    ($engine:expr, $addr:expr, [ $( ($h:expr, $m:expr, $l:expr) => $handler:expr ),* $(,)? ]) => {{
        let (h, m, l) = $addr.split();
        let engine = &mut *$engine;
        $(
            if ($h == 0xFF || h == $h) && ($m == 0xFF || m == $m) && ($l == 0xFF || l == $l) {
                $handler(engine, h, m, l);
                return;
            }
        )*
    }}
}

impl Engine {
    pub fn pre_hooks(&mut self, addr: MemoryAddr) {
        hook_rules!(self, addr, [
            (0x08, 0xFF, 0x07) => |e: &mut Engine, _h: u8, m: u8, _l: u8| {
                let ch = m as usize;
                e.channels[ch].prev_bank_msb = e.ram.xg.multi_part[ch].bank_select_msb;
                e.channels[ch].prev_bank_lsb = e.ram.xg.multi_part[ch].bank_select_lsb;
                e.channels[ch].prev_program = e.ram.xg.multi_part[ch].program_number;
            },
            // TODO: add more pre-hook rules here
            (0x08, 0xFF, 0x02) => |_e: &mut Engine, _h: u8, _m: u8, _l: u8| {

            },
        ]);
    }

    pub fn post_hooks(&mut self, addr: MemoryAddr) {
        hook_rules!(self, addr, [
            // TODO: add more post-hook rules here
            (0x08, 0xFF, 0x07) => |e: &mut Engine, _h: u8, _m: u8, _l: u8| {
                let ch = _m as usize;
                if e.ram.xg.multi_part[ch].part_mode != 0 {
                    e.ram.xg.multi_part[ch].bank_select_lsb = 0;
                    e.ram.xg.multi_part[ch].bank_select_msb = match e.ram.reset_mode {
                        MidiResetMode::GM | MidiResetMode::GS => DRUM_BANK_MSB_GS as u8,
                        MidiResetMode::XG => DRUM_BANK_MSB_XG as u8,
                        MidiResetMode::GM2 => DRUM_BANK_MSB_GM2 as u8,
                    };
                }
            },
            // Program change
            (0x08, 0xFF, 0x03) => |e: &mut Engine, _h: u8, _m: u8, _l: u8| {
                let ch = _m as usize;
                let bank_msb = e.ram.xg.multi_part[ch].bank_select_msb;
                let bank_lsb = e.ram.xg.multi_part[ch].bank_select_lsb;
                let program = e.ram.xg.multi_part[ch].program_number;
                if e.ram.xg.multi_part[ch].part_mode != 0 {
                    if let Some(drum_setup) = e.voice_manager.get_drum_setup(bank_msb, program) {
                        e.ram.xg.drum_setup = [drum_setup.map(|ds| DrumSetup::from(ds)); 16];
                    }

                    e.channels[ch].program_entry = e.voice_manager.get_program(bank_msb, bank_lsb, program)
                }
            },
            (0x08, 0xFF, 0x01) => |e: &mut Engine, _h: u8, _m: u8, _l: u8| {
                let ch = _m as usize;
                if matches!(
                    e.ram.xg.multi_part[ch].bank_select_msb as usize,
                    DRUM_BANK_MSB_GM2 | DRUM_BANK_MSB_GS | DRUM_BANK_MSB_XG
                ) {
                    // start drum mode
                    let part_mode_memory = MemoryAddr::new(0x08, ch as u8, 0x07);
                    let _ = e.mem_set(part_mode_memory, 0x02);
                }
            }
        ]);
    }
}
