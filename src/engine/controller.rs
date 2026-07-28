use crate::engine::{
    data_entry::DataEntrySelect,
    errors::MidiError,
    ram::{MemoryAddr, RAM, interface::Memory},
};

#[derive(Debug, Copy, Clone)]
pub struct Controller {
    pub _channel: usize,
    // CC#1
    pub modulation: u8,
    // CC11
    pub expression: u8,
    // CC#64, or Hold1
    pub sustain: u8,
    // CC#66
    pub sostenuto: u8,
    // CC#67
    pub soft_pedal: u8,
    // CC#84
    pub potamento_control: u8,
    // CC#101
    pub rpn_id_msb: u8,
    // CC#100
    pub rpn_id_lsb: u8,

    // CC#99
    pub nrpn_id_msb: u8,
    // CC#98
    pub nrpn_id_lsb: u8,
}

impl Controller {
    pub const fn new() -> Self {
        Self {
            _channel: 0xFF,

            modulation: 0,
            sustain: 0,
            expression: 0x7F,
            sostenuto: 0,
            soft_pedal: 0,
            potamento_control: 0,
            rpn_id_lsb: 0x7F,
            rpn_id_msb: 0x7F,
            nrpn_id_lsb: 0x7F,
            nrpn_id_msb: 0x7F,
        }
    }

    pub fn get(&self, ram: &RAM, cc: u8) -> Option<u8> {
        let addr = |lo: u8| MemoryAddr::new(0x08, self._channel as u8, lo);
        let ram_get = |lo: u8| ram.get(addr(lo)).ok();

        match cc {
            // 0=0-Bank Select MSB
            0 => ram_get(0x01),
            // 1=1-Modulation
            1 => Some(self.modulation),
            // 5=5-Portamento Time
            5 => ram_get(0x68),
            // 6=6-Data Entry MSB - skip (handled by RPN/NRPN logic)
            // 7=7-Master Volume
            7 => ram_get(0x0B),
            // 10=10-Panpot
            10 => ram_get(0x0E),
            // 11=11-Expression
            11 => Some(self.expression),
            // 32=32-Bank Select LSB
            32 => ram_get(0x02),
            // 38=38-Data Entry LSB - skip
            // 64=64-Sustain
            64 => Some(self.sustain),
            // 65=65-Portamento
            65 => ram_get(0x67),
            // 66=66-Sostenuto
            66 => Some(self.sostenuto),
            // 67=67-Soft Pedal
            67 => Some(self.soft_pedal),
            // 71=71-Harmonic Content
            71 => ram_get(0x19),
            // 72=72-Release Time
            72 => ram_get(0x1C),
            // 73=73-Attack Time
            73 => ram_get(0x1A),
            // 74=74-Brightness
            74 => ram_get(0x18),
            75 => ram_get(0x1B),
            // 84=84-Portamento Control
            84 => Some(self.potamento_control),
            // 91=91-Effects Send Level 1 (reverb)
            91 => ram_get(0x13),
            // 93=93-Effects Send Level 3 (chorus)
            93 => ram_get(0x12),
            // 94=94-Effects Send Level 4 (variation)
            94 => ram_get(0x14),
            // 96=96-RPN Increment - skip
            // 97=97-RPN Decrement - skip
            // 98=98-NRPN LSB
            98 => Some(self.nrpn_id_lsb),
            // 99=99-NRPN MSB
            99 => Some(self.nrpn_id_msb),
            // 100=100-RPN LSB
            100 => Some(self.rpn_id_lsb),
            // 101=101-RPN MSB
            101 => Some(self.rpn_id_msb),
            // 120=120-All Sound Off - skip (handled in engine)
            // 121=121-Reset All Controllers - skip (handled in engine)
            // 123=123-All Notes Off - skip (handled in engine)
            // 124=124-OMNI Off - skip
            // 125=125-OMNI On - skip
            // 126=126-Mono - skip
            // 127=127-Poly - skip
            _ => None,
        }
    }

