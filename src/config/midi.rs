use std::collections::HashMap;

use serde::Deserialize;
use strum_macros::EnumString;

#[derive(Debug, Deserialize, EnumString, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MidiInputEngine {
    Alsa,
    Jack,
    Pipewire,
}

fn default_input_engine() -> MidiInputEngine {
    MidiInputEngine::Alsa
}

use crate::config::{interface::ConfigObject, midi_errors::MidiConfigError};

fn default_poly_replicant() -> u16 {
    150
}

fn default_max_polyphony() -> u16 {
    512
}

fn default_device_id() -> u8 {
    16
}

fn default_master_tune() -> f32 {
    440.0
}

fn default_channel_size() -> usize {
    1024
}

#[derive(Debug, Deserialize, Clone)]
pub struct MidiConfig {
    #[serde(default = "default_poly_replicant")]
    pub poly_replicant: u16,

    /// max polyphony
    #[serde(default = "default_max_polyphony")]
    pub max_polyphony: u16, // never over 512, or boom.

    /// For sysex, should bigger than 16(0x10) or equal.
    #[serde(default = "default_device_id")]
    pub device_id: u8,

    #[serde(default = "default_master_tune")]
    pub master_tune: f32,

    /// Channel size for transmit midi events for audio loop.
    #[serde(default = "default_channel_size")]
    pub channel_size: usize,

    /// MIDI input backend (Alsa/Jack/Pipewire)
    #[serde(default = "default_input_engine")]
    pub input_engine: MidiInputEngine,

    pub scoring: ScoringConfig,
}

impl ConfigObject<MidiConfigError> for MidiConfig {
    fn check(&self) -> Result<(), MidiConfigError> {
        self.check_poly_replicant()?;
        self.check_max_polyphony()?;
        self.check_scoring()?;

        Ok(())
    }
}

impl MidiConfig {
    /// check max polyphony
    fn check_max_polyphony(&self) -> Result<(), MidiConfigError> {
        // 16 * 128 = 2048
        const LIMIT_UPPER: u16 = 2048;
        const LIMIT_LOWER: u16 = 32;

        if !matches!(self.max_polyphony, LIMIT_LOWER..=LIMIT_UPPER) {
            return Err(MidiConfigError::PolyphonyOutOfRange {
                max: self.max_polyphony,
                limit_lower: LIMIT_LOWER,
                limit_upper: LIMIT_UPPER,
            });
        }

        if self.max_polyphony % 16 != 0 {
            return Err(MidiConfigError::InvalidPolyphony {
                poly_phony: self.max_polyphony,
            });
        }
        Ok(())
    }
    
    fn check_poly_replicant(&self) -> Result<(), MidiConfigError> {
        if self.poly_replicant <= 100 {
            return Err(MidiConfigError::BadPolyReplicant {
                value: self.poly_replicant,
            });
        }

        Ok(())
    }

    fn check_scoring(&self) -> Result<(), MidiConfigError> {
        self.scoring.check()
    }
}

fn default_time_weight() -> u32 {
    1000
}

fn default_protect_attack() -> u32 {
    100
}

fn default_penalty_release() -> u32 {
    1500
}

fn default_protect_sustain_pedal() -> u32 {
    100
}

fn default_protect_drum() -> HashMap<u8, u32> {
    const DEFAULT_RATIO: u32 = 50;
    let mut config = HashMap::new();
    for k in [29, 31, 33, 34, 35, 36, 38, 40, 41, 43, 45, 47, 48, 50] {
        config.insert(k, DEFAULT_RATIO);
    }

    config
}

fn default_protect_non_looping() -> u32 {
    500
}

fn default_notes_config() -> HashMap<u8, u32> {
    let mut config = HashMap::new();
    config.insert(0, 500);
    config.insert(48, 1000);
    config.insert(95, 1100);

    config
}

