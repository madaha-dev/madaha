#![deny(warnings)]
pub mod args;
pub mod audio;
#[cfg(test)]
pub mod audit_tests;
pub mod config;
pub mod double_buffer;
pub mod fast_sine;
pub mod lfo;
pub mod midi;
pub mod synth;
pub mod utils;
pub mod voice_manager;

#[cfg(test)]
mod e2e_tests;

use clap::Parser;
use wd_log::{DEBUG, log_debug_ln, log_info_ln, log_panic, set_level, set_prefix};

use crate::{
    args::Args,
    config::{Config, ConfigObject},
    synth::Synth,
};

fn main() {
    set_prefix("Madaha");
    let args = Args::parse();

    log_info_ln!("hello!");

    if args.generate_default_config {
        match Config::generate_default(args.config) {
            Ok(_) => return,
            Err(err) => log_panic!("{:?}", err),
        }
    }

    log_info_ln!("loading config file \"{}\"", args.config);
    let cfg = match Config::from_file(args.config) {
        Ok(c) => c,
        Err(err) => log_panic!("{:?}", err),
    };

    if let Err(err) = cfg.check() {
        log_panic!("{:?}", err)
    }

    if args.debug {
        set_level(DEBUG);
        log_debug_ln!("debug mode force enabled");
    }
    log_debug_ln!("config={:?}", cfg);

    let mut synth = Synth::new();
    synth.run(&cfg);
}
