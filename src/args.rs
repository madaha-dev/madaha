use clap::Parser;

const DEFAULT_CONFIG_FILE: &'static str = "madaha.toml";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Enable debug mode
    #[clap(short = 'D', long, value_parser, default_value_t = false)]
    pub debug: bool,

    /// Config file path
    #[clap(short = 'C', long, value_parser, env = "MADAHA_CONFIG_FILE", default_value = DEFAULT_CONFIG_FILE)]
    pub config: String,

    #[clap(short = 'G', long, value_parser, default_value_t = false)]
    pub generate_default_config: bool,
}
