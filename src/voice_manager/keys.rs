use super::drum_setup::DrumSetupEntry;
use super::sample_meta::SampleMeta;

#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub note: u8,
    // velocity layer, may have 1 or 2 element(s).
    pub samples: [[Option<&'static SampleMeta>; 2]; 128],
    pub drum_setup: Option<DrumSetupEntry>,
}

impl Key {
    pub fn new(
        note: u8,
        samples0: &'static [SampleMeta],
        samples1: &Option<&'static [SampleMeta]>,
        drum_setup: Option<DrumSetupEntry>,
    ) -> Option<Self> {
        // Main element should not be None, or no Key defination.
        let sample0 = samples0.iter().filter(|&sm| sm.check_key(note)).next()?;
        let sample1 = if let Some(sms) = samples1 {
            sms.iter()
                .filter(|&sm| sm.check_key(note))
                .next()
        } else {
            None
        };

        let mut samples = [[None; 2]; 128];
        for v in sample0.vel_min..=sample0.vel_max {
            samples[v as usize][0] = Some(sample0);
        }
        if let Some(sample) = sample1 {
            for v in sample.vel_min..=sample.vel_max {
                samples[v as usize][1] = Some(sample)
            }
        }

        Some(Self {
            note,
            samples,
            drum_setup,
        })
    }
}
