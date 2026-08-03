use serde::Deserialize;
use strum_macros::EnumString;

#[derive(Debug, Deserialize, EnumString, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum InterpolatingMethods {
    /// Low cpu usage
    Linear,

    /// Mid cpu usage
    Hermite,

    /// High cpu usage
    #[serde(alias = "lanczos", alias = "lanczos-3")]
    Lanczos3,
}

