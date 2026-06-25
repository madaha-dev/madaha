#[derive(Debug, Clone, Copy)]
pub enum RPNType {
    PitchbendSensitivity,
    FineTuning,
    CoarseTuning,
    TuningProgSelect,
    TuningBankSelect,
}

impl From<u8> for RPNType {
    fn from(value: u8) -> Self {
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

impl Into<u8> for RPNType {
    fn into(self) -> u8 {
        match self {
            Self::PitchbendSensitivity => 0,
            Self::FineTuning => 1,
            Self::CoarseTuning => 2,
            Self::TuningBankSelect => 3,
            Self::TuningProgSelect => 4,
        }
    }
}
