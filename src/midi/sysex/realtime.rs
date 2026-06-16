use super::super::consts::DEFAULT_MASTER_VOLUME;
use super::super::engine::Engine;
use super::interface;
use crate::midi::sysex::SYSEX_CHANNEL_ALL_DEVICE;

const SUB_ID1_DEVICE_CONTROL: u8 = 0x04;
const SUB_ID1_MIDI_TUNING_STANDARD_MTS: u8 = 0x08;

// Should be used when sub_id = 4;
const SUB_ID2_MASTER_VOLUME: u8 = 0x01;
// Should be used when sub_id = 8;
const SUB_ID2_SINGLE_NOTE_RETUNE: u8 = 0x02;
const SUB_ID2_SINGLE_BANK_NOTE_RETUNE: u8 = 0x07;

#[derive(Debug)]
pub struct UniversalRealtimeSysEx {}

impl interface::Event for UniversalRealtimeSysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) {
        let dev_id = data.get(0).unwrap_or(&SYSEX_CHANNEL_ALL_DEVICE);
        if *dev_id != e.dev_id || *dev_id != SYSEX_CHANNEL_ALL_DEVICE {
            return;
        }

        let sub_id1 = get_or_skip!(data, 1);
        let sub_id2 = get_or_skip!(data, 2);
        match *sub_id1 {
            SUB_ID1_DEVICE_CONTROL => match *sub_id2 {
                SUB_ID2_MASTER_VOLUME => Self::change_master_volume(e, data),
                _ => return,
            },
            SUB_ID1_MIDI_TUNING_STANDARD_MTS => match *sub_id2 {
                SUB_ID2_SINGLE_NOTE_RETUNE => Self::single_note_retune(e, data),
                SUB_ID2_SINGLE_BANK_NOTE_RETUNE => Self::single_bank_note_retune(e, data),
                _ => return,
            },
            _ => return,
        }
    }
}

impl UniversalRealtimeSysEx {
    fn change_master_volume(e: &mut Engine, data: Box<[u8]>) {
        let volume_lsb = get_or_skip!(data, 3);
        let volume_msb = get_or_skip!(data, 4);
        let volume: u16 = (*volume_msb as u16) << 8 | *volume_lsb as u16;
        e.master_volume = (volume <= DEFAULT_MASTER_VOLUME).then_some(volume).unwrap();
    }

    fn single_note_retune(e: &mut Engine, data: Box<[u8]>) {
        let tune_prog = get_or_skip!(data, 3);
        let note_count = get_or_skip!(data, 4);
        for i in 0..(*note_count as usize) {
            let key = get_or_skip!(data, 5 + i * 4);
            let base_note = get_or_skip!(data, 6 + i * 4);
            let tune_msb = get_or_skip!(data, 7 + i * 4);
            let tune_lsb = get_or_skip!(data, 8 + i * 4);

            if *base_note == 0x7F && *tune_msb == 0x7F && *tune_lsb == 0x7F {
                continue;
            }
            let cent = Self::calc_retune_cent(base_note, tune_msb, tune_lsb);

            e.note_cent_table[0][*tune_prog as usize][*key as usize] = cent;
        }
    }

    fn single_bank_note_retune(e: &mut Engine, data: Box<[u8]>) {
        let bank = get_or_skip!(data, 3);
        let tune_prog = get_or_skip!(data, 4);
        let note_count = get_or_skip!(data, 5);
        for i in 0..(*note_count as usize) {
            let key = get_or_skip!(data, 6 + i * 4);
            let base_note = get_or_skip!(data, 7 + i * 4);
            let tune_msb = get_or_skip!(data, 8 + i * 4);
            let tune_lsb = get_or_skip!(data, 9 + i * 4);

            if *base_note == 0x7F && *tune_msb == 0x7F && *tune_lsb == 0x7F {
                continue;
            }
            let cent = Self::calc_retune_cent(base_note, tune_msb, tune_lsb);

            e.note_cent_table[*bank as usize][*tune_prog as usize][*key as usize] = cent;
        }
    }

    #[inline(always)]
    fn calc_retune_cent(base_note: &u8, tune_msb: &u8, tune_lsb: &u8) -> f64 {
        let frec = ((*tune_msb as u16) << 7 | (*tune_lsb as u16)) as f64;
        (*base_note as f64) * 100.0 + frec / 16383.0 * 100.0
    }
}
