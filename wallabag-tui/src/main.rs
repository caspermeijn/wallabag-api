mod event;

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::str::FromStr;

use failure::{bail, Fallible};
use log::{debug, error, info, warn};
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use simplelog::{Level, LevelFilter, WriteLogger};
use structopt::StructOpt;

use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Corner, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, SelectableList, Text, Widget};
use tui::Terminal;

use wallabag_backend::types::Entries;
use wallabag_backend::{Backend, Config as BackendConfig};

use crate::event::{Event, Events};

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
/// Command line user interface for Wallabag.
struct Opt {
    /// Uses a custom config file
    #[structopt(long = "config", short = "c")]
    config: Option<String>,

    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
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

struct App {
    size: Rect,
    selected: Option<usize>,
    info_style: Style,
    warning_style: Style,
    error_style: Style,
    critical_style: Style,
    entries: Entries,
}

impl App {
    fn new() -> App {
        App {
            size: Rect::default(),
            selected: None,
            info_style: Style::default().fg(Color::White),
            warning_style: Style::default().fg(Color::Yellow),
            error_style: Style::default().fg(Color::Magenta),
            critical_style: Style::default().fg(Color::Red),
            entries: vec![],
        }
    }

    fn run() -> Fallible<()> {
        loop {}
    }
}

fn main() -> Fallible<()> {
    let opt = Opt::from_args();

    // intercept this now, before attempting to load config
    if let Some(SubCommand::ExampleConf) = opt.cmd {
        let conf = include_str!("../example-config.toml");
        println!("{}", conf);
        return Ok(());
    }

    // load config
    let conf_file_name = opt
        .config
        .unwrap_or_else(|| "wallabag-tui/example-config.toml".to_owned());
    debug!("Attempting to load conf from {}", conf_file_name);
    let s = read_file(&conf_file_name)?;
    let config: Config = toml::from_str(&s)?;

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
        Some(SubCommand::Reset) => {
            println!(":: Resetting the database to a clean state.");
            backend.reset()?;
        }
        Some(SubCommand::ExampleConf) => {
            // can never reach here
        }
        Some(SubCommand::Sync { full }) => {
            if full {
                println!(":: Running a full sync.");
                backend.full_sync()?;
            } else {
                println!(":: Running a normal sync.");
                backend.sync()?;
            }
        }
        None => {
            run_tui(backend)?;
        }
    }

    Ok(())
}

fn run_tui(backend: Backend) -> Fallible<()> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let tbackend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(tbackend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // App
    let mut app = App::new();

    app.entries = backend.entries()?;

    let mut selected_entry = 0;
    let mut show_article = false;
    let mut scroll = 0;

    loop {
        let size = terminal.size()?;
        if size != app.size {
            terminal.resize(size)?;
            app.size = size;
        }

        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(app.size);

            let style = Style::default().fg(Color::Black).bg(Color::White);

            if !show_article {
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Entries"))
                    .items(
                        &app.entries
                            .iter()
                            .map(|entry| {
                                format!(
                                    "{} | {}",
                                    entry.id,
                                    match entry.title.as_ref() {
                                        Some(t) => t,
                                        None => "Untitled",
                                    }
                                )
                            })
                            .collect::<Vec<String>>(),
                    )
                    .select(app.selected)
                    .style(style)
                    .highlight_style(style.bg(Color::LightGreen).modifier(Modifier::Bold))
                    .render(&mut f, chunks[0]);
            } else {
                let content = app.entries[selected_entry].content.clone().unwrap();
                let text = [Text::raw(html2text::from_read(content.as_bytes(), 80))];
                tui::widgets::Paragraph::new(text.iter())
                    .block(Block::default().title("Paragraph").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .alignment(tui::layout::Alignment::Left)
                    .scroll(scroll)
                    .render(&mut f, chunks[0]);
            }
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    app.selected = None;
                }
                Key::Down => {
                    scroll += 1;
                    app.selected = if let Some(selected) = app.selected {
                        if selected >= app.entries.len() - 1 {
                            Some(0)
                        } else {
                            Some(selected + 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Up => {
                    scroll -= 1;
                    app.selected = if let Some(selected) = app.selected {
                        if selected > 0 {
                            Some(selected - 1)
                        } else {
                            Some(app.entries.len() - 1)
                        }
                    } else {
                        Some(0)
                    }
                }
                Key::Char('m') => {
                    if let Some(sel) = app.selected {
                        selected_entry = sel;
                        show_article = true;
                    }
                }
                _ => {}
            },
            Event::Tick => {
                debug!("tick");
                // app.advance();
            }
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
