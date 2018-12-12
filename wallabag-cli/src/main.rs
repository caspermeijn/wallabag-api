use std::fs::File;
use std::io::Read;

use clap::{App, Arg, SubCommand};
use failure::Fallible;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use simplelog::WriteLogger;

use wallabag_backend::{Backend, Config as BackendConfig};

const INIT: &'static str = "init";
const SYNC: &'static str = "sync";
const TAGS: &'static str = "tags";
const ADD: &'static str = "add";
const RESET: &'static str = "reset";

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    backend: BackendConfig,
}

fn main() -> Fallible<()> {
    // init logging
    // TODO: dynamically configure file name
    WriteLogger::init(
        simplelog::LevelFilter::Trace,
        simplelog::Config::default(),
        File::create("wallabag-cli.log").unwrap(),
    )?;

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
        .subcommand(SubCommand::with_name(RESET).about("reset the database (all data lost)"))
        .subcommand(
            SubCommand::with_name(SYNC)
                .about("bidirectional sync database with server")
                .arg(
                    Arg::with_name("full")
                        .long("full")
                        .help("Perform a full sync (slow)."),
                ),
        )
        .subcommand(
            SubCommand::with_name(ADD)
                .about("Add a new url")
                .arg(Arg::with_name("URL").help("The url to save").required(true))
                .arg(
                    Arg::with_name("upload")
                        .short("u")
                        .long("upload")
                        .help("Upload immediately (requires network connection)"),
                ),
        )
        .subcommand(SubCommand::with_name(TAGS).about("Display a list of tags"));

    let matches = app.get_matches();

    // Load config from file
    // TODO: default and override in args for filename
    let conf_file_name = "examples/wallabag-cli.toml";
    debug!("Attempting to load conf from {}", conf_file_name);
    let s = read_file(conf_file_name)?;
    let conf: Config = toml::from_str(&s)?;

    // TODO: allow command line args to override those in conf file

    let mut backend = Backend::new_with_conf(conf.backend)?;

    match matches.subcommand_name() {
        None => {
            println!(":: No subcommand given.");
        }
        Some(INIT) => {
            println!(":: Initing the database.");
            backend.init()?;
        }
        Some(RESET) => {
            println!(":: Resetting the database to a clean state.");
            backend.reset()?;
        }
        Some(SYNC) => {
            let sync_matches = matches.subcommand_matches(SYNC).unwrap();
            if sync_matches.is_present("full") {
                println!(":: Running a full sync.");
                backend.full_sync()?;
            } else {
                println!(":: Running a normal sync.");
                backend.sync()?;
            }
        }
        Some(ADD) => {
            let add_matches = matches.subcommand_matches(ADD).unwrap();
            let url = add_matches.value_of("URL").unwrap();
            if add_matches.is_present("upload") {
                backend.add_url_online(url)?;
            } else {
                backend.add_url(url)?;
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

fn read_file(fname: &str) -> Fallible<String> {
    let mut file = File::open(fname)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
