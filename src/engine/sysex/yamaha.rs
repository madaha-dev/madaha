use num_enum::{IntoPrimitive, TryFromPrimitive};
use wd_log::log_debug_ln;

use crate::engine::engine::MidiResetMode;
use crate::engine::ram::MemoryAddr;

use super::super::engine::Engine;
use super::SYSEX_CHANNEL_ALL_DEVICE;
use super::checksum::calc_checksum;
use super::interface;

// Only XG supported, MU50 or MU80 not planned.
const XG_MODEL_ID: u8 = 0x4C;
const XG_SYSTEM_ON_ADDR: MemoryAddr = MemoryAddr::new(0x00, 0x00, 0x7E);

// ── Yamaha SysEx (0x43) ───────────────────────────────────────────
// F0 43 [device-id] [sub-status] [addr-h] [addr-m] [addr-l] [data...] F7
#[derive(Clone, Debug)]
pub struct YamahaSysEx {}

impl interface::Event for YamahaSysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) {
        if let Some(model_id) = data.get(1)
            && *model_id != XG_MODEL_ID
        {
            return;
        }

        match XGWriteMode::try_from(data[0] & 0x10).unwrap() {
            XGWriteMode::Bulk => Self::bulk_write(e, data),
            XGWriteMode::Single => Self::single_write(e, data),
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

        if addr == XG_SYSTEM_ON_ADDR {
            e.xg_reset();
            return;
        }

        let dev_id = get_dev_id!(data);
        if dev_id != e.dev_id || dev_id != SYSEX_CHANNEL_ALL_DEVICE {
            return;
        }

        let value = match data.get(5) {
            Some(r) => *r,
            None => return,
        };

        if e.ram.reset_mode != MidiResetMode::XG {
            return;
        }

        if let Err(err) = e.mem_set(addr, value) {
            log_debug_ln!("{:?}", err);
        }
    }

    fn bulk_write(e: &mut Engine, data: Box<[u8]>) {
        let dev_id = get_dev_id!(data);
        if dev_id != e.dev_id || dev_id != SYSEX_CHANNEL_ALL_DEVICE {
            return;
        }

        if e.ram.reset_mode != MidiResetMode::XG {
            return;
        }

        let byte_length_msb = match data.get(2) {
            Some(r) => *r,
            None => return,
        };
        let byte_length_lsb = match data.get(3) {
            Some(r) => *r,
            None => return,
        };
        let byte_length: u16 = (byte_length_msb as u16) << 7 | (byte_length_lsb as u16);

        let mut addr: MemoryAddr = match data.get(4..7) {
            Some(r) => match r.try_into() {
                Ok(r) => r,
                Err(_) => return,
            },
            None => return,
        };

        if let Some(checksum) = data.get(data.len() - 1) {
            if *checksum != calc_checksum(&data) {
                return;
            }
        } else {
            return;
        }

        for i in 0..byte_length {
            let value = data.get(7 + i as usize).unwrap();
            if let Err(_) = e.mem_set(addr, *value) {
                return;
            };
            addr.inc();
        }
    }
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum XGWriteMode {
    Bulk,
    Single = 0x10,
}
