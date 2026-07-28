use crate::engine::channel::Channel;
use crate::engine::errors::MidiError;
use crate::engine::nrpn;
use crate::engine::ram::{MemoryAddr, RAM};
use crate::engine::rpn::RPNType;

pub fn data_entry_handler_msb(
    channel: &mut Channel,
    ram: &mut RAM,
    value: u8,
) -> Result<MemoryAddr, MidiError> {
    let rcv_rpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_rpn != 0;
    let rcv_nrpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_nrpn != 0;
    match channel.data_entry_select {
        DataEntrySelect::None => Ok(MemoryAddr::new(0xFF, 0xFF, 0xFF)),
        DataEntrySelect::RPN => rcv_rpn
            .then(|| {
                match RPNType::from((channel.controller.rpn_id_msb, channel.controller.rpn_id_lsb))
                {
                    RPNType::PitchbendSensitivity => channel.pitchbend_sensitivity = value,
                    RPNType::FineTuning => channel.fine_msb = value,
                    RPNType::CoarseTuning => channel.coarse = value,
                    RPNType::TuningBankSelect => channel.tuning_bank_select = value,
                    RPNType::TuningProgSelect => channel.tuning_prog_select = value,
                }
                Ok(MemoryAddr::new(0xFF, 0xFF, 0xFF))
            })
            .unwrap(),
        DataEntrySelect::NRPN => rcv_nrpn
            .then(|| {
                match nrpn::nrpn_to_addr(
                    ram,
                    channel._channel as u8,
                    channel.controller.nrpn_id_msb,
                    channel.controller.nrpn_id_lsb,
                ) {
                    Some(addr) => Ok(addr),
                    None => Err(MidiError::UnknownNRPN {
                        msb: channel.controller.nrpn_id_msb,
                        lsb: channel.controller.nrpn_id_lsb,
                    }),
                }
            })
            .unwrap(),
    }
}

pub fn data_entry_handler_lsb(
    channel: &mut Channel,
    ram: &mut RAM,
    value: u8,
) -> Result<(), MidiError> {
    let rcv_rpn = ram.xg.multi_part[channel._channel].rcv_switches.rcv_rpn != 0;
    match channel.data_entry_select {
        DataEntrySelect::None => Ok(()),
        DataEntrySelect::RPN => rcv_rpn
            .then(|| {
                match RPNType::from((channel.controller.rpn_id_msb, channel.controller.rpn_id_lsb))
                {
                    RPNType::PitchbendSensitivity => Ok(channel.pitchbend_cents = value),
                    RPNType::FineTuning => Ok(channel.fine_lsb = value),
                    _ => Ok(()),
                }
            })
            .unwrap(),
        DataEntrySelect::NRPN => Ok(()),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DataEntrySelect {
    None,
    RPN,
    NRPN,
}
