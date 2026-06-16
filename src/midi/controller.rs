use std::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum GMControllers {
    Modulation = 1,
    Breath = 2,
    FootContoller = 3,
    PortamentoTime = 5,
    Volume = 7,
    Balance = 8,
    Pan = 10,
    Expression = 11,
    PedalSustain = 64,
    Portamento = 65,
    PedalSostenuto = 66,
    PedalSoft = 67,
    Hold2 = 69,
    ExternalEffects = 91,
    TremoloDepth = 92,
    ChourusDepth = 93,
    CelesteDetune = 94,
    PhaserDepth = 95,
    ResetAllControllers = 121,
}

impl fmt::Debug for GMControllers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Modulation => "Modulation",
            Self::Breath => "Breath Controller",
            Self::FootContoller => "Foot Controller",
            Self::PortamentoTime => "Portamento Time",
            Self::Volume => "Volume",
            Self::Balance => "Balance",
            Self::Pan => "Pan",
            Self::Expression => "Expression",
            Self::PedalSustain => "Sustain Pedal",
            Self::Portamento => "Portamento",
            Self::PedalSostenuto => "Sostenuto Pedal",
            Self::PedalSoft => "Soft Pedal",
            Self::Hold2 => "Hold 2",
            Self::ExternalEffects => "External Effects Depth",
            Self::TremoloDepth => "Tremolo Depth",
            Self::ChourusDepth => "Chorus Depth",
            Self::CelesteDetune => "Celeste (Detune) Depth",
            Self::PhaserDepth => "Phaser Depth",
            Self::ResetAllControllers => "Reset All Controllers",
        };
        let value: usize = (*self).into();
        write!(f, "GM::{}(id = {})", name, value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum XGController {
    BankSelectMSB = 0,
    Modulation = 1,
    PortamentoTime = 5,
    DataEntryMSB = 6,
    MasterVolume = 7,
    Panpot = 10,
    Expression = 11,
    BankSelectLSB = 32,
    DataEntryLSB = 38,
    Sustain = 64,
    Portamento = 65,
    Sostenuto = 66,
    Soft = 67,
    HarmonicContent = 71,
    ReleaseTime = 72,
    AttackTime = 73,
    Brightness = 74,
    PortamentoControl = 84,
    Reverb = 91,
    Chourus = 93,
    Variation = 94,
    RPNIncrement = 96,
    RPNDecrement = 97,
    NRPNLSB = 98,
    NRPNMSB = 99,
    RPNLSB = 100,
    RPNMSB = 101,
    AllSoundOff = 120,
    ResetAllControllers = 121,
    AllNotesOff = 123,
    OMNIOff = 124,
    OMNIOn = 125,
    Mono = 126,
    Poly = 127,
}

impl fmt::Debug for XGController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::BankSelectMSB => "Bank Select MSB",
            Self::Modulation => "Modulation",
            Self::PortamentoTime => "Portamento Time",
            Self::DataEntryMSB => "Data Entry MSB",
            Self::MasterVolume => "Master Volume",
            Self::Panpot => "Panpot",
            Self::Expression => "Expression",
            Self::BankSelectLSB => "Bank Select LSB",
            Self::DataEntryLSB => "Data Entry LSB",
            Self::Sustain => "Sustain",
            Self::Portamento => "Portamento",
            Self::Sostenuto => "Sostenuto",
            Self::Soft => "Soft",
            Self::HarmonicContent => "Harmonic Content",
            Self::ReleaseTime => "Release Time",
            Self::AttackTime => "Attack Time",
            Self::Brightness => "Brightness",
            Self::PortamentoControl => "Portamento Control",
            Self::Reverb => "Reverb",
            Self::Chourus => "Chorus",
            Self::Variation => "Variation",
            Self::RPNIncrement => "RPN Increment",
            Self::RPNDecrement => "RPN Decrement",
            Self::NRPNLSB => "NRPN LSB",
            Self::NRPNMSB => "NRPN MSB",
            Self::RPNLSB => "RPN LSB",
            Self::RPNMSB => "RPN MSB",
            Self::AllSoundOff => "All Sound Off",
            Self::ResetAllControllers => "Reset All Controllers",
            Self::AllNotesOff => "All Notes Off",
            Self::OMNIOff => "OMNI Off",
            Self::OMNIOn => "OMNI On",
            Self::Mono => "Mono",
            Self::Poly => "Poly",
        };
        let value: usize = (*self).into();
        write!(f, "XG::{}(id = {})", name, value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum GSController {
    Modulation = 1,
    PortamentoTime = 5,
    DataEntryMSB = 6,
    Volume = 7,
    Pan = 10,
    Expression = 11,
    DataEntryLSB = 38,
    Hold1 = 64,
    Portamento = 65,
    Sostenuto = 66,
    Soft = 67,
    Reverb = 91,
    Chourus = 93,
    RPNIncrement = 96,
    RPNDecrement = 97,
    NRPNLSB = 98,
    NRPNMSB = 99,
    RPNLSB = 100,
    RPNMSB = 101,
}

impl fmt::Debug for GSController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Modulation => "Modulation",
            Self::PortamentoTime => "Portamento Time",
            Self::DataEntryMSB => "Data Entry MSB",
            Self::Volume => "Volume",
            Self::Pan => "Pan",
            Self::Expression => "Expression",
            Self::DataEntryLSB => "Data Entry LSB",
            Self::Hold1 => "Hold 1",
            Self::Portamento => "Portamento",
            Self::Sostenuto => "Sostenuto",
            Self::Soft => "Soft",
            Self::Reverb => "Reverb",
            Self::Chourus => "Chorus",
            Self::RPNIncrement => "RPN Increment",
            Self::RPNDecrement => "RPN Decrement",
            Self::NRPNLSB => "NRPN LSB",
            Self::NRPNMSB => "NRPN MSB",
            Self::RPNLSB => "RPN LSB",
            Self::RPNMSB => "RPN MSB",
        };
        let value: usize = (*self).into();
        write!(f, "GS::{}(id = {})", name, value)
    }
}