fn default_output_volume() -> HashMap<i32, u32> {
    let mut config = HashMap::new();
    config.insert(-20, 1200);
    config.insert(1, 800);

    config
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScoringConfig {
    /// Ratio for time weight, default 1000/1000.
    #[serde(default = "default_time_weight")]
    pub time_weight: u32,

    /// Ratio for protect attack, default 100/1000.
    #[serde(default = "default_protect_attack")]
    pub protect_attack: u32,

    /// Ratio for penalty release default 1500/1000.
    #[serde(default = "default_penalty_release")]
    pub penalty_release: u32,

    /// Ratio for protect sustain/hold1 (CC#64), default 100/1000.
    #[serde(default = "default_protect_sustain_pedal")]
    pub protect_sustain_pedal: u32,

    /// Ratio for protect drum, default 50/1000
    /// prefer bass drum/snare/tom(29/31/33/34/35/36/38/40/41/43/45/47/48/50)
    #[serde(default = "default_protect_drum")]
    pub protect_drum: HashMap<u8, u32>,

    /// Ratio for protect non-loop sample, default 500/1000
    #[serde(default = "default_protect_non_looping")]
    pub protect_non_looping: u32,

    /// Ratio for notes
    #[serde(default = "default_notes_config")]
    pub notes_config: HashMap<u8, u32>,

    /// Ratio for output volume
    /// dB and weight
    #[serde(default = "default_output_volume")]
    pub volume_config: HashMap<i32, u32>,
}

impl ConfigObject<MidiConfigError> for ScoringConfig {
    fn check(&self) -> Result<(), MidiConfigError> {
        self.check_time_weight()?;
        self.check_protect_attack()?;
        self.check_penalty_release()?;
        self.check_protect_sustain_pedal()?;
        self.check_protect_drum()?;
        self.check_protect_non_looping()?;
        Ok(())
    }
}

impl ScoringConfig {
    pub fn get_note_scoring_map(&self) -> [u32; 128] {
        // We has default weight
        let mut map = [1000u32; 128];

        if self.notes_config.is_empty() {
            return map;
        }

        // Sort config entries by note number
        let mut entries: Vec<(&u8, &u32)> = self.notes_config.iter().collect();
        entries.sort_by_key(|(note, _)| **note);

        // Fill from 0 to first entry with first entry's value
        let first_note = *entries[0].0;
        let first_weight = *entries[0].1;
        for note in 0..=first_note {
            map[note as usize] = first_weight;
        }

        // Fill between consecutive entries with next entry's value
        for window in entries.windows(2) {
            let note_a = *window[0].0;
            let note_b = *window[1].0;
            let weight_b = *window[1].1;
            for note in (note_a + 1)..=note_b {
                map[note as usize] = weight_b;
            }
        }

        // Fill from last entry to 127 with last entry's value
        let last_note = *entries[entries.len() - 1].0;
        let last_weight = *entries[entries.len() - 1].1;
        for note in last_note..128 {
            map[note as usize] = last_weight;
        }

        map
    }

    pub fn get_drum_scoring_map(&self) -> [u32; 128] {
        let mut data = [1000; 128];
        for (&note, &weight) in &self.protect_drum {
            data[note as usize] = weight
        }
        data
    }

    pub fn get_volume_weight(&self, db: f32) -> u32 {
        let db = db as i32;
        if self.volume_config.is_empty() {
            return 1000;
        }

        let mut entries: Vec<(&i32, &u32)> = self.volume_config.iter().collect();
        entries.sort_by_key(|(db, _)| **db);

        // Below first entry: use first entry's weight
        if db <= *entries[0].0 {
            return *entries[0].1;
        }

        // Find which range db falls into
        for window in entries.windows(2) {
            let db_a = *window[0].0;
            let db_b = *window[1].0;
            let weight_b = *window[1].1;
            if db_a < db && db <= db_b {
                return weight_b;
            }
        }

        // Above last entry: use last entry's weight
        *entries[entries.len() - 1].1
    }

    fn check_time_weight(&self) -> Result<(), MidiConfigError> {
        if self.time_weight == 0 {
            return Err(MidiConfigError::BadScoringConfig {
                reason: "time_weight should greater than 0",
            });
        }
        Ok(())
    }

    fn check_protect_attack(&self) -> Result<(), MidiConfigError> {
        if self.protect_attack > 1000 {
            return Err(MidiConfigError::BadScoringConfig {
                reason: "protect_attack should less than 1000",
            });
        }
        Ok(())
    }

    fn check_penalty_release(&self) -> Result<(), MidiConfigError> {
        if self.penalty_release < 1000 {
            return Err(MidiConfigError::BadScoringConfig {
                reason: "penalty_release should greater than 1000",
            });
        }
        Ok(())
    }

    fn check_protect_sustain_pedal(&self) -> Result<(), MidiConfigError> {
        if self.protect_sustain_pedal > 1000 {
            return Err(MidiConfigError::BadScoringConfig {
                reason: "protect_sustain_pedal should less than 1000",
            });
        }
        Ok(())
    }

    fn check_protect_drum(&self) -> Result<(), MidiConfigError> {
        for (_, &weight) in &self.protect_drum {
            if weight > 1000 {
                return Err(MidiConfigError::BadScoringConfig {
                    reason: "protect_drum weight should less than 1000",
                });
            }
        }
        Ok(())
    }

    fn check_protect_non_looping(&self) -> Result<(), MidiConfigError> {
        if self.protect_non_looping > 1000 {
            return Err(MidiConfigError::BadScoringConfig {
                reason: "protect_non_looping should less than 1000",
            });
        }
        Ok(())
    }
}
