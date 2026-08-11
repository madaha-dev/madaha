use std::sync::{Arc, LazyLock};
use std::time::Duration;

use wd_log::log_debug_ln;

use crate::audio::interface::Audio;

use super::super::interface::ToneGeneratorInterface;
use super::delay::Delay;
use super::interpolating::InterpolatingMethods;
use super::peg::PEG;
use super::pitch::Pitch;
use super::portamento::Portamento;

use crate::midi::Part;
use crate::voice_manager::SampleMeta;

/// Precomputed 2^(cents/1200) for cents in [-11520, +11520] (≈ ±8 octaves —
/// covers low keys played through high-region samples, e.g. glockenspiel
/// samples with a high base key; a narrow ±3.4-octave table clamped every
/// lower key to one identical pitch), 1 cent per entry, linear interpolation
/// between entries (sub-cent accuracy). The per-frame
/// `(cents * ln2/1200).exp()` was a per-voice hotspot; a lookup keeps the
/// DDS advance at plain multiply-adds.
static CENTS_TO_RATIO: LazyLock<[f32; 23041]> = LazyLock::new(|| {
    let mut t = [0.0f32; 23041];
    for (i, e) in t.iter_mut().enumerate() {
        *e = 2f32.powf((i as f32 - 11520.0) / 1200.0);
    }
    t
});

/// cents → frequency ratio = 2^(cents/1200), table lookup + linear interpolation
#[inline]
pub fn cents_to_ratio(cents: f32) -> f32 {
    let x = (cents + 11520.0).clamp(0.0, 23040.0);
    let i = x as usize;
    let f = x - i as f32;
    CENTS_TO_RATIO[i] * (1.0 - f) + CENTS_TO_RATIO[(i + 1).min(23040)] * f
}

#[derive(Debug)]
pub struct Oscillator {
    pub peg: PEG,
    pub delay: Delay,
    pub portamento: Portamento,
    pub pitch: Pitch,
    pub velocity: u8,
    /// External modulation (MW/Bend/CAT/PAT pitch control), in cents, updated each block
    pub pitch_mod: f32,

    /// Bound sample metadata (including PCM data)
    sample: Option<&'static SampleMeta>,
    /// DDS playback position (in samples, f64 to prevent drift)
    pos: f64,
    /// One-shot sample exhausted (set when pos passes the end with no loop);
    /// the renderer ends the voice so the AEG doesn't run on a silent source.
    pub finished: bool,
    /// Interpolation method
    pub interpolating: InterpolatingMethods,
    /// LFO waveform type (0-12, matches 2006LE)
    pub lfo_wave: u8,

    /// Source (sample) sample rate, kept for retargeting play_speed_base
    source_rate: f32,
    // source_sample_rate / target_sample_rate
    pub play_speed_base: f64,
    /// Bound part (melodic/drum mode etc., set at play time)
    part: Option<Arc<crate::double_buffer::DoubleBuffered<Part>>>,
}

impl Oscillator {
    pub fn new(source_sample_rate: f32, target_sample_rate: f32) -> Self {
        Self {
            pitch: Pitch::new(),
            peg: PEG::new(),
            delay: Delay::new(),
            portamento: Portamento::new(),
            velocity: 0,
            pitch_mod: 0.0,

            source_rate: source_sample_rate,
            play_speed_base: source_sample_rate as f64 / target_sample_rate as f64,
            interpolating: InterpolatingMethods::Linear,

            sample: None,
            pos: 0.0,
            finished: false,
            lfo_wave: 0,
            part: None,
        }
    }

    /// Bind the owning part (set at play time; no structural refactor of the call chain)
    pub fn bind_part(&mut self, part: Arc<crate::double_buffer::DoubleBuffered<Part>>) {
        self.part = Some(part);
    }

    /// Retarget to a different output rate (the sink's actual negotiated rate)
    pub fn set_target_rate(&mut self, target_sample_rate: f32) {
        self.play_speed_base = self.source_rate as f64 / target_sample_rate as f64;
    }

    pub fn set_sample(&mut self, sample: &'static SampleMeta) {
        log_debug_ln!(
            "sample loop_point=0x{:x}, loop_length=0x{:x}, as a character value",
            sample.loop_point,
            sample.loop_length
        );
        self.sample = Some(sample);
        self.pos = 0.0;
    }

