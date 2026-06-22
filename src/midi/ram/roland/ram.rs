use super::consts::{
    DEFAULT_CONTROLLER_ROUTES, DEFAULT_EFFECT_PARAMS, DEFAULT_PARTS_PARAMS, DEFAULT_SYSTEM_PARAMS,
};
use super::controller_route::ControllerRoutesTable;
use crate::midi::errors::MidiError;
use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use crate::midi::ram::roland::consts::DEFAULT_DRUM_SETUP;
use crate::midi::ram::roland::drum_setup::DrumSetup;

#[derive(Debug)]
pub struct RAM {
    /// addr: 40 00 ??
    pub system: [u8; 0x80],
    /// addr: 40 01 ??
    pub effect: [u8; 0x80],
    /// addr: 40 1? ??
    pub parts: [[u8; 0x80]; 0x10],
    /// addr: 40 2? ??,
    pub controller_routes: [ControllerRoutesTable; 0x10],
    /// addr: 41 ?? ??
    pub drum_setup: [[DrumSetup; 128]; 2],
}

impl RAM {
    pub fn new() -> Self {
        Self {
            system: DEFAULT_SYSTEM_PARAMS,
            effect: DEFAULT_EFFECT_PARAMS,
            parts: DEFAULT_PARTS_PARAMS,
            controller_routes: DEFAULT_CONTROLLER_ROUTES,
            drum_setup: DEFAULT_DRUM_SETUP,
        }
    }

    fn get_drum_setup(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (setup, drum_key, param) = DrumSetup::addr(addr);
        let drum_setup = self.drum_setup.get(setup as usize).ok_or(err.clone())?;
        let drum_key = drum_setup.get(drum_key as usize).ok_or(err.clone())?;

        match drum_key.get(param as usize) {
            Some(r) => Ok(*r),
            None => Err(err),
        }
    }

    fn set_drum_setup(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        let (setup, drum_key, param) = DrumSetup::addr(addr);

        let drum_setup = self.drum_setup.get_mut(setup as usize).ok_or(err.clone())?;
        let drum_key = drum_setup.get_mut(drum_key as usize).ok_or(err.clone())?;

        match drum_key.get_mut(param as usize) {
            Some(r) => *r = value,
            None => return Err(err),
        }

        Ok(())
    }
}

impl Memory for RAM {
    fn reset(&mut self) {
        self.system = DEFAULT_SYSTEM_PARAMS;
        self.effect = DEFAULT_EFFECT_PARAMS;
        self.parts = DEFAULT_PARTS_PARAMS;
        self.controller_routes = DEFAULT_CONTROLLER_ROUTES;
        self.drum_setup = DEFAULT_DRUM_SETUP;
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let err = MidiError::BadMemoryAddress { bytes: addr.into() };
        match addr[0] {
            0x40 => match addr[1] {
                0x00 => match self.system.get_mut(addr[2] as usize) {
                    Some(r) => Ok(*r = value),
                    None => Err(err),
                },
                0x01 => match self.effect.get_mut(addr[2] as usize) {
                    Some(r) => Ok(*r = value),
                    None => Err(err),
                },
                0x10..0x20 => {
                    let channel = (addr[1] & 0xF) as usize;
                    let param = match self.parts.get_mut(channel) {
                        Some(r) => r,
                        None => return Err(err.into()),
                    };

                    match param.get_mut(addr[2] as usize) {
                        Some(r) => Ok(*r = value),
                        None => Err(err.into()),
                    }
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
                0x00 => match self.system.get(addr[2] as usize) {
                    Some(r) => Ok(*r),
                    None => Err(err.clone()),
                },
                0x01 => match self.effect.get(addr[2] as usize) {
                    Some(r) => Ok(*r),
                    None => Err(err.clone()),
                },
                0x10..0x20 => {
                    let channel = (addr[1] & 0xF) as usize;
                    let param = match self.parts.get(channel) {
                        Some(r) => r,
                        None => return Err(err.clone()),
                    };
                    match param.get(addr[2] as usize) {
                        Some(r) => Ok(*r),
                        None => Err(err.clone()),
                    }
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
