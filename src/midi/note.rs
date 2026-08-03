use std::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Note {
    Cm2 = 0x00,  // 0
    Csm2 = 0x01, // 1
    Dm2 = 0x02,  // 2
    Dsm2 = 0x03, // 3
    Em2 = 0x04,  // 4
    Fm2 = 0x05,  // 5
    Fsm2 = 0x06, // 6
    Gm2 = 0x07,  // 7
    Gsm2 = 0x08, // 8
    Am2 = 0x09,  // 9
    Asm2 = 0x0A, // 10
    Bm2 = 0x0B,  // 11
    Cm1 = 0x0C,  // 12
    Csm1 = 0x0D, // 13
    Dm1 = 0x0E,  // 14
    Dsm1 = 0x0F, // 15
    Em1 = 0x10,  // 16
    Fm1 = 0x11,  // 17
    Fsm1 = 0x12, // 18
    Gm1 = 0x13,  // 19
    Gsm1 = 0x14, // 20
    Am1 = 0x15,  // 21
    Asm1 = 0x16, // 22
    Bm1 = 0x17,  // 23
    C0 = 0x18,   // 24
    Cs0 = 0x19,  // 25
    D0 = 0x1A,   // 26
    Ds0 = 0x1B,  // 27
    E0 = 0x1C,   // 28
    F0 = 0x1D,   // 29
    Fs0 = 0x1E,  // 30
    G0 = 0x1F,   // 31
    Gs0 = 0x20,  // 32
    A0 = 0x21,   // 33
    As0 = 0x22,  // 34
    B0 = 0x23,   // 35
    C1 = 0x24,   // 36
    Cs1 = 0x25,  // 37
    D1 = 0x26,   // 38
    Ds1 = 0x27,  // 39
    E1 = 0x28,   // 40
    F1 = 0x29,   // 41
    Fs1 = 0x2A,  // 42
    G1 = 0x2B,   // 43
    Gs1 = 0x2C,  // 44
    A1 = 0x2D,   // 45
    As1 = 0x2E,  // 46
    B1 = 0x2F,   // 47
    C2 = 0x30,   // 48
    Cs2 = 0x31,  // 49
    D2 = 0x32,   // 50
    Ds2 = 0x33,  // 51
    E2 = 0x34,   // 52
    F2 = 0x35,   // 53
    Fs2 = 0x36,  // 54
    G2 = 0x37,   // 55
    Gs2 = 0x38,  // 56
    A2 = 0x39,   // 57
    As2 = 0x3A,  // 58
    B2 = 0x3B,   // 59
    C3 = 0x3C,   // 60
    Cs3 = 0x3D,  // 61
    D3 = 0x3E,   // 62
    Ds3 = 0x3F,  // 63
    E3 = 0x40,   // 64
    F3 = 0x41,   // 65
    Fs3 = 0x42,  // 66
    G3 = 0x43,   // 67
    Gs3 = 0x44,  // 68
    A3 = 0x45,   // 69
    As3 = 0x46,  // 70
    B3 = 0x47,   // 71
    C4 = 0x48,   // 72
    Cs4 = 0x49,  // 73
    D4 = 0x4A,   // 74
    Ds4 = 0x4B,  // 75
    E4 = 0x4C,   // 76
    F4 = 0x4D,   // 77
    Fs4 = 0x4E,  // 78
    G4 = 0x4F,   // 79
    Gs4 = 0x50,  // 80
    A4 = 0x51,   // 81
    As4 = 0x52,  // 82
    B4 = 0x53,   // 83
    C5 = 0x54,   // 84
    Cs5 = 0x55,  // 85
    D5 = 0x56,   // 86
    Ds5 = 0x57,  // 87
    E5 = 0x58,   // 88
    F5 = 0x59,   // 89
    Fs5 = 0x5A,  // 90
    G5 = 0x5B,   // 91
    Gs5 = 0x5C,  // 92
    A5 = 0x5D,   // 93
    As5 = 0x5E,  // 94
    B5 = 0x5F,   // 95
    C6 = 0x60,   // 96
    Cs6 = 0x61,  // 97
    D6 = 0x62,   // 98
    Ds6 = 0x63,  // 99
    E6 = 0x64,   // 100
    F6 = 0x65,   // 101
    Fs6 = 0x66,  // 102
    G6 = 0x67,   // 103
    Gs6 = 0x68,  // 104
    A6 = 0x69,   // 105
    As6 = 0x6A,  // 106
    B6 = 0x6B,   // 107
    C7 = 0x6C,   // 108
    Cs7 = 0x6D,  // 109
    D7 = 0x6E,   // 110
    Ds7 = 0x6F,  // 111
    E7 = 0x70,   // 112
    F7 = 0x71,   // 113
    Fs7 = 0x72,  // 114
    G7 = 0x73,   // 115
    Gs7 = 0x74,  // 116
    A7 = 0x75,   // 117
    As7 = 0x76,  // 118
    B7 = 0x77,   // 119
    C8 = 0x78,   // 120
    Cs8 = 0x79,  // 121
    D8 = 0x7A,   // 122
    Ds8 = 0x7B,  // 123
    E8 = 0x7C,   // 124
    F8 = 0x7D,   // 125
    Fs8 = 0x7E,  // 126
    G8 = 0x7F,   // 127
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value: u8 = (*self).into();
        let octave = (value / 12) as i8 - 2;
        let note_name = match value % 12 {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            _ => unreachable!("Hey! how can you get here?"),
        };
        write!(f, "{}{}({})", note_name, octave, value)
    }
}

