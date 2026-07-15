use crate::{engine::voice::drum_setup::DrumSetupEntry, voice_manager::errors::TBLErrors};

pub type SampleData = &'static [u8];

#[derive(Debug, Clone, Copy)]
pub struct SampleMeta {
    pub start: usize,
    pub loop_point: usize,
    pub end: usize,
    pub root_tone: u8,
    pub fine_tune: i32,
    pub is_drum: bool, // will not reponse to pitchbend, modulation, etc

    pub drum_setup: Option<DrumSetupEntry>,
    // TODO: may be some another parameters.
}

impl SampleMeta {
    pub fn get_sample(&self, data: SampleData) -> SampleData {
        &data[self.start..self.end]
    }

    pub fn loopable(&self) -> bool {
        self.loop_point < self.end - self.start
    }

    pub fn validate(&self) -> Result<(), TBLErrors> {
        if self.end <= self.start {
            return Err(TBLErrors::InvalidSampleMeta {
                reason: "sample end less than sample start".into(),
            });
        }

        if self.loop_point > self.end - self.start {
            return Err(TBLErrors::InvalidSampleMeta {
                reason: "loop point bigger than sample length".into(),
            });
        }

        Ok(())
    }
}
