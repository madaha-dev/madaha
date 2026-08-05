use super::super::consts::DEFAULT_MASTER_VOLUME;
use super::super::engine::Engine;
use super::interface;
use crate::midi::MIDICallbackEffects;
use crate::midi::sysex::SYSEX_CHANNEL_ALL_DEVICE;

use super::consts::{
    SUB_ID1_DEVICE_CONTROL, SUB_ID1_GM2_DESTINATION, SUB_ID1_GM2_SYSTEM, SUB_ID1_MTS,
    SUB_ID2_CC_DEST, SUB_ID2_CHANNEL_PRESSURE_DEST, SUB_ID2_GM2_EFFECTS, SUB_ID2_KEY_BASED,
    SUB_ID2_MASTER_VOLUME, SUB_ID2_SINGLE_BANK_NOTE_RETUNE, SUB_ID2_SINGLE_NOTE_RETUNE,
};

#[derive(Debug)]
pub struct UniversalRealtimeSysEx {}

impl interface::Event for UniversalRealtimeSysEx {
    fn parse(e: &mut Engine, data: Box<[u8]>) -> Vec<MIDICallbackEffects> {
        let dev_id = get_dev_id!(data);
        if (dev_id == e.dev_id || dev_id == SYSEX_CHANNEL_ALL_DEVICE)
            && let Some(sub_id1) = data.get(1)
            && let Some(sub_id2) = data.get(2)
        {
            match (*sub_id1, *sub_id2) {
                (SUB_ID1_DEVICE_CONTROL, SUB_ID2_MASTER_VOLUME) => {
                    Self::change_master_volume(e, data)
                }
                (SUB_ID1_MTS, SUB_ID2_SINGLE_NOTE_RETUNE) => {
                    Self::single_note_retune(e, data)
                }
                (SUB_ID1_MTS, SUB_ID2_SINGLE_BANK_NOTE_RETUNE) => {
                    Self::single_bank_note_retune(e, data)
                }
                // GM2 Effect 1/2 (Reverb/Chorus): 7F <dev> 04 05 01 01 01 01 <fx> <pp> <vv>
                (SUB_ID1_DEVICE_CONTROL, SUB_ID2_GM2_EFFECTS) => Self::gm2_effect_params(e, data),
                // GM2 Key-Based Instrument Controllers: 7F <dev> 0A 01 <0n> <kk> <nn> <vv>
                (SUB_ID1_GM2_SYSTEM, SUB_ID2_KEY_BASED) => Self::key_based_controllers(e, data),
                // GM2 Channel Pressure Destination: 7F <dev> 09 01 <0n> <pp> <rr>
                (SUB_ID1_GM2_DESTINATION, SUB_ID2_CHANNEL_PRESSURE_DEST) => {
                    Self::channel_pressure_destination(e, data)
                }
                // GM2 Control Change Destination: 7F <dev> 09 03 <0n> <cc> <pp> <rr>
                (SUB_ID1_GM2_DESTINATION, SUB_ID2_CC_DEST) => Self::cc_destination(e, data),
                _ => {
                    // do nothing
                }
            }
        }

        vec![]
    }
}

impl UniversalRealtimeSysEx {
    fn change_master_volume(e: &mut Engine, data: Box<[u8]>) {
        let volume_lsb = get_or_skip!(data, 3);
        let volume_msb = get_or_skip!(data, 4);
        let volume: u16 = (*volume_msb as u16) << 8 | *volume_lsb as u16;
        let volume = (volume <= DEFAULT_MASTER_VOLUME).then_some(volume).unwrap();
        e.master_volume = volume;
        e.audio_master_volume.write_with(|m| *m = volume);
    }

    fn single_note_retune(e: &mut Engine, data: Box<[u8]>) {
        Self::bank_note_retune(e, 0, data);
    }

    fn single_bank_note_retune(e: &mut Engine, data: Box<[u8]>) {
        let bank = get_or_skip!(data, 3);
        Self::bank_note_retune(e, *bank as usize, data);
    }

