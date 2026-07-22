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
    Finished
}

pub struct PEG {
    state: PEGState,
    counter: u32,
}