#![allow(dead_code)] // protocol constants, documented values
use crate::midi::engine::MidiResetMode::{GM, GM2};
use crate::midi::MIDICallbackEffects;

use super::consts::{
    SUB_ID1_DEVICE_CONTROL, SUB_ID1_MIDI_TUNING, SUB_ID2_MASTER_COARSE_TUNING,
    SUB_ID2_MASTER_FINE_TUNING, SUB_ID2_MASTER_VOLUME, SUB_ID2_SCALE_OCTAVE_TUNING,
};
use super::super::consts::DEFAULT_MASTER_VOLUME;
use super::super::engine::{Engine, tuning_14bit_to_xg};
use super::SYSEX_CHANNEL_ALL_DEVICE;
use super::interface;

const SUB_ID1_GM: u8 = 0x09;

const SUB_ID2_GM_SYSTEM_ON: u8 = 0x01;
//const SUB_ID2_GM_SYSTEM_OFF: u8 = 0x02; // not used.
const SUB_ID2_GM2_SYSTEM_ON: u8 = 0x03;

// ── Universal SysEx (0x7E / 0x7F) ──────────────────────────────────
// F0 [7E|7F] [channel] [sub-id1] [payload...] F7
#[derive(Clone, Debug)]
pub struct GeneralMIDISysEx {}

impl interface::Event for GeneralMIDISysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) -> Vec<MIDICallbackEffects> {
        let dev_id = get_dev_id!(data);
        if (dev_id == e.dev_id || dev_id == SYSEX_CHANNEL_ALL_DEVICE)
            && let Some(sub_id1) = data.get(1)
            && let Some(sub_id2) = data.get(2)
        {
            match (*sub_id1, *sub_id2) {
                (SUB_ID1_GM, SUB_ID2_GM_SYSTEM_ON) => {
                    vec![MIDICallbackEffects::ChangeResetMode { mode: GM }]
                }
                (SUB_ID1_GM, SUB_ID2_GM2_SYSTEM_ON) => {
                    vec![MIDICallbackEffects::ChangeResetMode { mode: GM2 }]
                }
                // GM2 Universal Non-Real Time (7E) device control
                // Master Volume: 7E <dev> 04 01 01 <LSB> <MSB>
                (SUB_ID1_DEVICE_CONTROL, SUB_ID2_MASTER_VOLUME) => {
                    Self::master_volume(e, data);
                    vec![]
                }
                // Master Coarse Tuning: 7E <dev> 04 02 01 <semitones>
                (SUB_ID1_DEVICE_CONTROL, SUB_ID2_MASTER_COARSE_TUNING) => {
                    Self::master_coarse_tuning(e, data);
                    vec![]
                }
                // Master Fine Tuning: 7E <dev> 04 03 01 <LSB> <MSB>
                (SUB_ID1_DEVICE_CONTROL, SUB_ID2_MASTER_FINE_TUNING) => {
                    Self::master_fine_tuning(e, data);
                    vec![]
                }
                // Scale/Octave Tuning Adjust: 7E <dev> 08 01 <note> <adjust>
                (SUB_ID1_MIDI_TUNING, SUB_ID2_SCALE_OCTAVE_TUNING) => {
                    Self::scale_octave_tuning(e, data);
                    vec![]
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }
}

impl GeneralMIDISysEx {
    /// GM2 Master Volume (14-bit) → engine.master_volume + audio double-buffer
    fn master_volume(e: &mut Engine, data: Box<[u8]>) {
        let lsb = get_or_skip!(data, 4);
        let msb = get_or_skip!(data, 5);
        let volume: u16 = (*msb as u16) << 8 | *lsb as u16;
        let volume = (volume <= DEFAULT_MASTER_VOLUME).then_some(volume).unwrap();
        e.master_volume = volume;
        e.audio_master_volume.write_with(|m| *m = volume);
    }

    /// GM2 Master Coarse Tuning (64=0, ±64 semitones) → System.transpose
    fn master_coarse_tuning(e: &mut Engine, data: Box<[u8]>) {
        let semi = get_or_skip!(data, 4);
        e.ram.xg.system.write_with(|s| s.transpose = *semi);
    }

    /// GM2 Master Fine Tuning (14-bit, 0x2000=A440) → System.master_tune
    fn master_fine_tuning(e: &mut Engine, data: Box<[u8]>) {
        let lsb = get_or_skip!(data, 4);
        let msb = get_or_skip!(data, 5);
        let tune: u16 = (*msb as u16) << 8 | *lsb as u16;
        e.master_tuning = tune;
        e.ram
            .xg
            .system
            .write_with(|s| s.set_master_tune(tuning_14bit_to_xg(tune)));
    }

    /// GM2 Scale/Octave Tuning Adjust: per-note global tuning → all parts' scale_tuning[note%12]
    fn scale_octave_tuning(e: &mut Engine, data: Box<[u8]>) {
        let note = get_or_skip!(data, 3);
        let adj = get_or_skip!(data, 4);
        let idx = (*note % 12) as usize;
        let value = *adj;
        for part in 0..0x10 {
            e.ram.xg.multi_part[part].write_with(|m| m.scale_tuning[idx] = value);
        }
    }
}
