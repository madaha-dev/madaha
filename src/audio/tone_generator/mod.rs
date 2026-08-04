/// # Tone Genration Model
///
/// >From XG Spec 2.00
///
/// ```
/// LFO +--------------------------+---------------------+
///     |                          |                     |
///     +-> Delay  >-| Resonance >-| CutOFf >-+    AEG >-|
///     Portamento >-|       FEG >-|          | Volume >-|
///            PEG >-|    CutOff >-|          |          |
///          Pitch >-|             |          |          |  EQ >-| Pan >-|
///              Oscillator ----> LPF -----> HPF -----> Amp ---> EQ ---> Pan -> ...
/// ```
/// Figure shows the XG tone-generation model.
/// Specifically, the Figure shows the model of the tone-generation module provided for each part.
/// The module includes an oscillator, LPF, amplifier, and pan (with HPF and EQ available as options).
///
/// The module includes a pitch envelo)pe generator (PEG) that time-modulates the pitch,
/// an amplitude envelope generator (AEG) that time-modulates the volume,
/// and a filter envelope generator (FEG) that time-modulates the low-pass-filter's cutoff frequency.
/// An LFO (low frequency oscillator) implements periodic modulation of pitch, filtering, and volume.
///
/// These features are mainly controlled using the various part parameters that can be set for each part.
///
/// # Pseudocoding
///
/// ```
/// let delay = Delay(LFO.pitch_output)
/// let portamento = Portamento(...)
/// let peg = PEG(attackTime, initialLevel, releaseTime, releaseLevel)
/// let pitch = Pitch(...)
/// let oscillator = Oscillator(delay, portamento, peg, pitch)
///
/// let resonance = Resonance(...)
/// let feg = FEG(attackTime, decayTime, releaseTime)
/// let cutOff = CutOff(...)
/// let lpf = LPF(LFO.lpf_output, feg, cutOff)
///
/// let cutOff = CutOff(...)
/// let hpf = HPF(cutOff)
///
/// let aeg = AEG(attackTime, decayTime, releaseTime)
/// let amp = Amp(LFO.amp_output, aeg, volume)
///
/// let eq = EQ(...)
///
/// Output.input(oscillator).input(lpf).input(hpf).input(amp).input(eq).input(pan).to_output()
/// ```
///
pub mod oscillator;
mod lpf;
mod hpf;
mod amp;
mod pan;
mod eq;
pub mod interface;
mod tone_generator;
mod types;

pub use amp::Amp;
pub use eq::EQ;
pub use hpf::HPF;
pub use pan::Pan;
pub use lpf::LPF;
pub use tone_generator::ToneGenerator;
pub use tone_generator::ToneGeneratorStatus;
