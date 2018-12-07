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

use wallabag_api::types::{Annotation, Config, EntriesFilter, Entry, Tag, Tags, ID};
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

    pub fn init(&self) -> Fallible<()> {
        self.db.init()?;

        Ok(())
    }

    /// Get a Vec of tags from the db.
    pub fn tags(&self) -> Fallible<Tags> {
        self.db.get_tags()
    }

    /// Full sync. Can be slow if many articles. This will sync everything,
    /// including things that can't be synced with a quick/normal sync.
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
    ///
    /// What this does _not_ sync:
    ///
    /// - Entries deleted server-side.
    /// - Annotations deleted server-side.
    /// - Annotations updated or created server-side that are not associated
    ///   with entries updated since previous sync.
    ///
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
        // everything. We don't want to allow editing tags offline?
        self.pull_tags(&mut client)?;

        // Then sync entries. Entries have tag links and annotations embedded.
        let mut filter = EntriesFilter::default();
        let since = self.db.get_last_sync()??;
        filter.since = since.timestamp() as u64;

        let entries = client.get_entries_with_filter(filter)?;

        // NOTE: changing tags on an entry touches updated_at. To add a tag
        // locally, create a new tag object with an arbitrary `slug` and `id`,
        // add it to the Entry object, and save to db. taglinks and tags table
        // will be updated in next sync.

        // NOTE: this will not get annotations where annotation has been updated recently but not
        // the associated entry
        for entry in entries.into_iter() {
            // first check if existing entry with same id
            if let Some(saved_entry) = self.db.get_entry(entry.id.as_u32())? {
                match saved_entry.updated_at.cmp(&entry.updated_at) {
                    Less => {
                        // saved entry is older than pulled version; overwrite
                        self.pull_entry(&mut client, entry)?;
                    }
                    Equal => {
                        // noop; already synced and same version
                    }
                    Greater => {
                        // local entry is newer, push to server
                        client.update_entry(saved_entry.id, &(&saved_entry).into())?;
                        self.db.set_sync_entry(saved_entry)?;
                    }
                }
            } else {
                self.pull_entry(&mut client, entry)?;
            }
        }

        for entry in self.db.get_unsynced_entries()?.into_iter() {
            // push, since it's still marked as unsynced and has been updated
            // since previous sync
            if entry.updated_at > since {
                client.update_entry(entry.id, &(&entry).into())?;
                self.db.set_sync_entry(entry)?;
            }
            // TODO: otherwise, what does this mean?
        }


        for ann in self.db.get_unsynced_annotations()?.into_iter() {
            // push, since it's still marked as unsynced and has been updated
            // since previous sync
            if ann.updated_at > since {
                client.update_annotation(&ann)?;
                self.db.set_sync_annotation(ann)?;
            }
            // TODO: if not, what does this mean?
        }

        // finally push new things to the server
        // TODO: add new tables to track these and code to push

        // Touch the last sync time ready for next sync.
        // This must be done last to ensure the sync has successfully completed.
        self.db.touch_last_sync()?;

        Ok(())
    }

    /// save an entry to the database where the entry has been determined to be
    /// newer than any in the database, but still need to do bidirectional sync
    /// for associated annotations and tags
    fn pull_entry(&self, client: &mut Client, entry: Entry) -> Fallible<()> {
        self.db.save_entry(&entry, true)?;

        if let Some(ref anns) = entry.annotations {
            for ann in anns {
                self.sync_annotation(client, ann, &entry);
            }
        }

        // pull tags
        self.db.drop_tag_links_for_entry(&entry)?;
        for tag in entry.tags.iter() {
            self.db.save_tag_link(&entry, tag)?;
        }

        Ok(())
    }

    /// sync an annotation given an annotation from the server.
    fn sync_annotation<T: Into<ID>>(
        &self,
        client: &mut Client,
        ann: &Annotation,
        entry_id: T,
    ) -> Fallible<()> {
        let entry_id = entry_id.into().as_u32();
        if let Some(saved_ann) = self.db.get_annotation(ann.id.as_u32())? {
            match saved_ann.updated_at.cmp(&ann.updated_at) {
                Less => {
                    // saved annotation is older than pulled version; overwrite
                    self.db.save_annotation(ann, entry_id, true)?;
                }
                Equal => {
                    // noop; already synced and same version
                }
                Greater => {
                    // local annotation is newer, push to server
                    client.update_annotation(&saved_ann)?;
                    self.db.set_sync_annotation(saved_ann)?;
                }
            }
        } else {
            self.db.save_annotation(ann, entry_id, true)?;
        }

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
