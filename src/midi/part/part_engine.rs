#[derive(Debug, Clone, Copy)]
pub enum PartEngine {
    /// Default engine, use wavetable.
    AWM2,
    /// PLG-VL engine, low priority.
    VL,
    /// OPL-3 FM engine, low priority.
    OPL3,
}