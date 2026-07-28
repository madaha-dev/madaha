use libmadaha::yxg50::{BinTbl, pre_voice::Element};

use super::super::{
    keys::Key,
    program::Program,
    sample_meta::{SampleMeta, SampleMetaFactory},
    voice_manager::Instruments,
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
    let mut banks: Instruments = [[[Program::from([None; 128]); 128]; 128]; 128];
    // TODO: how can i parse tbl?
    // Step 1: Melody
    melody_instruments(b, &mut banks);
    // Step 2: Drums && SFX Kits

    banks
}

fn melody_instruments(b: &BinTbl, inst: &mut Instruments) {
    for msb in 0..128 {
        for lsb in 0..128 {
            for prog in 0..128 {
                let prevoice_selector = b.get_program_index(msb, lsb, prog);
                let prevoice = b.get_prevoice(prevoice_selector);
                if let Some((elm0, o_elm1)) = prevoice {
                    // Element 0
                    let mut samples = vec![];
                    b.get_sample_meta(&mut samples, elm0.index as usize);
                    // convert to voice_manager::SampleMeta
                    let samples0 = load_elements(b, &elm0);
                    // Element 1 (optional)
                    let samples1 = o_elm1.as_ref().map(|e| load_elements(b, e));

                    let mut keys = [None; 128];
                    for k in elm0.key_min..=elm0.key_max {
                        // Will 
                        keys[k as usize] = Key::new(k, &samples0, &samples1, None)
                    }

                    inst[msb as usize][lsb as usize][prog as usize] = Program::from(keys)
                }
            }
        }
    }
}

fn load_elements(b: &BinTbl, elm: &Element) -> &'static [SampleMeta] {
    let mut samples = vec![];
    b.get_sample_meta(&mut samples, elm.index as usize);
    let vec = samples.iter().map(|sm| SampleMeta::new(elm, sm)).collect();
    Box::leak(vec)
}

fn percussion_instruments(b: &BinTbl, inst: &mut Instruments) {
    
}