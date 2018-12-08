// backend module that links the api client to a local database and provides
// sync

mod db;

use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::result::Result;

use chrono::{DateTime, Utc};
use failure::Fallible;
use rusqlite::types::ToSql;
use rusqlite::Result as SQLResult;
use rusqlite::{Connection, NO_PARAMS};
use serde_json;

use wallabag_api::types::{Annotation, Config, EntriesFilter, Entry, NewEntry, Tag, Tags, ID};
use wallabag_api::Client;

use self::db::{DbNewUrl, DB};

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

    /// Add a new url and attempts to upload and create entry immediatedly. Fails if network
    /// connection down.
    pub fn add_url_online<T: AsRef<str>>(&self, url: T) -> Fallible<()> {
        let config = Config {
            client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
            client_secret: env::var("WALLABAG_CLIENT_SECRET")
                .expect("WALLABAG_CLIENT_SECRET not set"),
            username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
            password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
            base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
        };

        let mut client = Client::new(config);

        let url = reqwest::Url::parse(url.as_ref())?;

        let new_entry = NewEntry::new_with_url(url.into_string());
        let entry = client.create_entry(&new_entry)?;

        self.pull_entry(&mut client, entry)
    }

    /// Add a new url. Does not attempt to upload immediately.
    pub fn add_url<T: AsRef<str>>(&self, url: T) -> Fallible<()> {
        let url = reqwest::Url::parse(url.as_ref())?;
        self.db.add_new_url(url.as_str())
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

        // Sync entries recently updated server-side. Entries have tag links and annotations embedded.
        let mut filter = EntriesFilter::default();
        let since = self.db.get_last_sync()??;
        filter.since = since.timestamp() as u64;
        let entries = client.get_entries_with_filter(filter)?;

        // used when syncing up locally updated entries/annotations to avoid syncing twice
        let seen_entries: HashSet<ID> = entries.iter().map(|e| e.id).collect();
        let tmp_empty_vec = vec![];
        let seen_annotations: HashSet<ID> = entries
            .iter()
            .map(|e| {
                e.annotations
                    .as_ref()
                    .unwrap_or(&tmp_empty_vec)
                    .iter()
                    .map(|a| a.id)
            })
            .flatten()
            .collect();

        // NOTE: changing tags on an entry touches updated_at. To add a tag
        // locally, create a new tag object with an arbitrary `slug` and `id`,
        // add it to the Entry object, and save to db. taglinks and tags table
        // will be updated in next sync.
        // NOTE: changing an annotation does not update entry updated_at

        // NOTE: this will not get annotations where annotation has been updated recently but not
        // the associated entry
        for entry in entries.into_iter() {
            // first check if existing entry with same id
            if let Some(saved_entry) = self.db.get_entry(entry.id.as_u32())? {
                match Ord::cmp(&saved_entry.updated_at, &entry.updated_at) {
                    Less => {
                        // saved entry is older than pulled version; overwrite
                        self.pull_entry(&mut client, entry)?;
                    }
                    Equal => {
                        // noop; already synced and same version
                    }
                    Greater => {
                        // local entry is newer, push to server
                        let updated_entry =
                            client.update_entry(saved_entry.id, &(&saved_entry).into())?;
                        // run pull entry on the entry returned to sync any new tags
                        self.pull_entry(&mut client, updated_entry)?;
                    }
                }
            } else {
                self.pull_entry(&mut client, entry)?;
            }
        }

        // Update all locally-recently-updated entries and annotations that weren't touched
        // previously.
        for entry in self.db.get_entries_since(since)?.into_iter() {
            if !seen_entries.contains(&entry.id) {
                client.update_entry(entry.id, &(&entry).into())?;
            }
        }

        for ann in self.db.get_annotations_since(since)?.into_iter() {
            if !seen_annotations.contains(&ann.id) {
                client.update_annotation(&ann)?;
            }
        }

        // track and sync client-side deletes.
        for entry_id in self.db.get_entry_deletes()? {
            client.delete_entry(entry_id)?;
            self.db.remove_delete_entry(entry_id)?;
        }
        for annotation_id in self.db.get_annotation_deletes()? {
            client.delete_annotation(annotation_id)?;
            self.db.remove_delete_annotation(annotation_id)?;
        }

        // finally push new things to the server
        for DbNewUrl { id: id, url: url } in self.db.get_new_urls()? {
            let new_entry = NewEntry::new_with_url(url);
            let entry = client.create_entry(&new_entry)?;
            self.pull_entry(&mut client, entry)?;
            self.db.remove_new_url(id)?;
        }

        for (entry_id, new_ann_id, new_ann) in self.db.get_new_annotations()? {
            let ann = client.create_annotation(entry_id, new_ann)?;
            self.db.save_annotation(&ann, entry_id)?;
            self.db.remove_new_annotation(new_ann_id);
        }

        // last of all drop tags with no tag_links
        self.db.delete_unused_tags()?;

        // Touch the last sync time ready for next sync.
        // This must be done last to ensure the sync has successfully completed.
        self.db.touch_last_sync()?;

        Ok(())
    }

    /// save an entry to the database where the entry has been determined to be
    /// newer than any in the database, but still need to do bidirectional sync
    /// for associated annotations and tags
    fn pull_entry(&self, client: &mut Client, entry: Entry) -> Fallible<()> {
        self.db.save_entry(&entry)?;

        if let Some(ref anns) = entry.annotations {
            for ann in anns {
                self.sync_annotation(client, ann, &entry);
            }
        }

        // rebuild tag links
        self.db.drop_tag_links_for_entry(&entry)?;
        for tag in entry.tags.iter() {
            self.db.save_tag(tag)?;
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
            match Ord::cmp(&saved_ann.updated_at, &ann.updated_at) {
                Less => {
                    // saved annotation is older than pulled version; overwrite
                    self.db.save_annotation(ann, entry_id)?;
                }
                Equal => {
                    // noop; already synced and same version
                }
                Greater => {
                    // local annotation is newer, push to server
                    let updated_ann = client.update_annotation(&saved_ann)?;
                    self.db.save_annotation(&updated_ann, entry_id)?;
                }
            }
        } else {
            self.db.save_annotation(ann, entry_id)?;
        }

        Ok(())
    }
}
