#[derive(Debug)]
struct EGCnt(i32);
impl EGCnt {
    pub fn to_cent(self) -> i32 {
        (self.0 * 0x4b) >> 0xc
    }

    pub fn from_cent(cent: i32) -> Self {
        Self((cent << 0xc) / 0x4b)
    }
}

#[derive(Debug)]
enum PEGState {
    Hold,
    Stage1Level,
    Stage1Time,
    Stage2Level,
    Stage2Time,
    Stage3Level,
    Stage3Time,
    Sustain,
    Release,
    Finished,
}

pub struct PEG {
    state: PEGState,
    counter: u32,
    current_output: EGCnt,

    attack_time: u32,   // Unit: ms
    initial_level: u32, // cents
    release_time: u32,  // Unit: ms
    release_level: u32, // cents
}
