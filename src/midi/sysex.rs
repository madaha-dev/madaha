use std::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SYSEX_MSG_START: u8 = 0xF0;
pub const SYSEX_MSG_END: u8 = 0xF7;

macro_rules! define_manufacturer_ids {
    ( $( $variant:ident = $value:expr ),* $(,)? ) => {
        #[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
        #[repr(u8)]
        pub enum ManufacturerId {
            $( $variant = $value, )*
            #[num_enum(catch_all)]
            Other(u8),
        }

        impl fmt::Display for ManufacturerId {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $( Self::$variant => write!(f, concat!(stringify!($variant), "(0x{:02X})"), $value), )*
                    Self::Other(id) => write!(f, "Other(0x{:02X})", id),
                }
            }
        }

        impl fmt::Debug for ManufacturerId {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(self, f)
            }
        }
    };
}

define_manufacturer_ids! {
    // for 3-byte manufacturer
    ExtendedID = 0x00,

    // --- 01H - 3FH: one-byte IDs ---
    Sequential = 0x01,
    IDP = 0x02,
    TurtleBeach = 0x03,
    Moog = 0x04,
    Passport = 0x05,
    Lexicon = 0x06,
    Kurzweil = 0x07,
    Fender = 0x08,
    Midi9 = 0x09,
    Akg = 0x0A,
    Voyce = 0x0B,
    Waveframe = 0x0C,
    Ada = 0x0D,
    Garfield = 0x0E,
    Ensoniq = 0x0F,
    Oberheim = 0x10,
    Apple = 0x11,
    GreyMatter = 0x12,
    Digidesign = 0x13,
    Palmtree = 0x14,
    Jlcooper = 0x15,
    Lowrey = 0x16,
    AdamsSmith = 0x17,
    Emu = 0x18,
    Harmony = 0x19,
    Art = 0x1A,
    Baldwin = 0x1B,
    Eventide = 0x1C,
    Inventronics = 0x1D,
    KeyConcepts = 0x1E,
    Clarity = 0x1F,
    Passac = 0x20,
    Proel = 0x21,
    Synthaxe = 0x22,
    Stepp = 0x23,
    Hohner = 0x24,
    Twister = 0x25,
    Ketron = 0x26,
    Jellinghaus = 0x27,
    Southworth = 0x28,
    Ppg = 0x29,
    Cesyg = 0x2A,
    SolidStateLogic = 0x2B,
    AudioVeritrieb = 0x2C,
    Neve = 0x2D,
    Soundtracs = 0x2E,
    Elka = 0x2F,
    Dynacord = 0x30,
    Viscount = 0x31,
    Drawmer = 0x32,
    Clavia = 0x33,
    AudioArchitecture = 0x34,
    Generalmusic = 0x35,
    Cheetah = 0x36,
    Ctm = 0x37,
    Simmons = 0x38,
    Soundcraft = 0x39,
    Steinberg = 0x3A,
    Wersi = 0x3B,
    Avab = 0x3C,
    Digigram = 0x3D,
    Waldorf = 0x3E,
    Quasimidi = 0x3F,

    // --- 40H - 5FH: Japanese + others ---
    Kawai = 0x40,
    Roland = 0x41,
    Korg = 0x42,
    Yamaha = 0x43,
    Casio = 0x44,
    Kamiya = 0x46,
    Akai = 0x47,
    Jvc = 0x48,
    Fujitsu = 0x4B,
    Sony = 0x4C,
    Teac = 0x4E,
    Panasonic = 0x50,
    Fostex = 0x51,
    Zoom = 0x52,
    MatsushitaComm = 0x54,
    Suzuki = 0x55,
    FujiSound = 0x56,
    Atl = 0x57,
    Faith = 0x59,
    Internet = 0x5A,
    Seekers = 0x5C,
    SdCard = 0x5F,

    // --- 7EH - 7FH: universal ---
    UniversalNonRealTime = 0x7E,
    UniversalRealTime = 0x7F,
}
