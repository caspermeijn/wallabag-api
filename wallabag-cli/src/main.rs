use std::fs::{File, OpenOptions};
use std::io::Read;
use std::fmt;

use clap::{App, Arg, SubCommand};
use failure::{Fallible, bail};
use log::debug;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use simplelog::{Level, LevelFilter, WriteLogger};

use wallabag_backend::{Backend, Config as BackendConfig};

#[derive(Debug)]
pub struct MessageError(String);

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for MessageError {}

const INIT: &'static str = "init";
const SYNC: &'static str = "sync";
const TAGS: &'static str = "tags";
const ADD: &'static str = "add";
const RESET: &'static str = "reset";
const ENTRIES: &'static str = "entry";
const LIST: &'static str = "list";
const SHOW: &'static str = "show";

#[derive(Deserialize, Serialize, Debug)]
struct CliConfig {
    log_file: String,
    #[serde(deserialize_with = "parse_level_filter")]
    #[serde(serialize_with = "serialize_level_filter")]
    log_level: LevelFilter,
}

/// Parser for converting string to LevelFilter with serde
fn parse_level_filter<'de, D>(d: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    let x = String::deserialize(d)?;

    match x.as_str().to_lowercase().as_ref() {
        "off" => Ok(LevelFilter::Off),
        "error" => Ok(LevelFilter::Error),
        "warn" => Ok(LevelFilter::Warn),
        "info" => Ok(LevelFilter::Info),
        "debug" => Ok(LevelFilter::Debug),
        "trace" => Ok(LevelFilter::Trace),
        x => Err(DeError::custom(format!(
            "Could not deserialize {:?} as a level filter",
            x
        ))),
    }
}

/// Serializer for serializing a LevelFilter as a String
fn serialize_level_filter<S>(x: &LevelFilter, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}", x))
}

#[derive(Deserialize, Serialize, Debug)]
struct Config {
    cli: CliConfig,
    backend: BackendConfig,
}

fn main() -> Fallible<()> {
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
        .subcommand(SubCommand::with_name(TAGS).about("Display a list of tags"))
        .subcommand(
            SubCommand::with_name(ENTRIES)
                .about("Work with entries")
                .subcommand(SubCommand::with_name(LIST).about("List all entries"))
                .subcommand(SubCommand::with_name(SHOW).about("Print the entry's content")
                            .arg(Arg::with_name("entry_id").required(true).help("ID of the entry to show")
                            )),
        );

    let matches = app.get_matches();

    // Load config from file
    // TODO: default and override in args for filename
    let conf_file_name = "examples/wallabag-cli.toml";
    debug!("Attempting to load conf from {}", conf_file_name);
    let s = read_file(conf_file_name)?;
    let config: Config = toml::from_str(&s)?;
    // TODO: allow command line args to override those in conf file

    // init logging
    WriteLogger::init(
        config.cli.log_level,
        simplelog::Config {
            time: Some(Level::Error),
            level: Some(Level::Error),
            target: Some(Level::Error),
            location: Some(Level::Error),
            time_format: Some("%F %T"),
        },
        OpenOptions::new().create(true).append(true).open(config.cli.log_file)?,
    )?;

    let mut backend = Backend::new_with_conf(config.backend)?;

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
        Some(ENTRIES) => {
            let entries_matches = matches.subcommand_matches(ENTRIES).unwrap();
            match entries_matches
                .subcommand_name()
            {
                None => {
                    // ¯\_(ツ)_/¯
                }
                Some(LIST) => {
                    let entries = backend.entries()?;

                    for entry in entries {
                        println!(
                            "{} {}",
                            entry.id.as_int(),
                            entry.title.unwrap_or_else(|| "UNTITLED".to_owned())
                        );
                    }
                }
                Some(SHOW) => {
                    let id: i64 = entries_matches.subcommand_matches(SHOW).unwrap().value_of("entry_id").unwrap().parse()?;
                    let entry = match backend.get_entry(id)? {
                        Some(entry) => entry,
                        None => {
                            bail!("Entry not found");
                        }
                    };

                    match entry.content {
                        Some(s) => {
                            println!("{}", s);
                        }
                        None => {
                            bail!("No content");
                        }
                    }
                }
                Some(_) => {
                    // ¯\_(ツ)_/¯
                }
            }
        }
        Some(_) => {
            unreachable!();
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
