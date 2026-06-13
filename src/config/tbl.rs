use serde::Deserialize;
use strum_macros::EnumString;

fn default_tbl_type() -> TBLType {
    TBLType::Auto
}

#[derive(Debug, Deserialize, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum TBLType {
    Auto,
    #[serde(rename = "s-yxg50", alias = "xg50")]
    Syxg50,
    #[serde(rename = "syxg2006le", alias = "xg2006", alias = "xg2006le")]
    Syxg2006le,
}

#[derive(Debug, Deserialize)]
pub struct TBLConfig {
    /// type for tbl file
    #[serde(default = "default_tbl_type")]
    pub tbl_type: TBLType,

    /// bin tbl file, such as `sxgbin21.tbl` `sxgbnw6l.tbl`
    pub tbl_bin_file: String,
    /// wave tbl file
    pub tbl_data_file: String,
}