    pub fn set(
        &mut self,
        ram: &mut RAM,
        cc: u8,
        value: u8,
    ) -> Result<ControllerCallback, MidiError> {
        if ram.xg.multi_part[self._channel]
            .rcv_switches
            .rcv_control_change
            == 0
        {
            return Ok(ControllerCallback::None);
        }

        let rcv_moduration = ram.xg.multi_part[self._channel].rcv_switches.rcv_moduration != 0;
        let rcv_volume = ram.xg.multi_part[self._channel].rcv_switches.rcv_volume != 0;
        let rcv_pan = ram.xg.multi_part[self._channel].rcv_switches.rcv_pan != 0;
        let rcv_expression = ram.xg.multi_part[self._channel].rcv_switches.rcv_expression != 0;
        let rcv_sustain = ram.xg.multi_part[self._channel].rcv_switches.rcv_hold1 != 0;
        let rcv_portamento = ram.xg.multi_part[self._channel].rcv_switches.rcv_portamento != 0;
        let rcv_sostenuto = ram.xg.multi_part[self._channel].rcv_switches.rcv_sostenuto != 0;
        let rcv_soft_pedal = ram.xg.multi_part[self._channel].rcv_switches.rcv_soft_pedal != 0;
        let rcv_bank_select = ram.xg.multi_part[self._channel]
            .rcv_switches
            .rcv_bank_select
            != 0;

        let value = value & 0x7F;
        let addr = |lo: u8| MemoryAddr::new(0x08, self._channel as u8, lo);
        let ram_set = |lo: u8| Ok(ControllerCallback::RAMChange(addr(lo), value));

        match cc {
            // 0=0-Bank Select MSB
            0 => rcv_bank_select.then(|| ram_set(0x01)).unwrap(),
            // 1=1-Modulation
            1 => {
                rcv_moduration.then(|| self.modulation = value);
                Ok(ControllerCallback::None)
            }
            // 5=5-Portamento Time
            5 => rcv_portamento.then(|| ram_set(0x68)).unwrap(),
            // 6=6-Data Entry MSB - skip (handled by RPN/NRPN logic)
            6 => Ok(ControllerCallback::EntryMSBChange),
            // 7=7-Master Volume
            7 => rcv_volume.then(|| ram_set(0x0B)).unwrap(),
            // 10=10-Panpot
            10 => rcv_pan.then(|| ram_set(0x0E)).unwrap(),
            // 11=11-Expression
            11 => {
                rcv_expression.then(|| self.expression = value);
                Ok(ControllerCallback::None)
            }
            // 32=32-Bank Select LSB
            32 => rcv_bank_select.then(|| ram_set(0x02)).unwrap(),
            // 38=38-Data Entry LSB - skip
            38 => Ok(ControllerCallback::EntryLSBChange),
            // 64=64-Sustain
            64 => {
                rcv_sustain.then(|| self.sustain = value);
                Ok(ControllerCallback::None)
            }
            // 65=65-Portamento
            65 => rcv_portamento.then(|| ram_set(0x67)).unwrap(),
            // 66=66-Sostenuto
            66 => {
                rcv_sostenuto.then(|| self.sostenuto = value);
                Ok(ControllerCallback::None)
            }
            // 67=67-Soft Pedal
            67 => {
                rcv_soft_pedal.then(|| self.soft_pedal = value);
                Ok(ControllerCallback::None)
            }
            // 71=71-Harmonic Content
            71 => ram_set(0x19),
            // 72=72-Release Time
            72 => ram_set(0x1C),
            // 73=73-Attack Time
            73 => ram_set(0x1A),
            // 74=74-Brightness
            74 => ram_set(0x18),
            75 => ram_set(0x1B),
            // 84=84-Portamento Control
            84 => {
                self.potamento_control = value;
                Ok(ControllerCallback::None)
            }
            // 91=91-Effects Send Level 1 (reverb)
            91 => ram_set(0x13),
            // 93=93-Effects Send Level 3 (chorus)
            93 => ram_set(0x12),
            // 94=94-Effects Send Level 4 (variation)
            94 => ram_set(0x14),
            // 96=96-RPN Increment - skip
            96 => Ok(ControllerCallback::RPNChange(1)),
            // 97=97-RPN Decrement - skip
            97 => Ok(ControllerCallback::RPNChange(-1)),
            // 98=98-NRPN LSB
            98 => {
                self.nrpn_id_lsb = value;
                Ok(ControllerCallback::DataEntrySelectChange(
                    DataEntrySelect::NRPN,
                ))
            }
            // 99=99-NRPN MSB
            99 => {
                self.nrpn_id_msb = value;
                Ok(ControllerCallback::DataEntrySelectChange(
                    DataEntrySelect::NRPN,
                ))
            }
            // 100=100-RPN LSB
            100 => {
                self.rpn_id_lsb = value;
                Ok(ControllerCallback::DataEntrySelectChange(
                    DataEntrySelect::RPN,
                ))
            }
            // 101=101-RPN MSB
            101 => {
                self.rpn_id_msb = value;
                Ok(ControllerCallback::DataEntrySelectChange(
                    DataEntrySelect::RPN,
                ))
            }
            // 120=120-All Sound Off - skip (handled in engine)
            // 121=121-Reset All Controllers - skip (handled in engine)
            // 123=123-All Notes Off - skip (handled in engine)
            // 124=124-OMNI Off - skip
            // 125=125-OMNI On - skip
            // 126=126-Mono - skip
            // 127=127-Poly - skip
            _ => Err(MidiError::UnknownController { cc }),
        }
    }
}

pub enum ControllerCallback {
    EntryLSBChange,
    EntryMSBChange,
    DataEntrySelectChange(DataEntrySelect),
    RPNChange(i8),
    RAMChange(MemoryAddr, u8),
    None,
}
