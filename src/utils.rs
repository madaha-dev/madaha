/// Used for 16-bit data!
#[macro_export]
macro_rules! merge_data {
    ($msb:expr) => {
        $msb << 8
    };
    ($msb:expr, $lsb:expr) => {
        $msb << 8 | $lsb
    };
}

/// Used for 16-bit data!
#[macro_export]
macro_rules! get_msb_u16_u8 {
    ($num:expr) => {
        (($num as u16) >> 8) as u8
    };
}

/// Used for 16-bit data!
#[macro_export]
macro_rules! get_lsb_u16_u8 {
    ($num:expr) => {
        (($num as u16) & 0x00FF) as u8
    };
}

/// Used for 14-bit data
#[macro_export]
macro_rules! get_msb {
    ($num:expr) => {
        (($num >> 7) & 0x7F) as u8
    };
}

/// Used for 14-bit data
#[macro_export]
macro_rules! get_lsb {
    ($num:expr) => {
        ($num & 0x7F) as u8
    };
}

/// Used for 14-bit data
#[macro_export]
macro_rules! get_14bit {
    ($msb:expr, $lsb:expr) => {
        (($msb & 0x7F) as u16) << 7 | (($lsb & 0x7F) as u16)
    };
}
