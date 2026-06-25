use crate::engine::channel::Channel;
use crate::engine::ram::RAM;
use crate::{get_14bit, get_lsb, get_msb};

#[derive(Debug, Clone, Copy)]
pub enum RPNType {
    PitchbendSensitivity,
    FineTuning,
    CoarseTuning,
    TuningProgSelect,
    TuningBankSelect,
}

impl From<u16> for RPNType {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::PitchbendSensitivity,
            1 => Self::FineTuning,
            2 => Self::CoarseTuning,
            3 => Self::TuningBankSelect,
            4 => Self::TuningProgSelect,

            _ => panic!("RPNType: index {} out of bounds", value),
        }
    }
}

impl From<(u8, u8)> for RPNType {
    fn from(value: (u8, u8)) -> Self {
        let (msb, lsb) = value;
        match msb {
            0 => match lsb {
                0 => Self::PitchbendSensitivity,
                1 => Self::FineTuning,
                2 => Self::CoarseTuning,
                3 => Self::TuningBankSelect,
                4 => Self::TuningProgSelect,

                _ => panic!("RPNType: index ({:?}) out of bounds", value),
            },
            _ => panic!("RPNType: index ({:?}) out of bounds", value),
        }
    }
}

impl Into<u16> for RPNType {
    fn into(self) -> u16 {
        match self {
            Self::PitchbendSensitivity => 0,
            Self::FineTuning => 1,
            Self::CoarseTuning => 2,
            Self::TuningBankSelect => 3,
            Self::TuningProgSelect => 4,
        }
    }
}

impl Into<(u8, u8)> for RPNType {
    fn into(self) -> (u8, u8) {
        match self {
            Self::PitchbendSensitivity => (0, 0),
            Self::FineTuning => (0, 1),
            Self::CoarseTuning => (0, 2),
            Self::TuningBankSelect => (0, 3),
            Self::TuningProgSelect => (0, 4),
        }
    }
}

pub fn rpn_data_change(channel: &mut Channel, ram: &RAM, u: i8) {
    let rcv_rpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_rpn != 0;
    let rpn_type = RPNType::from((channel.controller.rpn_id_msb, channel.controller.rpn_id_lsb));
    rcv_rpn.then(|| match rpn_type {
        RPNType::PitchbendSensitivity => {
            if channel.pitchbend_sensitivity >= 0x7F {
                return;
            }
            channel.pitchbend_sensitivity = if u > 0 {
                channel.pitchbend_sensitivity.wrapping_add(1)
            } else if u < 0 {
                channel.pitchbend_cents.wrapping_sub(1)
            } else {
                channel.pitchbend_sensitivity
            };
        }
        RPNType::FineTuning => {
            let mut data = get_14bit!(channel.fine_msb, channel.fine_lsb);
            if data >= 0x3FFF {
                return;
            }
            data = if u > 0 {
                data.wrapping_add(1)
            } else if u < 0 {
                data.wrapping_sub(1)
            } else {
                data
            };
            channel.fine_msb = get_msb!(data);
            channel.fine_lsb = get_lsb!(data);
        }
        RPNType::CoarseTuning => {
            if channel.pitchbend_sensitivity >= 0x7F {
                return;
            }
            channel.coarse = if u > 0 {
                channel.coarse.wrapping_add(1)
            } else if u < 0 {
                channel.coarse.wrapping_sub(1)
            } else {
                channel.coarse
            };
        }
        RPNType::TuningBankSelect => {
            if channel.pitchbend_sensitivity >= 0x7F {
                return;
            }
            channel.tuning_bank_select = if u > 0 {
                channel.tuning_bank_select.wrapping_add(1)
            } else if u < 0 {
                channel.tuning_bank_select.wrapping_sub(1)
            } else {
                channel.tuning_bank_select
            };
        }
        RPNType::TuningProgSelect => {
            if channel.pitchbend_sensitivity >= 0x7F {
                return;
            }
            channel.tuning_prog_select = if u > 0 {
                channel.tuning_prog_select.wrapping_add(1)
            } else if u < 0 {
                channel.tuning_prog_select.wrapping_sub(1)
            } else {
                channel.tuning_prog_select
            };
        }
    });
}
