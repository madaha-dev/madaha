use libmadaha::yxg50::{
    BinTbl, drum_setup::DrumSetupEntry as YXG50DrumSetupEntry, pre_voice::Element,
};
use std::sync::Arc;
use wd_log::log_info_ln;

use crate::voice_manager::{
    DRUM_BANK_MSB_GM2, DRUM_BANK_MSB_GS, DRUM_BANK_MSB_XG, Instruments, SFX_BANK_MSB_XG,
    drum_setup::DrumSetupEntry,
    keys::Key,
    program::Program,
    sample_meta::{SampleMeta, SampleMetaFactory},
};
/// ## XG Bank MSB Categories
/// |  -  |   0    |           1          |       2      |       3         |       4      |      5       |       6      |          7         |
/// | --- | ------ | -------------------- | ------------ | --------------- | ------------ | ------------ | ------------ | ------------------ |
/// |  0  | XG PCM | XG for Sample Voices | XG for PCM   | Model Exclusive | XG SFX       | XG for PCM   | XG for PCM   |                    |
/// |  1  |        |                      | XG for VA    |                 | XG for VA    | XG for VA    | XG for VA    |                    |
/// |  2  |        |                      | XG for SG    |                 | XG for SG    | XG for SG    | XG for SG    |                    |
/// |  3  |        |                      | XG for FM    |                 | XG for FM    | XG for FM    | XG for FM    |                    |
/// |  4  |        |                      | XG for AN    |                 | XG for AN    | XG for AN    | XG for AN    |                    |
/// |  5  |        |                      | XG Extension |                 | XG SFX 2     | XG Extension | XG Extension |                    |
/// |  6  |        |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension |                    |
/// |  7  |        |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension |                    |
/// |  8  | XG PCM |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension | GM2 Percussion     |
/// |  9  |        |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension | GM2 Sound          |
/// |  A  |        |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension |                    |
/// |  B  |        |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension | (GS Drum internal) |
/// |  C  |        |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension |                    |
/// |  D  |        |                      | XG Extension |                 | XG Extension | XG Extension | XG Extension |                    |
/// |  E  |        |                      | XG Extension | Model-Exclusive | XG Extension | XG Extension | XG Extension | XG SFX Kit         |
/// |  F  |        |                      | XG for Kit   |                 | XG for Kit   | XG Extension | XG Extension | XG Drum Kit        |
pub fn parse_syxg50(b: &BinTbl) -> Instruments {
    log_info_ln!("found tbl type: S-YXG50");
    // sparse: 128³ slots × Option<Box<Program>> (Vec heap allocation)
    let mut banks: Instruments = vec![vec![vec![None; 128]; 128]; 128];
    // Step 1: Melody
    melody_instruments(b, &mut banks);
    log_info_ln!("melody instruments loaded");
    // Step 2: Drums
    percussion_instruments(b, &mut banks);
    log_info_ln!("drumkit instruments loaded");
    // Step 3: SFX
    sfx_instruments(b, &mut banks);
    log_info_ln!("SFX loaded");
    banks
}

fn melody_instruments(b: &BinTbl, inst: &mut Instruments) {
    // memoize: slots with the same prevoice index share one Program (Arc),
    // avoiding separate allocation for 2.09M slots. prevoiceIdx ∈ [0, 0x10000)
    // （u16 表值）→ 固定长度 Box 切片索引（不用 HashMap / Vec）。
    // 用 into_boxed_slice 直接在堆上分配，避免 Box<[T; N]> 的 512KB 栈临时量
    // （会把测试线程默认 2MB 栈撑爆）。
    let mut cache: Box<[Option<Arc<Program>>]> = vec![None; 0x10000].into_boxed_slice();
    for msb in 0..128 {
        for lsb in 0..128 {
            for prog in 0..128 {
                let prevoice_selector = b.get_program_index(msb, lsb, prog);
                if let Some(program) = &cache[prevoice_selector] {
                    inst[msb as usize][lsb as usize][prog as usize] = Some(program.clone());
                    continue;
                }
                let prevoice = b.get_prevoice(prevoice_selector);
                let program = if let Some((elm0, o_elm1)) = prevoice {
                    // Element 0
                    let mut samples = vec![];
                    b.get_sample_meta(&mut samples, elm0.index as usize);
                    // convert to voice_manager::SampleMeta
                    let samples0 = load_elements(b, &elm0);
                    // Element 1 (optional)
                    let samples1 = o_elm1.as_ref().map(|e| load_elements(b, e));

                    let mut keys: [Option<Box<Key>>; 128] = std::array::from_fn(|_| None);
                    for k in elm0.key_min..=elm0.key_max {
                        keys[k as usize] = Key::new(k, &samples0, &samples1, None).map(Box::new)
                    }

                    Some(std::sync::Arc::new(Program::from(keys)))
                } else {
                    None
                };
                inst[msb as usize][lsb as usize][prog as usize] = program.clone();
                cache[prevoice_selector] = program;
            }
        }
    }

    fn load_elements(b: &BinTbl, elm: &Element) -> &'static [SampleMeta] {
        let mut samples = vec![];
        b.get_sample_meta(&mut samples, elm.index as usize);
        let vec = samples.iter().map(|sm| SampleMeta::new(elm, sm)).collect();
        Box::leak(vec)
    }
}

