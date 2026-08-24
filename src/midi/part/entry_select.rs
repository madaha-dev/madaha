#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataEntrySelect {
    None,
    #[allow(clippy::upper_case_acronyms)]
    RPN,
    #[allow(clippy::upper_case_acronyms)]
    NRPN,
}
