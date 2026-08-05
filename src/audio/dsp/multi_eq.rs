/// Multi-part EQ (main output 5-band)
///
/// XG Spec: main bus → master_volume → MultiEQ → Master Attenuator → output
/// - eq_type: 0=Flat(bypass), 1=Jazz, 2=Pops, 3=Rock, 4=Concert (presets provided by RAM default data)
/// - each band: gain (64=0dB), frequency (XG_EQ_FREQ_TABLE), q, shape (0=shelving, 1=peaking)
use super::core::biquad::{Biquad, make_biquad};
use crate::midi::effect_params::parameter_table::XG_EQ_FREQ_TABLE;
use crate::midi::ram::xg::multi_eq::MultiEQ;

#[derive(Debug)]
pub struct MultiEqDsp {
    bands: [Biquad; 5],
    enabled: [bool; 5],
}

impl MultiEqDsp {
    pub fn new() -> Self {
        Self {
            bands: [Biquad::new(), Biquad::new(), Biquad::new(), Biquad::new(), Biquad::new()],
            enabled: [false; 5],
        }
    }

    /// Configure from MultiEQ RAM
    pub fn set_from(&mut self, eq: &MultiEQ, sample_rate: f32) {
        let flat = eq.eq_type == 0;
        let bands = [&eq.band1, &eq.band2, &eq.band3, &eq.band4, &eq.band5];
        for (i, band) in bands.iter().enumerate() {
            if flat || (band.gain == 0x40 && band.frequency == 0) {
                self.enabled[i] = false;
                self.bands[i] = Biquad::new();
                continue;
            }
            let gain_db = (band.gain as f32 - 64.0) / 64.0 * 12.0;
            let freq = XG_EQ_FREQ_TABLE[(band.frequency as usize).min(60)];
            let q = (band.q as f32 / 10.0).clamp(0.1, 12.0);
            let peak = band.shape != 0;
            self.enabled[i] = true;
            self.bands[i] = make_biquad(gain_db, freq, q, peak, sample_rate);
        }
    }

    #[inline]
    pub fn process(&mut self, input: (f32, f32)) -> (f32, f32) {
        let (mut l, mut r) = input;
        for (i, b) in self.bands.iter_mut().enumerate() {
            if self.enabled[i] {
                l = b.tick(l);
                r = b.tick(r);
            }
        }
        (l, r)
    }
}

impl Default for MultiEqDsp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::ram::xg::multi_eq::EQBand;

    fn band(gain: u8, freq: u8, q: u8, shape: u8) -> EQBand {
        EQBand { gain, frequency: freq, q, shape }
    }

    #[test]
    fn flat_bypasses() {
        let eq = MultiEQ { eq_type: 0, band1: band(0x40, 0, 0, 0), band2: band(0x40, 0, 0, 0), band3: band(0x40, 0, 0, 0), band4: band(0x40, 0, 0, 0), band5: band(0x40, 0, 0, 0) };
        let mut dsp = MultiEqDsp::new();
        dsp.set_from(&eq, 44100.0);
        let out = dsp.process((0.5, -0.5));
        assert!((out.0 - 0.5).abs() < 1e-6);
        assert!((out.1 + 0.5).abs() < 1e-6);
    }

    #[test]
    fn bass_band_boosts() {
        let eq = MultiEQ { eq_type: 1, band1: band(0x7F, 10, 7, 0), band2: band(0x40, 0, 0, 0), band3: band(0x40, 0, 0, 0), band4: band(0x40, 0, 0, 0), band5: band(0x40, 0, 0, 0) };
        let mut dsp = MultiEqDsp::new();
        dsp.set_from(&eq, 44100.0);
        // 100Hz sine amplified by +12dB shelf
        let mut peak: f32 = 0.0;
        let mut phase: f32 = 0.0;
        for _ in 0..4410 {
            let input = (phase * 2.0 * std::f32::consts::PI).sin();
            phase = (phase + 100.0f32 / 44100.0) % 1.0;
            let (l, _) = dsp.process((input, input));
            peak = peak.max(l.abs());
        }
        assert!(peak > 1.5, "peak={peak}");
    }
}
