mod delay;
mod interpolating;
mod oscillator;
mod peg;
mod portamento;
// Input note, combined with pitchbend/RAM/pitch offsets, yields cents (f32)
mod pitch;

pub use interpolating::InterpolatingMethods;
pub use oscillator::Oscillator;
