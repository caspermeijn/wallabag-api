use std::env;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Read;
use std::str::FromStr;
use std::path::PathBuf;

use failure::{bail, format_err, Fallible};
use log::debug;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use simplelog::{Level, LevelFilter, WriteLogger};
use structopt::StructOpt;

use gtk::Orientation::Vertical;
use gtk::{
    ContainerExt, EditableSignals, Entry, EntryExt, Inhibit, Label, LabelExt, WidgetExt, Window,
    WindowType,
};
use relm::{connect, connect_stream, Relm, Update, Widget};
use relm_derive::Msg;

use wallabag_backend::{Backend, Config as BackendConfig, StringOrCmd};

struct Model {
    content: String,
}

#[derive(Msg)]
enum Msg {
    Change,
    Quit,
}

struct App {
    model: Model,
    widgets: Widgets,
    backend: Backend,
}

#[derive(Clone)]
struct Widgets {
    input: Entry,
    label: Label,
    window: Window,
}

impl Update for App {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: Self::ModelParam) -> Model {
        Model {
            content: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Change => {
                self.model.content = self
                    .widgets
                    .input
                    .get_text()
                    .expect("get_text failed")
                    .chars()
                    .rev()
                    .collect();

                let tags = self.backend.tags().unwrap();
                let temptxt = tags.into_iter().map(|t| t.label).collect::<Vec<String>>().join("\n").to_owned();
                self.widgets.label.set_text(&temptxt);
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for App {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    /// Creates the initial view - put any other initialization here.
    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let vbox = gtk::Box::new(Vertical, 0);

        let input = Entry::new();
        vbox.add(&input);

        let label = Label::new(None);
        vbox.add(&label);

        let window = Window::new(WindowType::Toplevel);

        window.add(&vbox);

        window.show_all();

        connect!(relm, input, connect_changed(_), Msg::Change);
        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        // TODO: is here really a good place to run setup things that could fail? It seems like the
        // only place that Relm will accept it though...
        let backend = Backend::new_with_conf(BackendConfig {
            db_file: PathBuf::from("db.sqlite3"),
            client_id: StringOrCmd::S(env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set")),
            client_secret: StringOrCmd::S(env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set")),
            username: StringOrCmd::S(env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set")),
            password: StringOrCmd::S(env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set")),
            base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
        }).expect("TODO: nice error message");

        App {
            model,
            widgets: Widgets {
                input,
                label,
                window,
            },
            backend: backend,
        }
    }
}

fn main() -> Fallible<()> {
    WriteLogger::init(
        LevelFilter::Debug,
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
            .open("wallabag-gtk.log")?,
    )?;

    App::run(()).map_err(|_| format_err!("GTK app failed to launch"))
}
