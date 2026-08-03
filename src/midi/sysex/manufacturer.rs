use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

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
    // --- 40H - 5FH: Japanese + others ---
    Roland = 0x41,
    Yamaha = 0x43,

    // --- 7EH - 7FH: universal ---
    UniversalNonRealTime = 0x7E,
    UniversalRealTime = 0x7F,
}
