use super::super::engine::Engine;
use super::interface;

// ── Yamaha SysEx (0x43) ───────────────────────────────────────────
// F0 43 [device-id] [sub-status] [addr-h] [addr-m] [addr-l] [data...] F7
#[derive(Clone, Debug)]
pub struct YamahaSysEx {}

impl interface::Event for YamahaSysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) {}
}
