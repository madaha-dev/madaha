#[macro_export]
macro_rules! merge_data {
    ($msb:expr) => {
        $msb << 8
    };
    ($msb:expr, $lsb:expr) => {
        $msb << 8 | $lsb
    };
}

#[macro_export]
macro_rules! impl_xg_effect_type {
    ($type_name:ident, $no_effect:ident) => {
        impl $type_name {
            pub fn get_type(msb: u8, lsb: u8) -> Self {
                let full = merge_data!(msb as u16, lsb as u16);
                match Self::try_from(full) {
                    Ok(r) => r,
                    Err(_) => {
                        let msb_only = merge_data!(msb as u16);
                        Self::try_from(msb_only).unwrap_or(Self::$no_effect)
                    }
                }
            }
        }
    };
}