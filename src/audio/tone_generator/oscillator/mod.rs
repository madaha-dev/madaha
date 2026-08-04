mod delay;
mod interpolating;
mod oscillator;
mod peg;
mod portamento;
// 输入音符，配合 pitchbend/RAM/音高偏移 等数据，最终获取音分（f32）
mod pitch;

pub use interpolating::InterpolatingMethods;
pub use oscillator::Oscillator;
