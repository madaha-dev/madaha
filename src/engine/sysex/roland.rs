use num_enum::{IntoPrimitive, TryFromPrimitive};
use wd_log::log_debug_ln;

use crate::engine::Engine;
use crate::engine::ram::MemoryAddr;

use super::checksum::calc_checksum;
use super::consts::SYSEX_CHANNEL_ALL_DEVICE;
use super::interface;

const GS_MODEL_ID: u8 = 0x42;
const GS_SYSTEM_ON_ADDR: MemoryAddr = MemoryAddr::new(0x40, 0x00, 0x7F);

// ── Roland SysEx (0x41) ────────────────────────────────────────────
// F0 41 [device-id] [model-id] [msg-type] [addr 4B] [data...] [checksum] F7
#[derive(Clone, Debug)]
pub struct RolandSysEx {}

impl interface::Event for RolandSysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) {
        let dev_id = get_dev_id!(data);
        if dev_id != e.dev_id || dev_id != SYSEX_CHANNEL_ALL_DEVICE {
            return;
        }

        if let Some(checksum) = data.get(data.len() - 1) {
            if *checksum != calc_checksum(&data) {
                return;
            }
        }

        if let Some(model_id) = data.get(1)
            && *model_id != GS_MODEL_ID
        {
            return;
        }

        if let Some(command) = data.get(2) {
            match RolandCommand::try_from(*command) {
                Ok(c) => {
                    if c == RolandCommand::DT1 {
                        Self::single_write(e, &data);
                    }
                }
                Err(_) => return,
            }
        }
    }
}

impl RolandSysEx {
    fn single_write(e: &mut Engine, data: &Box<[u8]>) {
        let mut addr: MemoryAddr = match data.get(3..6) {
            None => return,
            Some(r) => match r.try_into() {
                Ok(r) => r,
                Err(_) => return,
            },
        };

        if addr == GS_SYSTEM_ON_ADDR {
            let v = match data.get(6) {
                None => return,
                Some(r) => *r,
            };

            Self::gs_system_reset(e, v);
            return;
        }

        let values = match data.get(6..data.len() - 1) {
            None => return,
            Some(v) => v,
        };

        for v in values.iter() {
            if let Err(err) = e.mem_set(addr, *v) {
                log_debug_ln!("{:?}", err);
            }

            addr.inc();
        }
    }

    fn gs_system_reset(e: &mut Engine, v: u8) {
        match v {
            0x00 => e.gs_reset(),
            0x7F => e.gm_reset(),
            _ => (),
        }
    }
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum RolandCommand {
    RQ1 = 0x11,
    DT1 = 0x12,
}
