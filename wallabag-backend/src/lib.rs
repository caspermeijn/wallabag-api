// backend module that links the api client to a local database and provides
// sync

mod db;

use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use failure::Fallible;
use serde::{Deserialize, Serialize};
use serde_json;

use log::debug;

pub use wallabag_api::types;

use wallabag_api::types::{
    Annotation, Config as APIConfig, Entries, EntriesFilter, Entry, NewEntry, Tags, ID,
};
use wallabag_api::Client;

use self::db::{NewUrl, DB};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum StringOrCmd {
    S(String),
    Cmd { cmd: Vec<String> },
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    #[serde(default = "default_db_file")]
    pub db_file: PathBuf,
    pub client_id: StringOrCmd,
    pub client_secret: StringOrCmd,
    pub username: StringOrCmd,
    pub password: StringOrCmd,
    pub base_url: String,
}

fn default_db_file() -> PathBuf {
    "db.sqlite3".into()
}

#[derive(Debug)]
pub struct Backend {
    db: DB,
    client: Client,
}

#[derive(Debug)]
pub enum BackendError {
    EmptyCommand,
    FailedCommand,
}
impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for BackendError {}

/// Get a string from possibly a command to run. Used for evaluating config values which maybe be a
/// literal string, or could be a command to evaluate and use the output.
fn get_string(x: &StringOrCmd) -> Fallible<String> {
    match x {
        StringOrCmd::S(s) => Ok(s.clone()),
        StringOrCmd::Cmd { cmd } => {
            debug!("Running command {:?}", cmd);
            let mut args = cmd.iter();

            let executable = args.next().ok_or(BackendError::EmptyCommand)?;

            let output = Command::new(executable).args(args).output()?;
            if output.status.success() {
                let output = String::from_utf8(output.stdout)?;
                Ok(output)
            } else {
                debug!("Command failed with exit status {:?}", output.status.code());
                Err(BackendError::FailedCommand)?
            }
        }
    }
}

// TODO: add method to backend to get a reference to the client for lower level stuff if required
impl Backend {
    pub fn new_with_conf(conf: Config) -> Fallible<Self> {
        let backend = Self {
            db: DB::new(conf.db_file),
            client: Client::new(APIConfig {
                client_id: get_string(&conf.client_id)?,
                client_secret: get_string(&conf.client_secret)?,
                username: get_string(&conf.username)?,
                password: get_string(&conf.password)?,
                base_url: conf.base_url,
            }),
        };
        Ok(backend)
    }

    pub fn reset(&self) -> Fallible<()> {
        self.db.reset()?;
        debug!("DB reset success");

        Ok(())
    }

    pub fn init(&self) -> Fallible<()> {
        self.db.init()?;
        debug!("DB init success");

        Ok(())
    }

    /// Get a Vec of tags from the db.
    pub fn get_entry<T: Into<ID>>(&self, id: T) -> Fallible<Option<Entry>> {
        self.db.get_entry(id)
    }

    /// Get a Vec of tags from the db.
    pub fn tags(&self) -> Fallible<Tags> {
        self.db.get_tags()
    }

    /// Export all the data (all the entries for now - TODO: decide if other data should be
    /// exported - if so, return `serde_json::Value`)
    pub fn export(&self) -> Fallible<Entries> {
        let mut entries = self.db.get_all_entries()?;
        let mut entries_map: HashMap<ID, &mut Entry> = HashMap::with_capacity(entries.len());

        for entry in entries.iter_mut() {
            entries_map.insert(entry.id, entry);
        }

        let annotations = self.db.get_annotations()?;
        for (entry_id, ann) in annotations {
            match entries_map.get_mut(&entry_id) {
                None => {
                    // Should never happen unless db is broken
                }
                Some(ref mut entry) => {
                    match entry.annotations {
                        Some(ref mut anns) => {
                            anns.push(ann);
                        }
                        None => {
                            entry.annotations = Some(vec![ann]);
                        }
                    }
                }
            }

        }

        Ok(entries)
    }

    /// Get a Vec of entries from the db.
    pub fn entries(&self) -> Fallible<Entries> {
        self.db.get_all_entries()
    }

    /// Add a new url and attempts to upload and create entry immediatedly. Fails if network
    /// connection down.
    pub fn add_url_online<T: AsRef<str>>(&mut self, url: T) -> Fallible<()> {
        let url = reqwest::Url::parse(url.as_ref())?;

        let new_entry = NewEntry::new_with_url(url.into_string());
        let entry = self.client.create_entry(&new_entry)?;

        self.pull_entry(&entry)
    }

