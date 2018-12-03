// backend module that links the api client to a local database and provides
// sync

mod db;

use std::cmp::Ordering::{Equal, Greater, Less};
use std::env;
use std::path::PathBuf;
use std::result::Result;

use chrono::{DateTime, Utc};
use failure::Fallible;
use rusqlite::types::ToSql;
use rusqlite::Result as SQLResult;
use rusqlite::{Connection, NO_PARAMS};
use serde_json;

use wallabag_api::types::{Annotation, Config, EntriesFilter, Entry, Tag, ID};
use wallabag_api::Client;

use self::db::DB;

pub struct Backend {
    db: DB,
}

impl Backend {
    pub fn new<T: Into<PathBuf>>(db_file: T) -> Self {
        Backend {
            db: DB::new(db_file),
        }
    }

    pub fn init(&self) -> Result<(), failure::Error> {
        self.db.init()?;

        Ok(())
    }

    /// Sync between the database and server.
    ///
    /// Newest wins.
    pub fn sync(&self) -> Fallible<()> {
        let config = Config {
            client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
            client_secret: env::var("WALLABAG_CLIENT_SECRET")
                .expect("WALLABAG_CLIENT_SECRET not set"),
            username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
            password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
            base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
        };

        let mut client = Client::new(config);

        // sync tags first

        let tags = client.get_tags()?;
        for tag in tags {
            self.db.save_tag(&tag, true)?;
        }

        // then sync entries
        let mut filter = EntriesFilter::default();
        filter.since = self.db.get_last_sync()??.timestamp() as u64;

        let entries = client.get_entries_with_filter(filter)?;

        for entry in entries.into_iter() {
            if let Some(saved_entry) = self.db.get_entry(entry.id.as_u32())? {
                match saved_entry.updated_at.cmp(&entry.updated_at) {
                    Less => {
                        self.db.save_entry(&entry, true)?;
                        if let Some(ref anns) = entry.annotations {
                            for ann in anns {
                                self.db.save_annotation(ann, &entry, true)?;
                            }
                        }

                        // TODO: tags support
                        println!("upsert");
                    }
                    Equal => {
                        // noop; already synced and same version
                    }
                    Greater => {
                        // TODO: db is newer; update
                        println!("need to sync to server");
                    }
                }
            } else {
                self.db.save_entry(&entry, true)?;
                if let Some(ref anns) = entry.annotations {
                    for ann in anns {
                        self.db.save_annotation(&ann, &entry, true)?;
                    }
                }
                println!("upsert");
            }
        }

        // finally sync up unsynced things;

        self.db.update_last_sync()?;

        Ok(())
    }
}
