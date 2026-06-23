use std::ops::IndexMut;

use wd_log::log_warn_ln;

use super::consts::{DEFAULT_CONTROLLER_ROUTES, DEFAULT_PARTS_PARAMS};
use super::controller_route::ControllerRoutesTable;
use crate::engine::errors::MidiError;
use crate::engine::ram::MemoryAddr;
use crate::engine::ram::interface::Memory;
use crate::engine::ram::roland::consts::DEFAULT_DRUM_SETUP;
use crate::engine::ram::roland::drum_setup::DrumSetup;
use crate::engine::ram::roland::effect::EffectData;
use crate::engine::ram::roland::part::Part;
use crate::engine::ram::roland::system::System;

#[derive(Debug)]
pub struct RAM {
    /// addr: 40 00 ??
    pub system: System,
    /// addr: 40 01 ??
    pub effect: EffectData,
    /// addr: 40 1? ??
    pub parts: [Part; 0x10],
    /// addr: 40 2? ??,
    pub controller_routes: [ControllerRoutesTable; 0x10],
    /// addr: 41 ?? ??
    pub drum_setup: [[DrumSetup; 128]; 2],
}

impl RAM {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            effect: EffectData::new(),
            parts: DEFAULT_PARTS_PARAMS,
            controller_routes: DEFAULT_CONTROLLER_ROUTES,
            drum_setup: DEFAULT_DRUM_SETUP,
        }
    }

    fn get_drum_setup(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (setup, drum_key, _) = DrumSetup::addr(addr);
        let drum_setup = self.drum_setup.get(setup as usize).ok_or(err.clone())?;
        let drum_key = drum_setup.get(drum_key as usize).ok_or(err.clone())?;

        drum_key.get(addr)
    }

    fn set_drum_setup(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (setup, drum_key, _) = DrumSetup::addr(addr);

        let drum_setup = self.drum_setup.get_mut(setup as usize).ok_or(err.clone())?;
        let drum_key = drum_setup.get_mut(drum_key as usize).ok_or(err.clone())?;

        drum_key.set(addr, value)
    }
}

impl Memory for RAM {
    fn reset(&mut self) {
        self.system.reset();
        self.effect.reset();
        self.parts = DEFAULT_PARTS_PARAMS;
        self.controller_routes = DEFAULT_CONTROLLER_ROUTES;
        self.drum_setup = DEFAULT_DRUM_SETUP;
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        match addr[0] {
            0x40 => match addr[1] {
                0x00 => self.system.set(addr, value),
                0x01 => self.effect.set(addr, value),
                0x10..0x20 => {
                    let channel = (addr[1] & 0xF) as usize;
                    let param = match self.parts.get_mut(channel) {
                        Some(r) => r,
                        None => return Err(err.into()),
                    };
                    param.set(addr, value)
                }
                0x20..0x30 => {
                    let channel = (addr[1] & 0xF) as usize;
                    let controller = match self.controller_routes.get_mut(channel) {
                        Some(r) => r,
                        None => return Err(err.clone()),
                    };

                    controller.set(addr, value)
                }

                _ => Err(err.into()),
            },
            0x41 => self.set_drum_setup(addr, value),
            _ => Err(err.into()),
        }
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        match addr[0] {
            0x40 => match addr[1] {
                0x00 => self.system.get(addr),
                0x01 => self.effect.get(addr),
                0x10..0x20 => {
                    let channel = (addr[1] & 0xF) as usize;
                    let param = match self.parts.get(channel) {
                        Some(r) => r,
                        None => return Err(err.clone()),
                    };
                    param.get(addr)
                }
                0x20..0x30 => {
                    let channel = (addr[1] & 0xF) as usize;
                    let controller = match self.controller_routes.get(channel) {
                        Some(r) => r,
                        None => return Err(err.clone()),
                    };

                    controller.get(addr)
                }

                _ => Err(err.clone()),
            },
            0x41 => self.get_drum_setup(addr),
            _ => Err(err.clone()),
        }
    }
}

impl std::ops::Index<usize> for RAM {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        log_warn_ln!("Use index() for RAM not recommended, or cause panic");
        let hi_addr = index >> 4;
        let mid_addr = (index >> 2) & 0xFF;
        let lo_addr = index & 0xFF;

        match hi_addr {
            0x40 => match mid_addr {
                0x00 => &self.system[lo_addr & 0x7F],
                0x01 => &self.effect[lo_addr & 0x7F],
                0x10..=0x1F => &self.parts[mid_addr & 0xF][lo_addr & 0x7F],
                0x20..=0x2F => {
                    &self.controller_routes[mid_addr & 0xF][lo_addr & 0xF0][lo_addr & 0xF]
                }

                _ => &0xFF,
            },
            0x41 => {
                let map = (mid_addr >> 3) & 1;
                let note = (mid_addr & 0xFF) << 4 | (lo_addr >> 3) & 0xF;
                let param = lo_addr & 0x7;
                &self.drum_setup[map][note][param]
            }

            _ => &0xFF,
        }
    }
}

impl IndexMut<usize> for RAM {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        log_warn_ln!("Use index_mut() for RAM not recommended, or cause panic");
        let hi_addr = index >> 4;
        let mid_addr = (index >> 2) & 0xFF;
        let lo_addr = index & 0xFF;

        match hi_addr {
            0x40 => match mid_addr {
                0x00 => &mut self.system[lo_addr & 0x7F],
                0x01 => &mut self.effect[lo_addr & 0x7F],
                0x10..=0x1F => &mut self.parts[mid_addr & 0xF][lo_addr & 0x7F],
                0x20..=0x2F => {
                    &mut self.controller_routes[mid_addr & 0xF][lo_addr & 0xF0][lo_addr & 0xF]
                }

                _ => panic!("RAM: invalid index {}", index),
            },
            0x41 => {
                let map = (mid_addr >> 3) & 1;
                let note = (mid_addr & 0xFF) << 4 | (lo_addr >> 3) & 0xF;
                let param = lo_addr & 0x7;
                &mut self.drum_setup[map][note][param]
            }

            _ => panic!("RAM: invalid index {}", index),
        }
    }
}
