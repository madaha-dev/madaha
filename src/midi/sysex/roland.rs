use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::midi::Engine;
use crate::midi::ram::interface::Memory;
use crate::midi::ram::{MemoryAddr, RAMCallbackEffects};

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
    fn parse(e: &mut Engine, data: Box<[u8]>) -> Vec<RAMCallbackEffects> {
        let dev_id = get_dev_id!(data);
        let mut effects = vec![];
        if (dev_id == e.dev_id || dev_id == SYSEX_CHANNEL_ALL_DEVICE)
            && let Some(checksum) = data.last()
            && *checksum == calc_checksum(&data)
            && let Some(model_id) = data.get(1)
            && *model_id == GS_MODEL_ID
            && let Some(command) = data.get(2)
            && let Ok(cmd) = RolandCommand::try_from(*command)
            && cmd == RolandCommand::DT1
        {
            effects.extend(Self::single_write(e, &data));
        }

        effects
    }
}

impl RolandSysEx {
    fn single_write(e: &mut Engine, data: &Box<[u8]>) -> Vec<RAMCallbackEffects> {
        let mut effects = vec![];
        if let Some(r_addr) = data.get(3..=5).map(|d| MemoryAddr::try_from(d))
            && let Ok(mut addr) = r_addr
            && let Some(values) = data.get(6..data.len() - 1)
        {
            values.iter().for_each(|v| {
                if let Ok(v) = e.ram.set(addr, *v) {
                    effects.extend(v);
                }
            });
            addr.inc();
        }

        effects
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