fn percussion_instruments(b: &BinTbl, inst: &mut Instruments) {
    // 鼓 SampleMeta / SFX 采样按 drum entry 索引共享——同一 entry 被多
    // bank/prog 引用时避免反复 clone PCM（否则 ~1.5 万键 × 每键克隆 → OOM）。
    // 用固定长度 Box 切片索引（0..312），不用 HashMap / 指针。
    let mut drum_cache: Box<[Option<&'static SampleMeta>]> =
        vec![None; b.drum_note_param_table.len()].into_boxed_slice();
    let mut sfx_cache: Box<[Option<&'static [SampleMeta]>]> =
        vec![None; b.drum_note_param_table.len()].into_boxed_slice();

    for prog in 0..128 {
        for note in 0..128 {
            // GS / XG / GM2（slot 按 MIDI program 索引，内部查 drum kit 表）
            load_drums(
                b,
                DRUM_BANK_MSB_GS,
                prog,
                note,
                inst,
                &mut drum_cache,
                &mut sfx_cache,
            );
            load_drums(
                b,
                DRUM_BANK_MSB_XG,
                prog,
                note,
                inst,
                &mut drum_cache,
                &mut sfx_cache,
            );
            load_drums(
                b,
                DRUM_BANK_MSB_GM2,
                prog,
                note,
                inst,
                &mut drum_cache,
                &mut sfx_cache,
            );
        }
    }

    fn load_drums(
        b: &BinTbl,
        bank_msb: usize,
        prog: usize,
        note: u8,
        inst: &mut Instruments,
        drum_cache: &mut Box<[Option<&'static SampleMeta>]>,
        sfx_cache: &mut Box<[Option<&'static [SampleMeta]>]>,
    ) {
        // `prog` is the MIDI program number: the slot is indexed by program,
        // while the drum kit (seg00-03 table) selects the actual DrumData row.
        let kit = match bank_msb {
            DRUM_BANK_MSB_GS => b.gs_drum_kit_table[prog] as usize,
            DRUM_BANK_MSB_XG => b.xg_drum_kit_table[prog] as usize,
            DRUM_BANK_MSB_GM2 => b.gm2_drum_kit_table[prog] as usize,
            _ => 0,
        };
        let Some(idx) = b.get_drum_index(kit, note) else {
            return;
        };
        let ds = &b.drum_note_param_table[idx];
        let keydef = if ds.drum_key_type == 0 {
            sfx_key(b, ds, note, idx, sfx_cache)
        } else {
            let sm = match drum_cache[idx] {
                Some(s) => s,
                None => {
                    let s: &'static SampleMeta =
                        Box::leak(Box::new(SampleMeta::from(ds)));
                    drum_cache[idx] = Some(s);
                    s
                }
            };
            let ds = DrumSetupEntry::from(ds);
            Some(Key {
                note,
                layers: [Some((0, 127, sm)), None],
                drum_setup: Some(ds),
            })
        };

        let slot = inst[bank_msb][0][prog]
            .get_or_insert_with(|| Arc::new(Program::from(std::array::from_fn(|_| None))));
        Arc::make_mut(slot)[note as usize] = keydef.map(Box::new)
    }
}

fn sfx_instruments(b: &BinTbl, inst: &mut Instruments) {
    let mut sfx_cache: Box<[Option<&'static [SampleMeta]>]> =
        vec![None; b.drum_note_param_table.len()].into_boxed_slice();

    for note in 0..128 {
        // XG SFX Kit 1
        load_xg_sfx(b, 0, note, inst, &mut sfx_cache);
        // XG SFX Kit 2
        load_xg_sfx(b, 1, note, inst, &mut sfx_cache);
    }

    fn load_xg_sfx(
        b: &BinTbl,
        prog: u8,
        note: u8,
        inst: &mut Instruments,
        sfx_cache: &mut Box<[Option<&'static [SampleMeta]>]>,
    ) {
        let sfx_def = b.xg_sfx_kit_table[prog as usize] as usize;
        let Some(idx) = b.get_drum_index(sfx_def, note) else {
            return;
        };
        let ds = &b.drum_note_param_table[idx];
        if ds.drum_key_type != 0 {
            return;
        }
        if let Some(key) = sfx_key(b, ds, note, idx, sfx_cache) {
            let slot = inst[SFX_BANK_MSB_XG][0][prog as usize]
                .get_or_insert_with(|| Arc::new(Program::from(std::array::from_fn(|_| None))));
            Arc::make_mut(slot)[note as usize] = Some(Box::new(key));
        }
    }
}

fn sfx_key(
    b: &BinTbl,
    ds: &YXG50DrumSetupEntry,
    note: u8,
    idx: usize,
    sfx_cache: &mut Box<[Option<&'static [SampleMeta]>]>,
) -> Option<Key> {
    // seg06[sfxSoundID] 是 byte offset into seg13（base_prevoice），而
    // get_prevoice 接受 prevoiceIdx（byte offset = idx × 2）→ 需 ÷2。
    let prevoice_idx = b.sfx_index_table[ds.sfx_sound_id as usize] as usize / 2;

    let sms = match sfx_cache[idx] {
        Some(s) => s,
        None => {
            let (elm, _) = b.get_prevoice(prevoice_idx)?;
            let mut sms = vec![];
            b.get_sample_meta(&mut sms, elm.index as usize);
            let s: &'static [SampleMeta] = Box::leak(
                sms.iter().map(|sm| SampleMeta::new(&elm, sm)).collect(),
            );
            sfx_cache[idx] = Some(s);
            s
        }
    };

    Key::new(note, sms, &None, Some(DrumSetupEntry::from(ds)))
}
