use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::midi::MIDICallbackEffects;
use crate::midi::engine::MidiResetMode;
use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;

use super::super::engine::Engine;
use super::SYSEX_CHANNEL_ALL_DEVICE;
use super::checksum::calc_checksum;
use super::interface;

// Only XG supported, MU50 or MU80 not planned.
const XG_MODEL_ID: u8 = 0x4C;
const XG_TUNING_ID: u8 = 0x27;
const XG_SYSTEM_ON_ADDR: MemoryAddr = MemoryAddr::new(0x00, 0x00, 0x7E);

// ── Yamaha SysEx (0x43) ───────────────────────────────────────────
// F0 43 [device-id] [sub-status] [addr-h] [addr-m] [addr-l] [data...] F7
#[derive(Clone, Debug)]
pub struct YamahaSysEx {}

impl interface::Event for YamahaSysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) -> Vec<MIDICallbackEffects> {
        if let Some(model_id) = data.get(1) {
            if *model_id == XG_MODEL_ID {
                match XGWriteMode::try_from(data[0] & 0x10).unwrap() {
                    XGWriteMode::Bulk => Self::bulk_write(e, data),
                    XGWriteMode::Single => Self::single_write(e, data),
                }
            } else if *model_id == XG_TUNING_ID
                && let Some(addr) = data.get(2..=4)
                && addr == [0x30, 0x00, 0x00]
                && let Some(tuning) = data.get(5..=6).map(|v| (v[0] as u16) << 7 | v[1] as u16)
            {
                vec![MIDICallbackEffects::ChangeMasterTuning { tuning }]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

impl YamahaSysEx {
    // single address write mode.
    fn single_write(e: &mut Engine, data: Box<[u8]>) -> Vec<MIDICallbackEffects> {
        let dev_id = get_dev_id!(data);

        if let Some(addr) = data.get(2..5).map(|d| MemoryAddr::from(d))
            && let Some(value) = data.get(5).map(|r| *r)
            && (addr == XG_SYSTEM_ON_ADDR || e.ram.reset_mode == MidiResetMode::XG)
            && (dev_id == e.dev_id || dev_id == SYSEX_CHANNEL_ALL_DEVICE)
        {
            e.ram.set(addr, value).unwrap_or(vec![])
        } else {
            vec![]
        }
    }

    fn bulk_write(e: &mut Engine, data: Box<[u8]>) -> Vec<MIDICallbackEffects> {
        let dev_id = get_dev_id!(data);
        if dev_id != e.dev_id || dev_id != SYSEX_CHANNEL_ALL_DEVICE {
            return vec![];
        }

        if e.ram.reset_mode != MidiResetMode::XG {
            return vec![];
        }

        let mut effects = vec![];
        if let Some(byte_length) = data.get(2..=3).map(|d| (d[0] as u16) << 7 | d[1] as u16)
            && let Some(r_addr) = data.get(4..=6).map(|d| MemoryAddr::try_from(d))
            && let Ok(mut addr) = r_addr
            && let Some(checksum) = data.last()
            && *checksum == calc_checksum(&data)
        {
            (0..byte_length).for_each(|i| {
                if let Some(value) = data.get(i as usize + 7)
                    && let Ok(eff) = e.ram.set(addr, *value)
                {
                    effects.extend(eff);
                }
                addr.inc();
            });
        }

        effects
    }
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum XGWriteMode {
    Bulk,
    Single = 0x10,
}
