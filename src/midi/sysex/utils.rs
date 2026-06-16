macro_rules! get_or_skip {
    ($data:expr, $idx:expr) => {
        match $data.get($idx) {
            Some(r) => r,
            None => return,
        }
    };
}
