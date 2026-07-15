const HEADER: &[u8] = b"MU50";

pub fn check_header(header: &[u8]) -> bool {
    header == HEADER
}
