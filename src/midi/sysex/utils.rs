macro_rules! get_or_skip {
    ($data:expr, $idx:expr) => {
        match $data.get($idx) {
            Some(r) => r,
            None => return,
        }
    };
}

macro_rules! get_dev_id {
    ($data:expr) => {
        *$data.get(0).unwrap_or(&SYSEX_CHANNEL_ALL_DEVICE)
    };
}