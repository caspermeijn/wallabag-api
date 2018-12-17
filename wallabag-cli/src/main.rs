use std::io;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Read;

use failure::{bail, Fallible};
use log::debug;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use simplelog::{Level, LevelFilter, WriteLogger};
use structopt::StructOpt;

use wallabag_backend::{Backend, Config as BackendConfig};

#[derive(Debug)]
pub struct MessageError(String);

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for MessageError {}

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

#[derive(Debug, StructOpt)]
/// Command line client for Wallabag.
struct Opt {
    /// Uses a custom config file
    #[structopt(long = "config", short = "c")]
    config: Option<String>,

    #[structopt(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// Initializes the database
    #[structopt(name = "init")]
    Init,

    /// Resets the database to a clean state
    #[structopt(name = "reset")]
    Reset,

    /// Prints example config
    #[structopt(name = "example-conf")]
    ExampleConf,

    /// Syncs database with the server
    #[structopt(name = "sync")]
    Sync {
        /// Performs a full sync (slow)
        #[structopt(long = "full")]
        full: bool,
    },

    /// Adds a new url
    #[structopt(name = "add")]
    Add {
        /// Uploads and saves immediately (requires network connection)
        #[structopt(long = "upload", short = "u")]
        upload: bool,

        /// Url to save
        #[structopt(name = "url")]
        url: String,
    },

    /// Exports all local data to json
    #[structopt(name = "export")]
    Export {
        /// Pretty prints json
        #[structopt(long = "pretty", short = "p")]
        pretty: bool,
    },

    /// Prints a list of tags in the db
    #[structopt(name = "tags")]
    Tags,

    /// Works with entries
    #[structopt(name = "entry")]
    Entry {
        #[structopt(subcommand)]
        cmd: EntrySubCommand,
    },
}

#[derive(Debug, StructOpt)]
enum EntrySubCommand {
    /// Lists all entries
    #[structopt(name = "list")]
    List,

    /// Prints the entry's content
    #[structopt(name = "show")]
    Show {
        /// Id of the entry to show
        #[structopt(name = "id")]
        id: i64,
    },
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
    let opt = Opt::from_args();

    // intercept this now, before attempting to load config
    if let SubCommand::ExampleConf = opt.cmd {
        let conf = include_str!("../example-config.toml");
        println!("{}", conf);
        return Ok(());
    }

    // Load config from file
    // TODO: sensible default for config file
    let conf_file_name = opt
        .config
        .unwrap_or_else(|| "wallabag-cli/example-config.toml".to_owned());
    debug!("Attempting to load conf from {}", conf_file_name);
    let s = read_file(&conf_file_name)?;
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
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(config.cli.log_file)?,
    )?;

    let mut backend = Backend::new_with_conf(config.backend)?;

    match opt.cmd {
        SubCommand::Init => {
            println!(":: Initing the database.");
            backend.init()?;
        }
        SubCommand::Reset => {
            println!(":: Resetting the database to a clean state.");
            backend.reset()?;
        }
        SubCommand::ExampleConf => {
            // can never reach here
        }
        SubCommand::Sync { full } => {
            if full {
                println!(":: Running a full sync.");
                backend.full_sync()?;
            } else {
                println!(":: Running a normal sync.");
                backend.sync()?;
            }
        }
        SubCommand::Add { upload, url } => {
            if upload {
                backend.add_url_online(url)?;
            } else {
                backend.add_url(url)?;
            }
        }
        SubCommand::Tags => {
            let mut tags = backend.tags()?;
            tags.sort_unstable_by(|left, right| left.label.cmp(&right.label));

            for tag in tags {
                println!("{}", tag.label);
            }
        }
        SubCommand::Export { pretty } => {
            let val = backend.export()?;

            let stdout = io::stdout();
            let handle = stdout.lock();

            if pretty {
                serde_json::to_writer_pretty(handle, &val)?;
            } else {
                serde_json::to_writer(handle, &val)?;
            }
        }
        SubCommand::Entry { cmd } => match cmd {
            EntrySubCommand::List => {
                let entries = backend.entries()?;

                for entry in entries {
                    println!(
                        "{} {}",
                        entry.id.as_int(),
                        entry.title.unwrap_or_else(|| "UNTITLED".to_owned())
                    );
                }
            }
            EntrySubCommand::Show { id } => {
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
        },
    }

    Ok(())
}

fn read_file(fname: &str) -> Fallible<String> {
    let mut file = File::open(fname)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
