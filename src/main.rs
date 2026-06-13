mod args;
mod config;
mod midi;
mod synth;
mod effects;
mod macros;
mod tbl;

use clap::Parser;
use wd_log::{DEBUG, log_debug_ln, log_info_ln, log_panic, set_level, set_prefix};

use crate::{args::Args, config::Config, synth::Synth};

fn main() {
    set_prefix("Madaha");
    let args = Args::parse();

    log_info_ln!("hello!");

    log_info_ln!("loading config file \"{}\"", args.config);
    let cfg = match Config::from_file(args.config) {
        Ok(c) => c,
        Err(err) => log_panic!("{:?}", err),
    };

    if args.debug {
        set_level(DEBUG);
        log_debug_ln!("debug mode force enabled");
    }
    log_debug_ln!("config={:?}", cfg);

    let synth = Synth::new(&cfg);
    synth.run();
}
