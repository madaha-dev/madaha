const HEADER: &[u8] = b"UTG VPRM";

pub fn check_header(header: &[u8]) -> bool {
    header == HEADER
}