use std::sync::Arc;
use std::time::Duration;

use crate::audio::interface::Audio;

use super::delay::Delay;
use super::interpolating::InterpolatingMethods;
use super::peg::PEG;
use super::pitch::Pitch;
use super::portamento::Portamento;
use super::super::interface::ToneGeneratorInterface;

use crate::midi::Part;
use crate::voice_manager::SampleMeta;

/// ln(2) / 1200 —— cent → frequency ratio
const LN2_OVER_1200: f64 = 0.000577_622_650_319_656_4;

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
    /// Interpolation method
    pub interpolating: InterpolatingMethods,
    /// LFO waveform type (0-12, matches 2006LE)
    pub lfo_wave: u8,

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

            play_speed_base: source_sample_rate as f64 / target_sample_rate as f64,
            interpolating: InterpolatingMethods::Linear,

            sample: None,
            pos: 0.0,
            lfo_wave: 0,
            part: None,
        }
    }

    /// Bind the owning part (set at play time; no structural refactor of the call chain)
    pub fn bind_part(&mut self, part: Arc<crate::double_buffer::DoubleBuffered<Part>>) {
        self.part = Some(part);
    }

    pub fn set_sample(&mut self, sample: &'static SampleMeta) {
        self.sample = Some(sample);
        self.pos = 0.0;
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
        self.velocity = vel;
        self.pitch.note = note;
        self.pitch.note_in_cent = note as f32 * 100.0;
        self.portamento.target_note = self.pitch.note_in_cent;
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
        self.part.as_ref().map_or(false, |p| p.snapshot().is_drum_channel())
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
        let Some(pcm) = sample.pcm else {
            return 0.0;
        };
        if pcm.is_empty() {
            return 0.0;
        }

        // 1. Real-time cents: note + modulation + element offset
        let note_in_cent = self.delay.tick(elapsed)
            + self.peg.tick(elapsed)
            + self.portamento.tick(elapsed)
            + self.pitch.tick(elapsed)
            + self.pitch_mod
            + sample.get_coarse_in_cent()
            + sample.get_fine_in_cent(self.velocity)
            + sample.get_pitch_offset();

        // 2. cent → frequency ratio: ratio = 2^(cents/1200)
        //    (key - base) × 100 + tone + element offset
        let ratio_cents =
            note_in_cent - sample.get_base_note_cent() + sample.get_tone();
        let ratio = (ratio_cents as f64 * LN2_OVER_1200).exp();

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
            // One-shot sample finished
            self.pos = len;
            return 0.0;
        }

        // 5. Interpolate the sample
        self.interpolating
            .interpolate(pcm, sample.loop_point, sample.loop_length, self.pos)
    }
}
