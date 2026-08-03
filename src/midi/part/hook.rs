use crate::{
    midi::{
        engine::{MidiResetMode},
        ram::{
            MemoryAddr, RAM,
            xg::{drum_setup::DrumSetup, drum_setup_wrapper::DrumSetupWrapper},
        },
    },
    voice_manager::{DRUM_BANK_MSB_GM2, DRUM_BANK_MSB_GS, DRUM_BANK_MSB_XG, VoiceManager},
};

use super::Part;

impl Part {
    pub fn hook_pre_exec(&mut self, ram: &mut RAM, addr: MemoryAddr) {
        let (_, _, l) = addr.split();
        let ram = &mut ram.xg.multi_part[self.id];
        match l {
            0x07 => {
                self.prev_bank_msb = ram.bank_select_msb;
                self.prev_bank_lsb = ram.bank_select_lsb;
                self.prev_program = ram.program_number;
            }
            _ => {
                // Do nothing.
            }
        }
    }

    pub fn hooks_post_exec(&mut self, ram: &mut RAM, addr: MemoryAddr, vm: Option<&VoiceManager>) {
        let (_, _, l) = addr.split();
        match l {
            // Part mode
            0x07 => {
                let multi_part = &mut ram.xg.multi_part[self.id];
                let program = if multi_part.part_mode != 0 {
                    multi_part.rcv_switches.rcv_bank_select = 0;
                    multi_part.bank_select_lsb = 0;
                    use MidiResetMode::*;
                    multi_part.bank_select_msb = match ram.reset_mode {
                        GM | GS => DRUM_BANK_MSB_GS as u8,
                        XG => DRUM_BANK_MSB_XG as u8,
                        GM2 => DRUM_BANK_MSB_GM2 as u8,
                    };
                    let drum_setup = multi_part.part_mode.wrapping_sub(2) as usize;
                    ram.xg.drum_setup[drum_setup].program
                } else {
                    multi_part.rcv_switches.rcv_bank_select = 1;
                    multi_part.bank_select_lsb = self.prev_bank_lsb;
                    multi_part.bank_select_msb = self.prev_bank_msb;
                    self.prev_program
                };
                if let Some(vm) = vm {
                    self.on_program_change(ram, program, vm);
                }
            }
            // Program Change
            0x03 => {
                let multi_part = ram.xg.multi_part[self.id];
                let bank_msb = multi_part.bank_select_msb;
                let bank_lsb = multi_part.bank_select_lsb;
                let program = multi_part.program_number;
                if multi_part.part_mode != 0 {
                    if let Some(vm) = vm {
                        if let Some(drum_setup) = vm.get_drum_setup(bank_msb, program) {
                            let drum_setup = drum_setup.map(|ds| DrumSetup::from(ds));
                            ram.xg.drum_setup.iter_mut().for_each(|f| {
                                *f = DrumSetupWrapper {
                                    program,
                                    drum_setup,
                                }
                            });
                            self.program_entry = Some(vm.get_program(bank_msb, bank_lsb, program))
                        }
                    }
                }
            }
            // MSB Change
            0x01 => {
                let part_mode_memory = MemoryAddr::new(0x08, self.id as u8, 0x07);
                let multi_part = ram.xg.multi_part[self.id];
                if matches!(
                    multi_part.bank_select_msb as usize,
                    DRUM_BANK_MSB_GM2 | DRUM_BANK_MSB_GS | DRUM_BANK_MSB_XG
                ) {
                    self.set_mem(ram, part_mode_memory, 0x02, vm);
                } else {
                    self.set_mem(ram, part_mode_memory, 0x00, vm);
                }
            }
            _ => {
                // Do nothing.
            }
        }
    }
}
