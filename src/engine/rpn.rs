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
