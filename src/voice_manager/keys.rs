use super::drum_setup::DrumSetupEntry;
use super::sample_meta::SampleMeta;

/// Velocity layer: (vel_min, vel_max, sample)
pub type VelLayer = (u8, u8, &'static SampleMeta);

/// Key definition (compressed storage: 2 velocity-range layers replace a 128×2 pointer array)
#[derive(Debug, Clone, Copy)]
pub struct Key {
    pub note: u8,
    /// vel layers, at most 2 (element 0/1)
    pub layers: [Option<VelLayer>; 2],
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
        let layer0 = (sample0.vel_min, sample0.vel_max, sample0);
        let layer1 = if let Some(sms) = samples1 {
            sms.iter()
                .filter(|&sm| sm.check_key(note))
                .next()
                .map(|sm| (sm.vel_min, sm.vel_max, sm))
        } else {
            None
        };

        Some(Self {
            note,
            layers: [Some(layer0), layer1],
            drum_setup,
        })
    }

    /// Get the element sample by velocity (i=0/1); velocity out of range → None
    #[inline]
    pub fn sample_at(&self, vel: u8, element: usize) -> Option<&'static SampleMeta> {
        self.layers
            .get(element)
            .and_then(|l| l.and_then(|(lo, hi, sm)| (vel >= lo && vel <= hi).then_some(sm)))
    }

    /// Number of elements hit by the current velocity (0-2)
    #[inline]
    pub fn element_count(&self, vel: u8) -> usize {
        self.sample_at(vel, 0).is_some() as usize + self.sample_at(vel, 1).is_some() as usize
    }
}
