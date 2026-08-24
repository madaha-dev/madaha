#![deny(warnings)]
#![deny(unsafe_code)]

pub mod args;
pub mod audio;
#[cfg(test)]
pub mod audit_tests;
pub mod config;
pub mod double_buffer;
pub mod fast_sine;
pub mod lfo;
pub mod midi;
pub mod plugin;
pub mod synth;
pub mod utils;
pub mod voice_manager;

#[cfg(test)]
mod e2e_tests;

use std::fs;

use clap::Parser;
use libmadaha::LoadedModule;
use libmadaha::load;
use mimalloc::MiMalloc;
use wd_log::{DEBUG, log_debug_ln, log_info_ln, log_panic, set_level, set_prefix};

use crate::{
    args::Args,
    config::{Config, ConfigObject},
    synth::Synth,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    set_prefix("Madaha");
    let args = Args::parse();

    log_info_ln!("hello!");

    if args.generate_default_config {
        match Config::generate_default(args.config.clone()) {
            Ok(_) => {
                log_info_ln!("default config file saved: {}", args.config);
                return;
            }
            Err(err) => log_panic!("{:?}", err),
        }
    }

    log_info_ln!("loading config file \"{}\"", args.config);
    let cfg = match Config::from_file(args.config.clone()) {
        Ok(c) => c,
        Err(err) => log_panic!("{:?}", err),
    };

    if let Err(err) = cfg.check() {
        log_panic!("{:?}", err)
    }

    if !args.dump_tbl_to_json_file.is_empty() {
        let data = load(
            cfg.sound_module.module_type,
            cfg.sound_module.tbl_bin_file,
            cfg.sound_module.tbl_data_file,
        )
        .unwrap();

        match data {
            LoadedModule::Syxg50(tbl) => {
                let j = tbl.to_json().unwrap();
                fs::write(&args.dump_tbl_to_json_file, j).unwrap();
                log_info_ln!("tbl file dump save to: {}", args.dump_tbl_to_json_file);
            }
        }

        return;
    }

    if args.debug {
        set_level(DEBUG);
        log_debug_ln!("debug mode force enabled");
    }
    log_debug_ln!("config={:?}", cfg);

    let mut synth = Synth::new();
    synth.run(&cfg, &args);
}
