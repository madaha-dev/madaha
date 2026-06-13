use crate::midi::{
    sysex::{ManufacturerId, SYSEX_MSG_START},
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MidiEventID {
    NoteOff = 0x80,
    NoteOn = 0x90,
    PolyPressure = 0xA0,
    ControlChange = 0xB0,
    ProgramChange = 0xC0,
    ChannelPressure = 0xD0,
    PitchBend = 0xE0,
    SysEx = SYSEX_MSG_START,
    MTCQuarterFrame = 0xF1,
    SongPosition = 0xF2,
    SongSelect = 0xF3,
    TuneRequest = 0xF4,
    TimingClock = 0xF8,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
    ActiveSensing = 0xFE,
    SystemReset = 0xFF,
}

#[derive(Debug)]
pub enum MidiEvent {
    // channel event
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
        off_velocity: u8, // alsa compatible
        duration: u32,    // don't panic
    },
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
        off_velocity: u8,
        duration: u32,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    PitchBend {
        channel: u8,
        value: u16,
    },

    // RPN/NRPN
    RPN {
        channel: u8,
        parameter: u16,
        value: u16,
    },
    NRPN {
        channel: u8,
        parameter: u16,
        value: u16,
    },

    // aftertouch
    ChannelPressure {
        channel: u8,
        pressure: u8,
    },
    PolyPressure {
        channel: u8,
        note: u8,
        pressure: u8,
    },

    // sysex, data without (0xF0/0xF7)
    SysEx {
        manufacturer_id: ManufacturerId,
        data: Box<[u8]>,
    },

    // system common
    TuneRequest,
    MTCQuarterFrame {
        frame_type: u8,
        value: u8,
    },
    SongPosition {
        position: u16,
    },
    SongSelect {
        song: u8,
    },

    // real time
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,
}

