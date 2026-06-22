use super::effects::EffectData;
use crate::midi::{
    consts::DRUM_CHANNEL_ID,
    effects::{
        chorus_type::XGChorusType,
        default_data::{xg_chorus_data, xg_reverb_data, xg_variation_data},
        reverb_type::XGReverbType,
        variation_type::XGVariationType,
    },
    errors::MidiError,
    ram::{
        MemoryAddr,
        interface::Memory,
        yamaha::{drum_setup::DrumSetup, effects::interface::EffectRAM},
    },
};

pub type DisplayBitmap = [[[u8; 0x30]; 0x8]; 0x10];
pub type MultiPart = [[u8; 128]; 16];

/// XG hardware memory emulate
/// but we never response bulk dump 23333
#[derive(Debug)]
pub struct RAM {
    system: [u8; 128],             // SysEx 00 00 ??
    effect: [EffectData; 16],      // SysEx 02 ?? ??
    display_letter: [u8; 0x20],    // SysEx 06 00 ??, text display
    display_bitmap: DisplayBitmap, // SysEx 07 ?? ??, bitmap display
    multi_part: MultiPart,         // SysEx 08 ?? ??
    drum_setup: DrumSetup,         // SysEx 3n ?? ??
}

impl Memory for RAM {
    fn set(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        match addr[0] {
            0x00 => return self.set_system(addr, value),
            0x02 => return self.set_effect(addr, value),
            0x06 => return self.set_text(addr, value),
            0x07 => return self.set_display(addr, value),
            0x08 => return self.set_multipart(addr, value),
            0x30..0x3F => return self.set_drumsetup(addr, value),

            _ => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
    }

    fn get(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        match addr[0] {
            0x00 => return self.get_system(addr),
            0x02 => return self.get_effect(addr),
            0x08 => return self.get_multipart(addr),
            0x30..0x3F => return self.get_drumsetup(addr),

            _ => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
    }

    fn reset(&mut self) {
        self.system = DEFAULT_SYSTEM_RAM;
        self.effect = default_effect_ram();
        self.multi_part = DEFAULT_MULTI_PART_RAM;
    }
}

impl RAM {
    pub fn new() -> RAM {
        Self {
            system: DEFAULT_SYSTEM_RAM,
            effect: default_effect_ram(),
            display_letter: [0; 0x20],
            display_bitmap: [[[0; 0x30]; 0x8]; 0x10],
            multi_part: DEFAULT_MULTI_PART_RAM,
            drum_setup: DrumSetup::new(),
        }
    }

    fn set_system(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let addr_l = addr[2] as usize;
        match self.system.get_mut(addr_l) {
            Some(r) => *r = value,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
        Ok(())
    }

    fn set_effect(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let addr_m = addr[1] as usize;
        let effect_table = match self.effect.get_mut(addr_m) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
        effect_table.set(addr, value)
    }

    fn set_text(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let addr_l = addr[2] as usize;
        match self.display_letter.get_mut(addr_l) {
            Some(r) => *r = value,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
        Ok(())
    }

    fn set_display(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let addr_m = addr[1] as usize;
        let h = addr_m & 0xF;
        let v = (addr_m & 0xF0) >> 4;
        let px = match self.display_bitmap.get_mut(h) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
        let py = match px.get_mut(v) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };

        let addr_l = addr[2] as usize;
        match py.get_mut(addr_l) {
            Some(r) => *r = value,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
        Ok(())
    }

    fn set_multipart(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let channel = addr[1] as usize;
        let parameter_table = match self.multi_part.get_mut(channel) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
        let index = addr[2] as usize;
        match parameter_table.get_mut(index) {
            Some(r) => *r = value,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
        Ok(())
    }

    fn set_drumsetup(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        self.drum_setup.set_addr(addr, value)
    }

    fn get_system(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let addr_l = addr[2] as usize;
        match self.system.get(addr_l) {
            Some(r) => return Ok(*r),
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
    }

    fn get_effect(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let addr_m = addr[1] as usize;
        let effect_table = match self.effect.get(addr_m) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
        
        effect_table.get(addr)
    }

    fn get_multipart(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let channel = addr[1] as usize;
        let parameter_table = match self.multi_part.get(channel) {
            Some(r) => r,
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        };
        let index = addr[2] as usize;
        match parameter_table.get(index) {
            Some(r) => Ok(*r),
            None => return Err(MidiError::BadMemoryAddress { bytes: addr.into() }),
        }
    }

    fn get_drumsetup(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        self.drum_setup.get_addr(addr)
    }
}

const DEFAULT_SYSTEM_RAM: [u8; 128] = {
    let mut data = [0u8; 128];
    data[0x01] = 0x40;
    data[0x04] = 0x7F;
    data[0x06] = 0x40;

    data
};

fn default_effect_ram() -> [EffectData; 16] {
    [EffectData::new(); 16]
}

const DEFAULT_MULTI_PART_RAM: MultiPart = {
    let mut data = [[0u8; 128]; 16];
    let mut channel = 0;
    while channel < 16 {
        // ELEMENT RESERVE
        data[channel][0x00] = if channel == DRUM_CHANNEL_ID { 0 } else { 2 };
        // BANK SELECT MSB - CC#00
        data[channel][0x01] = if channel == DRUM_CHANNEL_ID { 0x7F } else { 0 };
        // Rcv CHANNEL
        data[channel][0x04] = channel as u8;
        // MONO/POLY MODE, MONO = 0, POLY = 1
        data[channel][0x05] = 1;
        // SAME NOTE NOMBER
        // KEY ON ASSIGN, SINGLE = 0, MULTY = 1, INST = 2(for DRUM)
        data[channel][0x06] = if channel == DRUM_CHANNEL_ID { 2 } else { 0 };
        // PART MODE, NORMAL = 0, DRUM = 1, DRUMS1-4 = 2-5
        data[channel][0x07] = if channel == DRUM_CHANNEL_ID { 2 } else { 0 };
        // NOTE SHIFT, -24 ~ +24 semitones
        data[channel][0x08] = 0x40;
        // DETUNE
        data[channel][0x09] = 0x08;
        // VOLUME
        data[channel][0x0B] = 0x64;
        // VELOCITY SENSE DEPTH
        data[channel][0x0C] = 0x40;
        // VELOCITY SENSE OFFSET
        data[channel][0x0D] = 0x40;
        // PAN, random = 0, L63 = 1, C = 64, R63 = 127
        data[channel][0x0E] = 0x40;
        // NOTE_LIMIT_HIGH
        data[channel][0x10] = 0x7F;
        // DRY_LEVEL
        data[channel][0x11] = 0x7F;
        // CHORUS SEND
        data[channel][0x12] = 0x00;
        // REVERB SEND
        data[channel][0x13] = 0x28;
        // VARIATION SEND
        data[channel][0x14] = 0x00;
        // VIBRATO RATE
        data[channel][0x15] = 0x40;
        // VIBRATO DEPTH
        data[channel][0x16] = if channel == DRUM_CHANNEL_ID { 0 } else { 0x40 };
        // VIBRATO DELAY
        data[channel][0x17] = if channel == DRUM_CHANNEL_ID { 0 } else { 0x40 };
        // FILTER CUTOFF FREQ
        data[channel][0x18] = 0x40;
        // FILTER RESONANCE
        data[channel][0x19] = 0x40;
        // EG ATTACK TIME
        data[channel][0x1A] = 0x40;
        // EG DECAY TIME
        data[channel][0x1B] = 0x40;
        // EG RELEASE TIME
        data[channel][0x1C] = 0x40;
        // MW PITCH CONTROL
        data[channel][0x1D] = 0x40;
        // MW FILTER CONTROL
        data[channel][0x1E] = 0x40;
        // MW AMPLITUDE CONTROL
        data[channel][0x1F] = 0x40;
        // MW LFO PMOD DEPTH
        data[channel][0x20] = 0x0A;
        // BEND PITCH CONTROL
        data[channel][0x23] = 0x42;
        // BEND FILTER CONTROL
        data[channel][0x24] = 0x40;
        // BEND AMPLITUDE CONTROL
        data[channel][0x25] = 0x40;
        // BEND LFO PMOD DEPTH
        data[channel][0x26] = 0x40;
        // BEND LFO FMOD DEPTH
        data[channel][0x27] = 0x40;
        // BEND LFO AMOD DEPTH
        data[channel][0x28] = 0x40;
        // Rcv PITCH BEND, ON = 1, OFF = 0
        data[channel][0x30] = 1;
        // Rcv CH AFTER TOUCH(CAT)
        data[channel][0x31] = 1;
        // Rcv PROGRAM CHANGE
        data[channel][0x32] = 1;
        // Rcv CONTROL CHANGE
        data[channel][0x33] = 1;
        // Rcv POLY AFTER TOUCH(PAT)
        data[channel][0x34] = 1;
        // Rcv NOTE MESSAGE
        data[channel][0x35] = 1;
        // Rcv RPN
        data[channel][0x36] = 1;
        // Rcv NRPN, XG = 1, GM = 0, default GM Mode
        data[channel][0x37] = 0;
        // Rcv MODURATION
        data[channel][0x38] = 1;
        // Rcv VOLUME
        data[channel][0x39] = 1;
        // Rcv PAN
        data[channel][0x3A] = 1;
        // Rcv EXPRESSION
        data[channel][0x3B] = 1;
        // Rcv HOLD1
        data[channel][0x3C] = 1;
        // Rcv PORTAMENTO
        data[channel][0x3D] = 1;
        // Rcv SOSTENUTO
        data[channel][0x3E] = 1;
        // Rcv SOFT PEDAL
        data[channel][0x3F] = 1;
        // Rcv BANK SELECT, XG = 1, GM = 0, default GM Mode
        data[channel][0x40] = 0;

        // SCALE TUNING
        let mut note = 0x41;
        while note <= 0x4C {
            data[channel][note] = 0x40;
            note += 1;
        }

        // CAT PITCH CONTROL, -24 ~ +24[semitones]
        data[channel][0x4D] = 0x40;
        // CAT FILTER CONTROL, -9600 ~ +9450[cent]
        data[channel][0x4E] = 0x40;
        // CAT AMPLITUDE CONTROL, -64 ~ +63
        data[channel][0x4F] = 0x40;
        // CAT LFO PMOD DEPTH
        data[channel][0x50] = 0x00;
        // CAT LFO FMOD DEPTH
        data[channel][0x51] = 0x00;
        // CAT LFO AMOD DEPTH
        data[channel][0x52] = 0x00;

        // PAT PITCH CONTROL, -24 ~ +24[semitones]
        data[channel][0x53] = 0x40;
        // PAT FILTER CONTROL, -9600 ~ +9450[cent]
        data[channel][0x54] = 0x40;
        // PAT AMPLITUDE CONTROL, -64 ~ +63
        data[channel][0x55] = 0x40;
        // PAT LFO PMOD DEPTH
        data[channel][0x56] = 0x00;
        // PAT LFO FMOD DEPTH
        data[channel][0x57] = 0x00;
        // PAT LFO AMOD DEPTH
        data[channel][0x58] = 0x00;
        // AC1 CONTROLLER NUMBER
        data[channel][0x59] = 0x10;
        // AC1 PITCH CONTROL, -24 ~ +24[semitones]
        data[channel][0x5A] = 0x40;
        // AC1 FILTER CONTROL, -9600 ~ +9450[cent]
        data[channel][0x5B] = 0x40;
        // AC1 AMPLITUDE CONTROL, -64 ~ +63
        data[channel][0x5C] = 0x40;
        // AC1 LFO PMOD DEPTH
        data[channel][0x5D] = 0x00;
        // AC1 LFO FMOD DEPTH
        data[channel][0x5E] = 0x00;
        // AC1 LFO AMOD DEPTH
        data[channel][0x5F] = 0x00;
        // AC2 CONTROLLER NUMBER
        data[channel][0x60] = 0x11;
        // AC2 PITCH CONTROL, -24 ~ +24[semitones]
        data[channel][0x61] = 0x40;
        // AC2 FILTER CONTROL, -9600 ~ +9450[cent]
        data[channel][0x62] = 0x40;
        // AC2 AMPLITUDE CONTROL, -64 ~ +63
        data[channel][0x63] = 0x40;
        // AC2 LFO PMOD DEPTH
        data[channel][0x64] = 0x00;
        // AC2 LFO FMOD DEPTH
        data[channel][0x65] = 0x00;
        // AC2 LFO AMOD DEPTH
        data[channel][0x66] = 0x00;
        // PORTAMENTO SWITCH, OFF = 0, ON = 1
        data[channel][0x67] = 0x00;
        // PORTAMENTO TIME
        data[channel][0x68] = 0x00;
        // PITCH EG INITIAL LEVEL, -64 ~ +63
        data[channel][0x69] = 0x40;
        // PITCH EG ATTACK TIME, -64 ~ +63
        data[channel][0x6A] = 0x40;
        // PITCH EG RELEASE LEVEL, -64 ~ +63
        data[channel][0x6B] = 0x40;
        // PITCH EG RELEASE TIME, -64 ~ +63
        data[channel][0x6C] = 0x40;
        // VELOCITY LIMIT LOW
        data[channel][0x6D] = 0x01;
        // VELOCITY LIMIT HIGH
        data[channel][0x6E] = 0x7F;

        channel += 1;
    }

    data
};

#[derive(Debug)]
pub struct DrumSetupTable([[DrumSetup; 74]; 16]);

impl DrumSetupTable {
    pub fn new(data: [[[u8; 16]; 74]; 16]) -> Self {
        let data: [[[u8; 16]; 74]; 16] = {
            for setup in 0..16 {
                for note in 0..74 {
                    DrumSetup::from(data[setup][note])
                }
            }
        };
        // TODO: Load drum set from tbl file

        Self(data)
    }
    pub fn get_addr(&self, addr: MemoryAddr) -> Result<u8, MidiError> {
        let setup = (addr[0] & 0x0F) as usize;
        let note = (addr[1] as usize).wrapping_sub(0x0D);
        let param = addr[2] as usize;

        if (addr[0] & 0x30) == 0 || setup >= 16 || note >= 74 || param >= 16 {
            return Err(MidiError::BadMemoryAddress { bytes: addr.into() });
        }

        Ok(self.0[setup][note][param])
    }

    pub fn set_addr(&mut self, addr: MemoryAddr, value: u8) -> Result<(), MidiError> {
        let setup = (addr[0] & 0x0F) as usize;
        let note = (addr[1] as usize).wrapping_sub(0x0D);
        let param = addr[2] as usize;

        if (addr[0] & 0x30) == 0 || setup >= 16 || note >= 74 || param >= 16 {
            return Err(MidiError::BadMemoryAddress { bytes: addr.into() });
        }

        self.0[setup][note][param] = value;
        Ok(())
    }
}
