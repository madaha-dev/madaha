use crate::midi::sysex::interface;

use super::super::engine::{Engine, MidiResetMode};
use wd_log::log_debug_ln;

// ── Roland SysEx (0x41) ────────────────────────────────────────────
// F0 41 [device-id] [model-id] [msg-type] [addr 4B] [data...] [checksum] F7
#[derive(Clone, Debug)]
pub struct RolandSysEx {
    /// Device ID: 0x10 = ch1, 0x11 = ch2, ..., 0x7F = all devices
    pub device_id: u8,
    /// Model ID: 0x42 = GS
    pub model_id: u8,
    /// Message type: 0x12 = DT1 (write), 0x11 = RQ1 (request)
    pub msg_type: u8,
    /// 4-byte address
    pub address: [u8; 4],
    /// Variable-length data payload
    pub data: Box<[u8]>,
    /// Roland checksum: (~(device_id + model_id + msg_type + addr + data) + 1) & 0x7F
    pub checksum: u8,
}

impl RolandSysEx {
    /// Minimum: device_id + model_id + msg_type + 4 addr + 1 checksum = 7
    const MIN_LEN: usize = 7;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < Self::MIN_LEN {
            return None;
        }
        let device_id = data[0];
        let model_id = data[1];
        let msg_type = data[2];
        let address = [data[3], data[4], data[5], data[6]];
        let checksum = *data.last()?;
        let data_payload = if data.len() > Self::MIN_LEN {
            &data[7..data.len() - 1]
        } else {
            &[]
        };
        Some(Self {
            device_id,
            model_id,
            msg_type,
            address,
            data: data_payload.into(),
            checksum,
        })
    }

    /// Verify Roland checksum
    pub fn verify_checksum(&self) -> bool {
        let sum: u16 = self.device_id as u16
            + self.model_id as u16
            + self.msg_type as u16
            + self.address.iter().map(|b| *b as u16).sum::<u16>()
            + self.data.iter().map(|b| *b as u16).sum::<u16>();
        let computed = ((!sum + 1) & 0x7F) as u8;
        computed == self.checksum
    }
}

impl interface::Event for RolandSysEx {
    fn on(&self, e: &mut Engine) {
        // Verify checksum first
        if !self.verify_checksum() {
            log_debug_ln!("roland sysex checksum mismatch, ignoring");
            return;
        }

        // GS Reset: model_id=0x42, msg_type=0x12, addr=40 00 7F 00
        if self.model_id == 0x42
            && self.msg_type == 0x12
            && self.address == [0x40, 0x00, 0x7F, 0x00]
        {
            log_debug_ln!("GS Reset received (device={:#04X})", self.device_id);
            e.reset_mode = MidiResetMode::GS;
        }
    }
}
