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
}

impl Channel {
    pub fn new() -> Self {
        Self {
            bank: 0,
            program: 0,
            pitchbend: PITCH_BEND_MIDDLE,
            controllers: DEFAULT_CONTROLLER_VALUES,
            rpns: DEFAULT_RPN_VALUES,
        }
    }

    pub fn reset(&mut self) {
        self.pitchbend = PITCH_BEND_MIDDLE;
        self.program = 0;
        self.bank = 0;
        self.controllers = DEFAULT_CONTROLLER_VALUES;
        self.rpns[RPN::PitchbendSensitivity as usize] = 2;
        self.rpns[RPN::FineTuning as usize] = DEFAULT_FINE_TUNING;
        self.rpns[RPN::CoarseTuning as usize] = DEFAULT_COARSE_TUNING;
    }

    // cc event 121
    pub fn reset_controller(&mut self) {
        let volume = self.controllers[GMControllers::Volume as usize];
        let pan = self.controllers[GMControllers::Pan as usize];
        self.controllers = DEFAULT_CONTROLLER_VALUES;
        self.controllers[GMControllers::Volume as usize] = volume;
        self.controllers[GMControllers::Pan as usize] = pan;

        for (i, value) in DEFAULT_CONTROLLER_VALUES.iter().enumerate() {
            if i == GMControllers::Volume as usize || i == GMControllers::Pan as usize {
                continue;
            }
            self.controllers[i] = *value;
        }

        self.pitchbend = PITCH_BEND_MIDDLE;
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
