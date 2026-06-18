use wd_log::log_debug_ln;

use crate::midi::ram::MemoryAddr;

use super::super::engine::Engine;
use super::SYSEX_CHANNEL_ALL_DEVICE;
use super::interface;

// Only XG supported, MU50 or MU80 not planned.
const XG_MODEL_ID: u8 = 0x4C;

// ── Yamaha SysEx (0x43) ───────────────────────────────────────────
// F0 43 [device-id] [sub-status] [addr-h] [addr-m] [addr-l] [data...] F7
#[derive(Clone, Debug)]
pub struct YamahaSysEx {}

impl interface::Event for YamahaSysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) {
        let dev_id = get_dev_id!(data);
        if dev_id != e.dev_id || dev_id != SYSEX_CHANNEL_ALL_DEVICE {
            return;
        }

        if let Some(model_id) = data.get(1)
            && *model_id != XG_MODEL_ID
        {
            return;
        }

        let mode = data[0] & 0x10;
        if mode == 0 {
            // TODO: BULK MODE
        } else if mode == 1 {
            // TODO: SINGLE MODE
        }
    }
}

impl YamahaSysEx {
    // single address write mode.
    fn single_write(e: &mut Engine, data: Box<[u8]>) {
        let addr: MemoryAddr = match data.get(2..5) {
            Some(r) => match r.try_into() {
                Ok(r) => r,
                Err(_) => return,
            },
            None => return,
        };
        let value = match data.get(5) {
            Some(r) => *r,
            None => return,
        };

        if let Err(err) = e.memories.set(addr, value) {
            log_debug_ln!("{:?}", err);
        }
    }
}
