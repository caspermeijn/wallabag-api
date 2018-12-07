mod backend;

use std::env;
use std::fs::File;
use std::io;

use clap::{App, Arg, SubCommand};
use failure::Fallible;
use log::{debug, error, info, warn};
use simplelog::WriteLogger;

use wallabag_api::types::Config;
use wallabag_api::Client;

use crate::backend::Backend;

const INIT: &'static str = "init";
const SYNC: &'static str = "sync";
const TAGS: &'static str = "tags";

fn main() -> Fallible<()> {
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
        .subcommand(SubCommand::with_name(INIT).about("init the database"))
        .subcommand(
            SubCommand::with_name(SYNC)
                .about("bidirectional sync database with server")
                .arg(
                    Arg::with_name("full")
                        .long("full")
                        .help("Perform a full sync (slow)."),
                ),
        )
        .subcommand(SubCommand::with_name(TAGS).about("Display a list of tags"));

    let matches = app.get_matches();

    let backend = Backend::new("db.sqlite3");

    match matches.subcommand_name() {
        None => {
            println!("No subcommand given.");
        }
        Some(INIT) => {
            println!("Initing the database...");
            let res = backend.init();
            println!("{:#?}", res);
        }
        Some(SYNC) => {
            let sync_matches = matches.subcommand_matches(SYNC).unwrap();
            if sync_matches.is_present("full") {
                println!("Running a full sync.");
                backend.full_sync()?;
            } else {
                println!("Running a normal sync.");
                backend.sync()?;
            }
        }
        Some(TAGS) => {
            let mut tags = backend.tags()?;
            tags.sort_unstable_by(|left, right| left.label.cmp(&right.label));

            for tag in tags {
                println!("{}", tag.label);
            }
        }
        _ => {
            // shrug
        }
    }

    Ok(())
}