    /// GM2 Effect Parameters: 7F <dev> 04 05 01 01 01 01 <fx> <pp> <vv>
    /// fx: 01=Reverb, 02=Chorus; pp: 00=type, 01=time/rate, 02=level/depth, 03=feedback, 04=send-to-reverb
    fn gm2_effect_params(e: &mut Engine, data: Box<[u8]>) {
        let fx_id = get_or_skip!(data, 7);
        let pp = get_or_skip!(data, 8);
        let vv = get_or_skip!(data, 9);

        match *fx_id {
            1 => {
                // Reverb
                match *pp {
                    0 => {
                        // GM2 reverb type: 0-7 → XG (msb, lsb)
                        let (msb, lsb): (u8, u8) = match vv {
                            0 => (0x01, 0x00), // Hall1
                            1 => (0x01, 0x01), // Hall2
                            2 => (0x02, 0x00), // Room1
                            3 => (0x02, 0x01), // Room2
                            4 => (0x02, 0x02), // Room3
                            5 => (0x03, 0x00), // Stage1
                            6 => (0x03, 0x01), // Stage2
                            _ => (0x04, 0x00), // Plate
                        };
                        e.ram.xg.effect1.write_with(|fx| {
                            fx.reverb.type_msb = msb;
                            fx.reverb.type_lsb = lsb;
                        });
                    }
                    1 => {
                        e.ram.xg.effect1.write_with(|fx| fx.reverb.param1 = *vv);
                    }
                    2 => {
                        e.ram.xg.effect1.write_with(|fx| fx.reverb.reverb_return = *vv);
                    }
                    _ => {}
                }
            }
            2 => {
                // Chorus
                match *pp {
                    0 => {
                        // GM2 chorus type: 0-7 → XG (0x41/0x42 + lsb)
                        let (msb, lsb): (u8, u8) = if *vv < 4 {
                            (0x41, *vv) // Chorus1-4
                        } else {
                            (0x42, (*vv).saturating_sub(4)) // Celeste1-4
                        };
                        e.ram.xg.effect1.write_with(|fx| {
                            fx.chorus.type_msb = msb;
                            fx.chorus.type_lsb = lsb;
                        });
                    }
                    1 => {
                        e.ram.xg.effect1.write_with(|fx| fx.chorus.param1 = *vv); // rate
                    }
                    2 => {
                        e.ram.xg.effect1.write_with(|fx| fx.chorus.param2 = *vv); // depth
                    }
                    3 => {
                        e.ram.xg.effect1.write_with(|fx| fx.chorus.param3 = *vv); // feedback
                    }
                    4 => {
                        e.ram.xg.effect1.write_with(|fx| fx.chorus.send_to_reverb = *vv);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// GM2 Key-Based Instrument Controllers: 7F <dev> 0A 01 <0n> <kk> <nn> <vv>
    /// nn: 07=volume→DrumSetup.level, 0A=pan→DrumSetup.pan
    fn key_based_controllers(e: &mut Engine, data: Box<[u8]>) {
        let ch = get_or_skip!(data, 3) & 0x0F;
        let key = get_or_skip!(data, 4);
        let nn = get_or_skip!(data, 5);
        let vv = get_or_skip!(data, 6);

        // Find the part whose rcv_channel matches, then its drum setup index (part_mode-2)
        let mut setup_idx = None;
        for part in 0..0x10 {
            let ram = e.ram.xg.multi_part[part].snapshot();
            if ram.rcv_channel == ch {
                let pm = ram.part_mode as usize;
                if pm >= 2 {
                    setup_idx = Some(pm - 2);
                }
                break;
            }
        }
        let Some(setup_idx) = setup_idx else { return };
        let note_idx = (key.saturating_sub(12) as usize).min(78);

        match *nn {
            7 => {
                // Volume
                e.ram.xg.drum_setup.write_with(|a| a[setup_idx][note_idx].level = *vv);
            }
            0x0A => {
                // Pan
                e.ram.xg.drum_setup.write_with(|a| a[setup_idx][note_idx].pan = *vv);
            }
            _ => {}
        }
    }

    /// GM2 Channel Pressure Destination: 7F <dev> 09 01 <0n> <pp> <rr>
    /// pp: 01=Pitch, 02=Filter, 03=Volume → MultiPartExt.cat.* control depths
    fn channel_pressure_destination(e: &mut Engine, data: Box<[u8]>) {
        let ch = get_or_skip!(data, 3) & 0x0F;
        let pp = get_or_skip!(data, 4);
        let rr = get_or_skip!(data, 5);

        for part in 0..0x10 {
            let ram = e.ram.xg.multi_part[part].snapshot();
            if ram.rcv_channel == ch {
                e.ram.xg.multi_part[part].write_with(|m| match *pp {
                    1 => m.cat.pitch_control = *rr,
                    2 => m.cat.filter_control = *rr,
                    3 => m.cat.amplitude_control = *rr,
                    _ => {}
                });
                break;
            }
        }
    }

    /// GM2 Control Change Destination: 7F <dev> 09 03 <0n> <cc> <pp> <rr>
    /// Maps the CC to XG Assignable Controller 1 (controller_number + depths)
    fn cc_destination(e: &mut Engine, data: Box<[u8]>) {
        let ch = get_or_skip!(data, 3) & 0x0F;
        let cc = get_or_skip!(data, 4);
        let pp = get_or_skip!(data, 5);
        let rr = get_or_skip!(data, 6);

        for part in 0..0x10 {
            let ram = e.ram.xg.multi_part[part].snapshot();
            if ram.rcv_channel == ch {
                e.ram.xg.multi_part[part].write_with(|m| {
                    m.ac[0].controller_number = *cc;
                    match *pp {
                        1 => m.ac[0].pitch_control = *rr,
                        2 => m.ac[0].filter_control = *rr,
                        3 => m.ac[0].amplitude_control = *rr,
                        _ => {}
                    }
                });
                break;
            }
        }
    }

    fn bank_note_retune(e: &mut Engine, bank: usize, data: Box<[u8]>) {
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

            e.note_cent_table[bank as usize][*tune_prog as usize][*key as usize] = cent;
        }
    }

    #[inline(always)]
    fn calc_retune_cent(base_note: &u8, tune_msb: &u8, tune_lsb: &u8) -> f32 {
        let frec = ((*tune_msb as u16) << 7 | (*tune_lsb as u16)) as f32;
        (*base_note as f32) * 100.0 + frec / 16383.0 * 100.0
    }
}