    /// Bound sample metadata (pitch reference for diagnostics)
    pub fn sample_ref(&self) -> Option<&'static SampleMeta> {
        self.sample
    }

    /// Initialize sound parameters from SampleMeta (S-YXG50 element).
    ///
    /// Alignment notes (S-YXG50 data vs 2006LE program):
    /// - Aligned: coarse / fine (table lookup) / pitch_offset / tone / loop / pcm
    /// - PEG: `peg_rate0-4` conversion table not parsed → neutral values (see `PEG::setup`)
    /// - LFO: `lfo_wave` 0-12 matches 2006LE → mapped directly
    /// - Part level (08 pp: vibrato/bend/detune/note_shift) is read by the 2006LE
    ///   program from MultiPart; real-time voice modulation to be wired in
    pub fn setup(&mut self, sample: &'static SampleMeta, note: u8, vel: u8, sample_rate: f32) {
        self.set_sample(sample);
        self.finished = false;
        self.velocity = vel;
        self.pitch.note = note;
        self.pitch.note_in_cent = note as f32 * 100.0;
        // No glide by default: source = target → portamento outputs 0
        self.portamento
            .begin(self.pitch.note_in_cent, self.pitch.note_in_cent, 0.0);
        // PEG: S-YXG50 element[22..30] + velocity + key position
        self.peg.setup(sample, note, vel, sample_rate);
        self.lfo_wave = sample.lfo_wave & 0x07;
    }

    /// The note cannot be passed directly; the cent value must be looked up from a table,
    /// computed from the GM optional tuning standard. Madaha implements these standards.
    pub fn set_note(&mut self, note: u8, cent_table: [f32; 128]) {
        let note_in_cent = cent_table[note as usize];
        self.pitch.note_in_cent = note_in_cent;
        self.pitch.note = note;
        self.portamento.target_note = note_in_cent;
    }

    pub fn set_lfo(&mut self, lfo_input: f32) {
        self.delay.lfo_input = lfo_input;
    }

    pub fn is_drum(&self) -> bool {
        self.part
            .as_ref()
            .map_or(false, |p| p.snapshot().is_drum_channel())
    }

    pub fn is_looping(&self) -> bool {
        if let Some(sm) = self.sample {
            sm.loop_length != 0
        } else {
            true
        }
    }

    pub fn play(&mut self, _p: f32, _part: Arc<crate::double_buffer::DoubleBuffered<Part>>) {
        // TODO: bind part + real-time pitch computation
    }
}

impl ToneGeneratorInterface for Oscillator {
    fn reset(&mut self) {}

    fn kill(&mut self) {
        self.peg.kill();
    }

    fn release(&mut self) {}
}

impl Audio for Oscillator {
    /// Called once per sample, returns the current sample value
    fn tick(&mut self, elapsed: Duration) -> f32 {
        let Some(sample) = self.sample else {
            return 0.0;
        };
        let Some(pcm) = sample.pcm.as_deref() else {
            return 0.0;
        };
        if pcm.is_empty() {
            return 0.0;
        }

        // 1. Real-time cents: note + modulation
        let note_in_cent = self.delay.tick(elapsed)
            + self.peg.tick(elapsed)
            + self.portamento.tick(elapsed)
            + self.pitch.tick(elapsed)
            + self.pitch_mod;
        // 2. cent → frequency ratio: ratio = 2^(cents/1200)
        //    The PCM content is recorded at 22050Hz but played back at 44100Hz
        //    1:1 (×2 trick in the data files), so baseKey IS the sample's design
        //    pitch. Playing note N steps at 2^((N - baseKey)/12); note and
        //    baseKey share the same key numbering (Yamaha A3 = MIDI A4 = 69 =
        //    440Hz). seg16 data[2] (tone) fine-tunes the recorded content in
        //    cents. No octave compensation and no element-coarse term (coarse
        //    is not a pitch offset in the playback chain).
        let ratio_cents = note_in_cent
            - sample.get_base_note_cent()
            + sample.get_tone();
        let ratio = cents_to_ratio(ratio_cents) as f64;

        // 3. DDS advance: step = ratio × (source_sr / target_sr)
        self.pos += ratio * self.play_speed_base;

        // 4. Position wrap-around
        let len = pcm.len() as f64;
        if sample.loop_length > 0 {
            let loop_len = sample.loop_length as f64;
            if self.pos >= len {
                // Past the sample end (after the loop region) → wrap back
                let loop_start = sample.loop_point as f64;
                self.pos = loop_start + (self.pos - loop_start) % loop_len;
            }
        } else if self.pos >= len {
            // One-shot sample finished: the source is exhausted, so the whole
            // voice ends here (the AEG envelope must not keep running on a
            // silent source — osc and AEG are kept in sync).
            self.finished = true;
            self.pos = len;
            return 0.0;
        }

        // 5. Interpolate the sample
        self.interpolating
            .interpolate(pcm, sample.loop_point, sample.loop_length, self.pos)
    }
}

#[cfg(test)]
mod tests {
    use super::cents_to_ratio;

    #[test]
    fn cents_to_ratio_bounds() {
        // 边界值：表范围 ±11520 音分（±8 八度），极端输入不得 panic
        for c in [-12000.0, -11520.0, -11519.9, 0.0, 11519.9, 11520.0, 12000.0] {
            let r = cents_to_ratio(c);
            assert!(r > 0.0 && r.is_finite(), "c={c} → {r}");
        }
        // 低端必须单调递减（修复前 ±4096 表会把 -4100 以下全部 clamp 成同一音高）
        let r_low = cents_to_ratio(-11500.0);
        let r_high = cents_to_ratio(-11400.0);
        assert!(r_low < r_high, "low range must stay monotonic");
        // 插值一致性：表内连续，1 音分 ≈ 2^(1/1200)
        let r1 = cents_to_ratio(100.0);
        let r2 = cents_to_ratio(101.0);
        assert!((r2 / r1 - 2f32.powf(1.0 / 1200.0)).abs() < 1e-3);
    }
}
