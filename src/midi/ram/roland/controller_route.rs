use crate::midi::errors::MidiError;
use crate::midi::ram::MemoryAddr;
use crate::midi::ram::interface::Memory;
use super::lfo_params::LFOParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerRoute {
    pub pitch: u8,
    pub tvf_cutoff: u8,
    pub amplitude: u8,
    pub lfo: [LFOParams; 2],
}

impl ControllerRoute {
    pub const fn new() -> Self {
        Self {
            pitch: 0x40,
            tvf_cutoff: 0x40,
            amplitude: 0x40,
            lfo: [LFOParams::new(); 2],
        }
    }
}

impl From<[u8; 16]> for ControllerRoute {
    fn from(value: [u8; 16]) -> Self {
        Self {
            pitch: value[0],
            tvf_cutoff: value[1],
            amplitude: value[2],
            lfo: [
                LFOParams {
                    rate: value[3],
                    pitch_depth: value[4],
                    tvf_depth: value[5],
                    tva_depth: value[6],
                },
                LFOParams {
                    rate: value[7],
                    pitch_depth: value[8],
                    tvf_depth: value[9],
                    tva_depth: value[10],
                },
            ],
        }
    }
}

impl Into<[u8; 16]> for ControllerRoute {
    fn into(self) -> [u8; 16] {
        [
            self.pitch,
            self.tvf_cutoff,
            self.amplitude,
            self.lfo[0].rate,
            self.lfo[0].pitch_depth,
            self.lfo[0].tvf_depth,
            self.lfo[0].tva_depth,
            self.lfo[1].rate,
            self.lfo[1].pitch_depth,
            self.lfo[1].tvf_depth,
            self.lfo[1].tva_depth,
            0,
            0,
            0,
            0,
            0,
        ]
    }
}

impl std::ops::Index<usize> for ControllerRoute {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => return &self.pitch,
            1 => return &self.tvf_cutoff,
            2 => return &self.amplitude,
            3 => return &self.lfo[0].rate,
            4 => return &self.lfo[0].pitch_depth,
            5 => return &self.lfo[0].tvf_depth,
            6 => return &self.lfo[0].tva_depth,
            7 => return &self.lfo[1].rate,
            8 => return &self.lfo[1].pitch_depth,
            9 => return &self.lfo[1].tvf_depth,
            10 => return &self.lfo[1].tva_depth,
            _ => return &0x0,
        }
    }
}

impl std::ops::IndexMut<usize> for ControllerRoute {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.pitch,
            1 => &mut self.tvf_cutoff,
            2 => &mut self.amplitude,
            3 => &mut self.lfo[0].rate,
            4 => &mut self.lfo[0].pitch_depth,
            5 => &mut self.lfo[0].tvf_depth,
            6 => &mut self.lfo[0].tva_depth,
            7 => &mut self.lfo[1].rate,
            8 => &mut self.lfo[1].pitch_depth,
            9 => &mut self.lfo[1].tvf_depth,
            10 => &mut self.lfo[1].tva_depth,
            _ => panic!("ControllerRoutesTable: index {} out of bounds", index),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ControllerRoutesTable {
    pub modulation: ControllerRoute,
    pub pitchbend: ControllerRoute,
    pub channel_pressure: ControllerRoute,
    pub poly_pressure: ControllerRoute,
    pub cc_a: ControllerRoute,
    pub cc_b: ControllerRoute,
}

impl ControllerRoutesTable {
    pub const fn new() -> Self {
        Self {
            modulation: ControllerRoute::new(),
            channel_pressure: ControllerRoute::new(),
            poly_pressure: ControllerRoute::new(),
            cc_a: ControllerRoute::new(),
            cc_b: ControllerRoute::new(),
            pitchbend: ControllerRoute::new(),
        }
    }
}

impl std::ops::Index<usize> for ControllerRoutesTable {
    type Output = ControllerRoute;
    fn index(&self, index: usize) -> &Self::Output {
        let data = match index {
            0 => &self.modulation,
            1 | 0x10 => &self.pitchbend,
            2 | 0x20 => &self.channel_pressure,
            3 | 0x30 => &self.poly_pressure,
            4 | 0x40 => &self.cc_a,
            5 | 0x50 => &self.cc_b,

            _ => panic!("ControllerRoutesTable: index {} out of bounds", index),
        };
        data
    }
}

impl std::ops::IndexMut<usize> for ControllerRoutesTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let data = match index {
            0 => &mut self.modulation,
            1 | 0x10 => &mut self.pitchbend,
            2 | 0x20 => &mut self.channel_pressure,
            3 | 0x30 => &mut self.poly_pressure,
            4 | 0x40 => &mut self.cc_a,
            5 | 0x50 => &mut self.cc_b,

            _ => panic!("ControllerRoutesTable: index {} out of bounds", index),
        };
        data
    }
}

impl Memory for ControllerRoutesTable {
    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let addr = addr[3] as usize;
        let base_addr = addr & 0xF0;
        let type_addr = addr & 0x0F;
        Ok(self[base_addr][type_addr])
    }

    fn reset(&mut self) {
        self.modulation = ControllerRoute::new();
        self.channel_pressure = ControllerRoute::new();
        self.poly_pressure = ControllerRoute::new();
        self.cc_a = ControllerRoute::new();
        self.cc_b = ControllerRoute::new();
        self.pitchbend = ControllerRoute::new();
    }

    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let addr = addr[3] as usize;
        let base_addr = addr & 0xF0;
        let type_addr = addr & 0x0F;
        self[base_addr][type_addr] = value;
        Ok(())
    }
}