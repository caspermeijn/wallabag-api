
mod backend;

use std::env;
use std::fs::File;
use std::io;

use clap::{App, Arg, SubCommand};
use log::{debug, error, info, warn};
use simplelog::WriteLogger;

use wallabag_api::types::Config;
use wallabag_api::Client;

use crate::backend::init_db;

const INIT: &'static str = "init";

fn main() -> Result<(), failure::Error> {
    // init logging
    WriteLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        File::create("wallabag-cli.log").unwrap(),
    );

    let app = App::new("Wallabag CLI")
        .version("alpha")
        .about("Command line client for Wallabag")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Use a custom config file")
                .takes_value(true),
        )
        .subcommand(SubCommand::with_name(INIT).about("init the database"));

    let matches = app.get_matches();

    match matches.subcommand_name() {
        None => {
            println!("No subcommand given.");
            backend::load_tags();
        }
        Some(INIT) => {
            println!("Initing the database...");
            init_db()?;
        }
        _ => {
            // shrug
        }
    }

    Ok(())
}
