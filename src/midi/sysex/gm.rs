use crate::midi::engine::MidiResetMode::{GM, GM2};
use crate::midi::ram::RAMCallbackEffects;

use super::super::engine::Engine;
use super::SYSEX_CHANNEL_ALL_DEVICE;
use super::interface;

const SUB_ID1_GM: u8 = 0x09;

const SUB_ID2_GM_SYSTEM_ON: u8 = 0x01;
const SUB_ID2_GM_SYSTEM_OFF: u8 = 0x02; // not used.
const SUB_ID2_GM2_SYSTEM_ON: u8 = 0x03;
const SUB_ID2_GM2_MASTER_VOLUME: u8 = 0x04;
const SUB_ID2_GM2_MASTER_FINE_TUNING: u8 = 0x05;
const SUB_ID2_GM2_MASTER_COARSE_TUNING: u8 = 0x06;
const SUB_ID2_GM2_REVERB: u8 = 0x07;
const SUB_ID2_GM2_CHORUS: u8 = 0x08;
const SUB_ID2_GM2_DELAY: u8 = 0x08;

// ── Universal SysEx (0x7E / 0x7F) ──────────────────────────────────
// F0 [7E|7F] [channel] [sub-id1] [payload...] F7
#[derive(Clone, Debug)]
pub struct GeneralMIDISysEx {}

impl interface::Event for GeneralMIDISysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) -> Vec<RAMCallbackEffects> {
        let dev_id = get_dev_id!(data);
        if (dev_id == e.dev_id || dev_id == SYSEX_CHANNEL_ALL_DEVICE)
            && let Some(sub_id1) = data.get(1)
            && let Some(sub_id2) = data.get(2)
        {
            match (*sub_id1, *sub_id2) {
                (SUB_ID1_GM, SUB_ID2_GM_SYSTEM_ON) => {
                    vec![RAMCallbackEffects::ChangeResetMode { mode: GM }]
                }
                (SUB_ID1_GM, SUB_ID2_GM2_SYSTEM_ON) => {
                    vec![RAMCallbackEffects::ChangeResetMode { mode: GM2 }]
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }
}
