use std::collections::VecDeque;

use wd_log::log_debug_ln;

use crate::{
    midi::{
        Engine, MIDICallbackEffects,
        consts::DEFAULT_MASTER_TUNING,
        engine::{MidiResetMode, tuning_14bit_to_xg},
        ram::{
            MemoryAddr,
            interface::Memory,
            xg::{drum_setup::DrumSetup, drum_setup_wrapper::DrumSetupWrapper},
        },
    },
    voice_manager::{DRUM_BANK_MSB_GM2, DRUM_BANK_MSB_GS, DRUM_BANK_MSB_XG},
};

impl Engine {
    pub fn hook_exec(&mut self, callbacks: Vec<MIDICallbackEffects>) {
        let mut queue = VecDeque::from(callbacks);
        while let Some(callback) = queue.pop_back() {
            use MIDICallbackEffects::*;
            log_debug_ln!("queue={:?}", queue);
            match callback {
                NoEffect => continue,
                ChangeResetMode { mode } => self.reset(mode),
                ChangeProgram {
                    part_id,
                    program,
                    bank_msb,
                    bank_lsb,
                } => {
                    let mut part_mode = 0;
                    self.parts[part_id].write_with(|part| {
                        part.set_program(&self.voice_manager, bank_msb, bank_lsb, program);
                        part_mode = part.get_ram().part_mode;
                    });
                    if part_mode > 1 {
                        queue.extend(vec![SetDrumSetup {
                            set: part_mode,
                            bank_msb,
                            program,
                        }]);
                    }
                }
                SetDrumSetup {
                    set,
                    bank_msb,
                    program,
                } => {
                    if let Some(drum_setup) = self
                        .voice_manager
                        .get_drum_setup(bank_msb, program)
                        .map(|d| d.map(|ds| DrumSetup::from(ds)))
                        && set > 1
                    {
                        self.ram.xg.drum_setup.write_with(|a| {
                            a[set as usize - 2] = DrumSetupWrapper {
                                program,
                                drum_setup,
                            };
                        });

                        // XG Spec v2.0 3.2.1.7.7.
                        // Assume that Parts 1 and 2 are both set to Drum Setup 1.
                        // If a Program Change is now sent to Part 1 to change to the Jazz Kit,
                        // then Part 2's voices also automatically change to the Jazz Kit.
                        self.parts
                            .iter()
                            .filter(|p| p.snapshot().get_ram().part_mode == set)
                            .for_each(|p| {
                                p.write_with(|p| {
                                    p.set_program(&self.voice_manager, bank_msb, 0, program)
                                });
                            });
                    }
                }
                SetPartModeToRhythm {
                    part_id,
                    drum_set_id,
                } => {
                    let msb = {
                        use MidiResetMode::*;
                        (match self.ram.reset_mode {
                            GM | GS => DRUM_BANK_MSB_GS,
                            XG => DRUM_BANK_MSB_XG,
                            GM2 => DRUM_BANK_MSB_GM2,
                        }) as u8
                    };
                    let prog = self.ram.xg.drum_setup.snapshot()[drum_set_id as usize].program;
                    self.parts[part_id].write_with(|part| {
                        part.ram.write_with(|multi_part| {
                            multi_part.bank_select_lsb = 0;
                            multi_part.bank_select_msb = msb;
                        });
                        part.set_program(&self.voice_manager, msb, 0, prog);
                    });
                }
                SetPartModeToDrums { part_id } => {
                    let prev = self.parts[part_id].snapshot().prev_rhythm;
                    let _ = self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x01),
                        prev.msb,
                    );
                    let _ = self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x02),
                        prev.lsb,
                    );
                    queue.extend(
                        self.ram
                            .set(
                                MemoryAddr::new(0x08, part_id as u8, 0x03),
                                prev.prog,
                            )
                            .unwrap_or(vec![]),
                    );
                }
                SetPartModeToMelodic { part_id } => {
                    let prev = self.parts[part_id].snapshot().prev_melodic;
                    let _ = self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x01),
                        prev.msb,
                    );
                    let _ = self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x02),
                        prev.lsb,
                    );
                    queue.extend(
                        self.ram
                            .set(
                                MemoryAddr::new(0x08, part_id as u8, 0x03),
                                prev.prog,
                            )
                            .unwrap_or(vec![]),
                    );
                }
                BackupBankSet {
                    part_id,
                    bank_msb,
                    bank_lsb,
                    program,
                    current_part_mode,
                } => {
                    self.parts[part_id].write_with(|part| {
                        if current_part_mode == 0 {
                            part.prev_melodic.set(bank_msb, bank_lsb, program);
                        } else if current_part_mode == 1 {
                            part.prev_rhythm.set(bank_msb, bank_lsb, program);
                        }
                    });
                }
                ResetDrumSetup { setup_id } => {
                    self.ram.xg.drum_setup
                        .write_with(|a| a[setup_id as usize].reset());
                }
                InsertionEffectON { for_part, eff_id } => {
                    self.parts[for_part as usize]
                        .write_with(|part| part.insertion_effects.push(eff_id));
                }
                InsertionEffectOFF { for_part, eff_id } => {
                    self.parts[for_part as usize]
                        .write_with(|part| part.insertion_effects.retain(|e| *e != eff_id));
                }
                ResetAllParameter => {
                    self.master_tuning = DEFAULT_MASTER_TUNING;
                    self.ram.xg.system.write_with(|s| s.set_master_tune(0x0400));
                    self.reset(self.ram.reset_mode);
                }
                ChannelResetAllController { part_id } => {
                    self.parts[part_id].write_with(|part| {
                        part.controller.reset();
                        part.pitchbend = 0x2000;
                    });
                }
                ChangeMasterTuning { tuning } => {
                    self.master_tuning = tuning;
                    self.ram
                        .xg
                        .system
                        .write_with(|s| s.set_master_tune(tuning_14bit_to_xg(tuning)));
                }
                _ => {
                    log_debug_ln!("non-proceed callback: {:?}", callback);
                }
            }
        }
    }
}