    /// Add a new url. Does not attempt to upload immediately.
    pub fn add_url<T: AsRef<str>>(&self, url: T) -> Fallible<()> {
        let url = reqwest::Url::parse(url.as_ref())?;
        self.db.add_new_url(url.as_str())
    }

    /// Full sync. Can be slow if many articles. This will sync everything,
    /// including things that can't be synced with a quick/normal sync (eg. server-side deleted
    /// items)
    ///
    /// For entries and annotations existing in the database, object with latest
    /// updated_at value wins.
    ///
    /// Tags are tied to Entries, even though the tags will also have seperate db tables dedicated
    /// to them. An updated tag should mean an updated updated_at field in an entry.
    ///
    /// NOTE: changing tags on an entry touches updated_at. To add a tag
    /// locally, create a new tag object with an arbitrary `slug` and `id`,
    /// add it to the Entry object, and save to db. taglinks and tags table
    /// will be updated in next sync.
    ///
    /// NOTE: changing an annotation does not update entry updated_at
    pub fn full_sync(&mut self) -> Fallible<()> {
        // sync local deletes first otherwise entries will be re-created locally...
        self.sync_local_deletes()?;

        // get _all_ entries on the server
        let server_entries = self.client.get_entries()?;

        // used when syncing up locally updated entries/annotations to avoid syncing twice
        let seen_entries: HashSet<ID> = server_entries.iter().map(|e| e.id).collect();
        let tmp_empty_vec = vec![];
        let seen_annotations: HashSet<ID> = server_entries
            .iter()
            .flat_map(|e| {
                e.annotations
                    .as_ref()
                    .unwrap_or(&tmp_empty_vec)
                    .iter()
                    .map(|a| a.id)
            })
            .collect();

        for remote_entry in server_entries {
            // first check if existing entry with same id
            if let Some(saved_entry) = self.db.get_entry(remote_entry.id.as_int())? {
                match Ord::cmp(&saved_entry.updated_at, &remote_entry.updated_at) {
                    Less => {
                        // saved entry is older than pulled version; overwrite
                        self.pull_entry(&remote_entry)?;
                    }
                    Equal => {
                        // already synced and same version
                        // still need to sync annotations though
                        if let Some(ref anns) = remote_entry.annotations {
                            for ann in anns {
                                self.sync_annotation(ann, &remote_entry)?;
                            }
                        }
                    }
                    Greater => {
                        // local entry is newer, push to server
                        let updated_entry = self
                            .client
                            .update_entry(saved_entry.id, &(&saved_entry).into())?;
                        // run pull entry on the entry returned to sync any new tags
                        self.pull_entry(&updated_entry)?;
                    }
                }
            } else {
                self.pull_entry(&remote_entry)?;
            }
        }

        // delete all local entries that have been deleted on the server
        let local_entries: HashSet<ID> = self.db.get_all_entry_ids()?;
        let remotely_deleted_entries = HashSet::difference(&local_entries, &seen_entries);
        for entry_id in remotely_deleted_entries {
            self.db.delete_entry(*entry_id)?;
        }

        // delete all local annotations that have been deleted on the server
        let local_anns: HashSet<ID> = self.db.get_all_annotation_ids()?;
        let remotely_deleted_anns = HashSet::difference(&local_anns, &seen_annotations);
        for ann_id in remotely_deleted_anns {
            self.db.delete_annotation(*ann_id)?;
        }

        // finally push new things to the server
        for NewUrl { id, url } in self.db.get_new_urls()? {
            let new_entry = NewEntry::new_with_url(url);
            let entry = self.client.create_entry(&new_entry)?;
            self.pull_entry(&entry)?;
            self.db.remove_new_url(id)?;
        }

        for (entry_id, new_ann_id, new_ann) in self.db.get_new_annotations()? {
            let ann = self.client.create_annotation(entry_id, &new_ann)?;
            self.db.save_annotation(&ann, entry_id)?;
            self.db.remove_new_annotation(new_ann_id)?;
        }

        // last of all drop tags with no tag_links
        self.db.delete_unused_tags()?;

        // Touch the last sync time ready for next sync.
        // This must be done last to ensure the sync has successfully completed.
        self.db.touch_last_sync()?;

        Ok(())
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
    ///   with entries updated since previous sync. (ie. recently updated annotations on
    ///   non-recently updated entries)
    ///
    /// TODO: ignore errors relating to actions that have already been done - eg. 404 error on
    /// client delete entry.
    pub fn sync(&mut self) -> Fallible<()> {
        self.sync_local_deletes()?;

        // Sync entries recently updated server-side. Entries have tag links and annotations embedded.
        let mut filter = EntriesFilter::default();
        let since = self.db.get_last_sync()?;
        filter.since = since.timestamp();
        let entries = self.client.get_entries_with_filter(&filter)?;

        // used when syncing up locally updated entries/annotations to avoid syncing twice
        let seen_entries: HashSet<ID> = entries.iter().map(|e| e.id).collect();
        let tmp_empty_vec = vec![];
        let seen_annotations: HashSet<ID> = entries
            .iter()
            .flat_map(|e| {
                e.annotations
                    .as_ref()
                    .unwrap_or(&tmp_empty_vec)
                    .iter()
                    .map(|a| a.id)
            })
            .collect();

        // sync recently updated entries
        for remote_entry in entries {
            // first check if existing entry with same id
            if let Some(saved_entry) = self.db.get_entry(remote_entry.id.as_int())? {
                match Ord::cmp(&saved_entry.updated_at, &remote_entry.updated_at) {
                    Less => {
                        // saved entry is older than pulled version; overwrite
                        self.pull_entry(&remote_entry)?;
                    }
                    Equal => {
                        // already synced and same version
                        // still need to sync annotations though
                        if let Some(ref anns) = remote_entry.annotations {
                            for ann in anns {
                                self.sync_annotation(ann, &remote_entry)?;
                            }
                        }
                    }
                    Greater => {
                        // local entry is newer, push to server
                        let updated_entry = self
                            .client
                            .update_entry(saved_entry.id, &(&saved_entry).into())?;
                        // run pull entry on the entry returned to sync any new tags and
                        // update annotations
                        self.pull_entry(&updated_entry)?;
                    }
                }
            } else {
                self.pull_entry(&remote_entry)?;
            }
        }

        // Update all locally-recently-updated entries and annotations that weren't touched
        // previously.
        for entry in self.db.get_entries_since(since)? {
            if !seen_entries.contains(&entry.id) {
                self.client.update_entry(entry.id, &(&entry).into())?;
            }
        }

        for ann in self.db.get_annotations_since(since)? {
            if !seen_annotations.contains(&ann.id) {
                self.client.update_annotation(&ann)?;
            }
        }

        // finally push new things to the server
        for NewUrl { id, url } in self.db.get_new_urls()? {
            let new_entry = NewEntry::new_with_url(url);
            let entry = self.client.create_entry(&new_entry)?;
            self.pull_entry(&entry)?;
            self.db.remove_new_url(id)?;
        }

        for (entry_id, new_ann_id, new_ann) in self.db.get_new_annotations()? {
            let ann = self.client.create_annotation(entry_id, &new_ann)?;
            self.db.save_annotation(&ann, entry_id)?;
            self.db.remove_new_annotation(new_ann_id)?;
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
    fn pull_entry(&mut self, entry: &Entry) -> Fallible<()> {
        self.db.save_entry(entry)?;

        if let Some(ref anns) = entry.annotations {
            for ann in anns {
                self.sync_annotation(ann, entry)?;
            }
        }

        // rebuild tag links
        self.db.drop_tag_links_for_entry(entry)?;
        for tag in &entry.tags {
            self.db.save_tag(&tag)?;
            self.db.save_tag_link(entry, &tag)?;
        }

        Ok(())
    }

    /// Push up all local delete actions.
    fn sync_local_deletes(&mut self) -> Fallible<()> {
        // Track and sync client-side deletes. This needs to be done before pulling
        // entries/annotations otherwise they will simply be re-created.
        // Delete annotation deletes before entry deletes to avoid 404s.
        // TODO: ignore not found errors here
        for annotation_id in self.db.get_annotation_deletes()? {
            self.client.delete_annotation(annotation_id)?;
            self.db.remove_delete_annotation(annotation_id)?;
        }
        for entry_id in self.db.get_entry_deletes()? {
            self.client.delete_entry(entry_id)?;
            self.db.remove_delete_entry(entry_id)?;
        }

        Ok(())
    }

    /// sync an annotation given an annotation from the server.
    fn sync_annotation<T: Into<ID>>(&mut self, ann: &Annotation, entry_id: T) -> Fallible<()> {
        let entry_id = entry_id.into().as_int();
        if let Some(saved_ann) = self.db.get_annotation(ann.id.as_int())? {
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
                    let updated_ann = self.client.update_annotation(&saved_ann)?;
                    self.db.save_annotation(&updated_ann, entry_id)?;
                }
            }
        } else {
            self.db.save_annotation(ann, entry_id)?;
        }

        Ok(())
    }
}
