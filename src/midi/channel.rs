use crate::midi::{
    consts::{CONTROLLER_PAN_MIDDLE, DEFAULT_COARSE_TUNING, DEFAULT_FINE_TUNING},
    controller::GMControllers,
    rpn::RPN,
};

use super::consts::PITCH_BEND_MIDDLE;

#[derive(Debug, Copy, Clone)]
pub struct Channel {
    pub bank: u8,
    pub program: u8,
    pub pitchbend: u16,

    pub controllers: [u8; 128],
    pub rpns: [u16; 128],
    pub nrpns: [u16; 0x4000],

    pub aftertouch: u8,
    pub mute: bool,
    pub solo: bool,
    pub active_notes: u8,
    pub last_note: u8,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            bank: 0,
            program: 0,
            pitchbend: PITCH_BEND_MIDDLE,
            controllers: DEFAULT_CONTROLLER_VALUES,
            rpns: DEFAULT_RPN_VALUES,
            nrpns: DEFAULT_NRPN_VALUES,
            aftertouch: 0,
            mute: false,
            solo: false,
            active_notes: 0,
            last_note: 0,
        }
    }

    // on sysex
    pub fn reset(&mut self) {
        self.pitchbend = PITCH_BEND_MIDDLE;
        self.program = 0;
        self.bank = 0;
        self.controllers = DEFAULT_CONTROLLER_VALUES;
        self.reset_rpn();
    }

    // cc event 121
    pub fn reset_controller(&mut self) {
        let volume = self.controllers[GMControllers::Volume as usize];
        let pan = self.controllers[GMControllers::Pan as usize];
        self.controllers = DEFAULT_CONTROLLER_VALUES;
        self.controllers[GMControllers::Volume as usize] = volume;
        self.controllers[GMControllers::Pan as usize] = pan;

        self.pitchbend = PITCH_BEND_MIDDLE;
        self.reset_rpn();
    }

    fn reset_rpn(&mut self) {
        self.rpns[RPN::PitchbendSensitivity as usize] = 2;
        self.rpns[RPN::FineTuning as usize] = DEFAULT_FINE_TUNING;
        self.rpns[RPN::CoarseTuning as usize] = DEFAULT_COARSE_TUNING;
    }
}

const DEFAULT_CONTROLLER_VALUES: [u8; 128] = {
    let mut data = [0; 128];
    data[GMControllers::Volume as usize] = 100;
    data[GMControllers::Pan as usize] = CONTROLLER_PAN_MIDDLE;
    data[GMControllers::Expression as usize] = 127;
    data[GMControllers::ExternalEffects as usize] = 40;

    data
};

const DEFAULT_RPN_VALUES: [u16; 128] = {
    let mut data = [0; 128];
    data[RPN::PitchbendSensitivity as usize] = 2;
    data[RPN::FineTuning as usize] = DEFAULT_FINE_TUNING;
    data[RPN::CoarseTuning as usize] = DEFAULT_COARSE_TUNING;

    data
};

const DEFAULT_NRPN_VALUES: [u16; 0x4000] = [0; 0x4000];
