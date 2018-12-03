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

    /// Full sync. Can be slow if many articles. This will also sync deletes.
    ///
    /// For entries and annotations existing in the database, object with latest
    /// updated_at value wins.
    pub fn full_sync(&self) -> Fallible<()> {
        unimplemented!()
    }

    /// Normal sync. Syncs everything changed since the last sync, with the
    /// exception of deleted entries and annotations (syncing deletes requires a
    /// full sweep through everything).
    ///
    /// For entries and annotations existing in the database, object with latest
    /// updated_at value wins.
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

        // sync tags first.
        // there are no updated-at timestamps attached, so just pull and save
        // everything.
        self.pull_tags(&mut client)?;

        // TODO: push up seperate 'new tags' or 'tag edits' items

        // Then sync entries. Entries have tag links and annotations embedded.
        let mut filter = EntriesFilter::default();
        filter.since = self.db.get_last_sync()??.timestamp() as u64;

        let entries = client.get_entries_with_filter(filter)?;

        // NOTE: changing tags on an entry touches updated_at

        for entry in entries.into_iter() {
            if let Some(saved_entry) = self.db.get_entry(entry.id.as_u32())? {
                match saved_entry.updated_at.cmp(&entry.updated_at) {
                    Less => {
                        self.pull_entry(entry)?;
                    }
                    Equal => {
                        // noop; already synced and same version
                    }
                    Greater => {
                        // local entry is newer, push to server
                        client.update_entry(saved_entry.id, &(&saved_entry).into())?;
                        // TODO: handle annonations for entry
                    }
                }
            } else {
                self.pull_entry(entry)?;
            }
        }

        // finally sync up unsynced things;
        // TODO

        // touch the last sync time ready for next sync
        self.db.touch_last_sync()?;

        Ok(())
    }

    /// save an entry to the database where the entry has been determined to be
    /// newer than any in the database, but still need to do bidirectional sync
    /// for associated annotations and tags
    fn pull_entry(&self, entry: Entry) -> Fallible<()> {
        self.db.save_entry(&entry, true)?;
        if let Some(ref anns) = entry.annotations {
            for ann in anns {
                self.db.save_annotation(ann, &entry, true)?;
            }
        }

        // TODO: tags support

        Ok(())
    }

    fn pull_tags(&self, client: &mut Client) -> Fallible<()> {
        let tags = client.get_tags()?;
        for tag in tags {
            self.db.save_tag(&tag, true)?;
        }
        Ok(())
    }
}
