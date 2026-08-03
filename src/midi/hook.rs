use std::collections::VecDeque;

use wd_log::log_debug_ln;

use crate::{
    midi::{
        Engine,
        engine::MidiResetMode,
        ram::{
            MemoryAddr, RAMCallbackEffects,
            interface::Memory,
            xg::{drum_setup::DrumSetup, drum_setup_wrapper::DrumSetupWrapper},
        },
    },
    voice_manager::{DRUM_BANK_MSB_GM2, DRUM_BANK_MSB_GS, DRUM_BANK_MSB_XG},
};

impl Engine {
    pub fn hook_exec(&mut self, callbacks: Vec<RAMCallbackEffects>) {
        let mut queue = VecDeque::from(callbacks);
        while let Some(callback) = queue.pop_back() {
            use RAMCallbackEffects::*;
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
                    let part = &mut self.parts[part_id];
                    part.set_program(&self.voice_manager, bank_msb, bank_lsb, program);
                    if let Some(p) = part.get_ram()
                        && p.part_mode > 1
                    {
                        queue.extend(vec![SetDrumSetup { bank_msb, program }]);
                    }
                }
                SetDrumSetup { bank_msb, program } => {
                    if let Some(drum_setup) = self
                        .voice_manager
                        .get_drum_setup(bank_msb, program)
                        .map(|d| d.map(|ds| DrumSetup::from(ds)))
                    {
                        self.ram.xg.drum_setup.iter_mut().for_each(|f| {
                            *f = DrumSetupWrapper {
                                program,
                                drum_setup,
                            }
                        });
                    }
                }
                SetPartModeToRhythm {
                    part_id,
                    drum_set_id,
                } => {
                    let part = &mut self.parts[part_id];

                    let msb = {
                        if let Some(mut multi_part) = part.get_ram_mut() {
                            multi_part.bank_select_lsb = 0;
                            use MidiResetMode::*;
                            let msb = match self.ram.reset_mode {
                                GM | GS => DRUM_BANK_MSB_GS,
                                XG => DRUM_BANK_MSB_XG,
                                GM2 => DRUM_BANK_MSB_GM2,
                            } as u8;
                            multi_part.bank_select_msb = msb;
                            msb
                        } else {
                            continue;
                        }
                    };

                    let prog = self.ram.xg.drum_setup[drum_set_id as usize].program;
                    part.set_program(&self.voice_manager, msb, 0, prog);
                }
                SetPartModeToDrums { part_id } => {
                    let part = &mut self.parts[part_id];
                    self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x01),
                        part.prev_rhythm.msb,
                    );
                    self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x02),
                        part.prev_rhythm.lsb,
                    );
                    queue.extend(
                        self.ram
                            .set(
                                MemoryAddr::new(0x08, part_id as u8, 0x03),
                                part.prev_rhythm.prog,
                            )
                            .unwrap_or(vec![]),
                    );
                }
                SetPartModeToMelodic { part_id } => {
                    let part = &mut self.parts[part_id];
                    self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x01),
                        part.prev_melodic.msb,
                    );
                    self.ram.set(
                        MemoryAddr::new(0x08, part_id as u8, 0x02),
                        part.prev_melodic.lsb,
                    );
                    queue.extend(
                        self.ram
                            .set(
                                MemoryAddr::new(0x08, part_id as u8, 0x03),
                                part.prev_melodic.prog,
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
                    let part = &mut self.parts[part_id];
                    if current_part_mode == 0 {
                        part.prev_melodic.set(bank_msb, bank_lsb, program);
                    } else if current_part_mode == 1 {
                        part.prev_rhythm.set(bank_msb, bank_lsb, program);
                    }
                }
                ResetDrumSetup { setup_id } => {
                    self.ram.xg.drum_setup[setup_id as usize].reset();
                }
                InsertionEffectON { for_part, eff_id } => {
                    self.parts[for_part as usize].insertion_effects.push(eff_id)
                }
                InsertionEffectOFF { for_part, eff_id } => self.parts[for_part as usize]
                    .insertion_effects
                    .retain(|e| *e != eff_id),
                _ => {
                    log_debug_ln!("non-proceed callback: {:?}", callback);
                }
            }
        }
    }
}
