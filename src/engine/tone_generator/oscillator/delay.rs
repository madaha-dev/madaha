#[derive(Debug)]
enum DelayState {
    Delay,
    Fade,
    Full,
}

#[derive(Debug)]
pub struct Delay {
    pub delay_samples: u32,
    pub fade_samples: u32,
    pub fade_step: f32,

    state: DelayState,
    counter: u32,
}

impl Delay {
    pub fn new() -> Self {
        Self {
            delay_samples: 0,
            fade_samples: 0,
            fade_step: 1.0,
            state: DelayState::Delay,
            counter: 0,
        }
    }
    pub fn delay(&mut self, lfo_input: f32) -> f32 {
        
        match self.state {
            DelayState::Delay => {
                self.counter += 1;
                if self.counter >= self.delay_samples {
                    self.counter = 0;
                    self.state = if self.fade_samples > 0 {
                        DelayState::Fade
                    } else {
                        DelayState::Full
                    }
                }
                0.0
            }
            DelayState::Fade => {
                self.counter += 1;
                let mut eg = (self.counter as f32) * self.fade_step;
                if eg >= 1.0 {
                    eg = 1.0;
                    self.state = DelayState::Full;
                }
                lfo_input * eg
            }
            DelayState::Full => lfo_input,
        }
    }

    pub fn reset(&mut self) {
        self.counter = 0;
        self.state = DelayState::Delay;
    }
}
