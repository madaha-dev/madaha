use libmadaha::yxg50::{
    BinTbl, drum_setup::DrumSetupEntry as YXG50DrumSetupEntry, pre_voice::Element,
};

use crate::voice_manager::{
    DRUM_BANK_MSB_GM2, DRUM_BANK_MSB_GS, DRUM_BANK_MSB_XG,SFX_BANK_MSB_XG,
    drum_setup::DrumSetupEntry,
    keys::Key,
    program::Program,
    sample_meta::{SampleMeta, SampleMetaFactory},
    Instruments,
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
    // sparse: 128³ slots × Option<Box<Program>> (Vec heap allocation)
    let mut banks: Instruments = vec![vec![vec![None; 128]; 128]; 128];
    // Step 1: Melody
    melody_instruments(b, &mut banks);
    // Step 2: Drums
    percussion_instruments(b, &mut banks);
    // Step 3: SFX
    sfx_instruments(b, &mut banks);
    banks
}

fn melody_instruments(b: &BinTbl, inst: &mut Instruments) {
    // memoize: slots with the same prevoice index share one Program (Arc), avoiding separate allocation for 2.09M slots
    let mut cache: std::collections::HashMap<usize, Option<std::sync::Arc<Program>>> =
        std::collections::HashMap::new();
    for msb in 0..128 {
        for lsb in 0..128 {
            for prog in 0..128 {
                let prevoice_selector = b.get_program_index(msb, lsb, prog);
                if let Some(program) = cache.get(&prevoice_selector) {
                    inst[msb as usize][lsb as usize][prog as usize] = program.clone();
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
                cache.insert(prevoice_selector, program);
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
    for prog in 0..128 {
        for note in 0..128 {
            // GS
            load_drums(
                b,
                DRUM_BANK_MSB_GS,
                b.gs_drum_kit_table[prog] as usize,
                note,
                inst,
            );

            // XG
            load_drums(
                b,
                DRUM_BANK_MSB_XG,
                b.xg_drum_kit_table[prog] as usize,
                note,
                inst,
            );

            // GM2
            load_drums(
                b,
                DRUM_BANK_MSB_GM2,
                b.gm2_drum_kit_table[prog] as usize,
                note,
                inst,
            );
        }
    }

    fn load_drums(b: &BinTbl, bank_msb: usize, prog: usize, note: u8, inst: &mut Instruments) {
        if let Some(ds) = b.get_drum(prog, note) {
            let keydef = if ds.drum_key_type == 0 {
                sfx_key(b, ds, note)
            } else {
                let sm: &'static SampleMeta = Box::leak(Box::new(SampleMeta::from(ds)));
                let ds = DrumSetupEntry::from(ds);
                Some(Key {
                    note,
                    layers: [Some((0, 127, sm)), None],
                    drum_setup: Some(ds),
                })
            };

            let slot = inst[bank_msb][0][prog]
                .get_or_insert_with(|| {
                    std::sync::Arc::new(Program::from(std::array::from_fn(|_| None)))
                });
            std::sync::Arc::make_mut(slot)[note as usize] = keydef.map(Box::new)
        }
    }
}

fn sfx_instruments(b: &BinTbl, inst: &mut Instruments) {
    for note in 0..128 {
        // XG SFX Kit 1
        load_xg_sfx(b, 0, note, inst);
        // XG SFX Kit 2
        load_xg_sfx(b, 1, note, inst);
    }

    fn load_xg_sfx(b: &BinTbl, prog: u8, note: u8, inst: &mut Instruments) {
        let sfx_def = b.xg_sfx_kit_table[prog as usize] as usize;
        if let Some(ds) = b.get_drum(sfx_def, note) {
            if ds.drum_key_type != 0 {
                return;
            }
            if let Some(key) = sfx_key(b, ds, note) {
                let slot = inst[SFX_BANK_MSB_XG][0][prog as usize]
                    .get_or_insert_with(|| {
                        std::sync::Arc::new(Program::from(std::array::from_fn(|_| None)))
                    });
                std::sync::Arc::make_mut(slot)[note as usize] = Some(Box::new(key));
            }
        }
    }
}

fn sfx_key(b: &BinTbl, ds: &YXG50DrumSetupEntry, note: u8) -> Option<Key> {
    if let Some((elm, _)) = b.get_prevoice(b.sfx_index_table[ds.sfx_sound_id as usize] as usize) {
        let mut sms = vec![];
        b.get_sample_meta(&mut sms, elm.index as usize);
        let sms: &'static [SampleMeta] =
            Box::leak(sms.iter().map(|sm| SampleMeta::new(&elm, sm)).collect());

        Key::new(note, sms, &None, None);
    }
    None
}
