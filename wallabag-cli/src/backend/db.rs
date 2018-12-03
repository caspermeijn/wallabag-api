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

pub struct DB {
    db_file: PathBuf,
}

impl DB {
    pub fn new<T: Into<PathBuf>>(db_file: T) -> Self {
        DB {
            db_file: db_file.into(),
        }
    }

    fn conn(&self) -> SQLResult<Connection> {
        Connection::open(&self.db_file)
    }

    /// Reset the database to a clean state. Database file will be created if
    /// not existing.
    ///
    /// Warning: this means all data in db tables will be lost.
    pub fn reset(&self) -> SQLResult<()> {
        if self.db_file.exists() {
            self.down()?;
        }

        self.init()
    }

    /// Initiates the database if the database file doesn't exist. If the
    /// database file does exist but is in a broken state, then you should
    /// manually delete the file and start again.
    pub fn init(&self) -> SQLResult<()> {
        if self.db_file.exists() {
            return Ok(());
        }

        self.up()
    }

    /// Create tables in the database, creating the
    pub fn up(&self) -> SQLResult<()> {
        let query = include_str!("../../sql/up.sql");
        self.conn()?.execute_batch(query)
    }

    pub fn down(&self) -> SQLResult<()> {
        let query = include_str!("../../sql/down.sql");
        self.conn()?.execute_batch(query)
    }

    pub fn get_entry<T: Into<ID>>(&self, id: T) -> SQLResult<Option<Entry>> {
        // TODO: for entry by id, set up so that returns None if not found
        Ok(None)
    }

    pub fn get_last_sync(&self) -> SQLResult<chrono::ParseResult<DateTime<Utc>>> {
        self.conn()?.query_row(
            "SELECT (last_sync) from config where id = 1",
            NO_PARAMS,
            |row| {
                DateTime::parse_from_rfc3339(&row.get::<usize, String>(0))
                    .map(|dt| dt.with_timezone(&Utc))
            },
        )
    }

    /// Sets the last sync time to now.
    pub fn update_last_sync(&self) -> SQLResult<()> {
        self.conn()?
            .execute(
                "UPDATE config SET last_sync = ? where id = 1",
                &[&chrono::offset::Utc::now().to_rfc3339()],
            )
            .map(|i| ())
    }

    /// Save an entry to the database. If not existing (by id), it will be
    /// inserted; if existing, it will replace the old value.
    pub fn save_entry(&self, entry: &Entry, synced: bool) -> SQLResult<()> {
        let conn = self.conn()?;

        conn.execute(
            r#"INSERT OR REPLACE INTO entries
            (id, content, created_at, domain_name, http_status, is_archived,
            is_public, is_starred, language, mimetype, origin_url,
            preview_picture, published_at, published_by, reading_time,
            starred_at, title, uid, updated_at, url, headers, user_email,
            user_id, user_name, synced) VALUES
             (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            &[
                &*entry.id as &ToSql,
                &entry.content,
                &entry.created_at.to_rfc3339(),
                &entry.domain_name,
                &entry.http_status,
                &entry.is_archived,
                &entry.is_public,
                &entry.is_starred,
                &entry.language,
                &entry.mimetype,
                &entry.origin_url,
                &entry.preview_picture,
                &entry.published_at.map(|dt| dt.to_rfc3339()),
                &serde_json::ser::to_string(&entry.published_by).unwrap(),
                &entry.reading_time,
                &entry.starred_at.map(|dt| dt.to_rfc3339()),
                &entry.title,
                &entry.uid,
                &entry.updated_at.to_rfc3339(),
                &entry.url,
                &serde_json::ser::to_string(&entry.headers).unwrap(),
                &entry.user_email,
                &entry.user_id.as_u32(),
                &entry.user_name,
                &synced,
            ],
        )
        .map(|i| ())
    }

    /// Save an annotation to the database. If not existing (by id), it will be
    /// inserted; if existing, it will replace the old value.
    pub fn save_annotation<T: Into<ID>>(
        &self,
        ann: &Annotation,
        entry_id: T,
        synced: bool,
    ) -> SQLResult<()> {
        let conn = self.conn()?;

        conn.execute(
            r#"INSERT OR REPLACE INTO annotations
            (id, annotator_schema_version, created_at, ranges, text, updated_at,
            quote, user, entry_id, synced) VALUES
             (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            &[
                &*ann.id as &ToSql,
                &ann.annotator_schema_version,
                &ann.created_at.to_rfc3339(),
                &serde_json::ser::to_string(&ann.ranges).unwrap(),
                &ann.text,
                &ann.updated_at.to_rfc3339(),
                &ann.quote,
                &ann.user,
                &*entry_id.into(),
                &synced,
            ],
        )
        .map(|i| ())
    }

    pub fn save_tag(&self, tag: &Tag, synced: bool) -> SQLResult<()> {
        self.conn()?
            .execute(
                "insert or replace into tags (id, label, slug, synced) values (?1, ?2, ?3, ?4)",
                &[
                    &tag.id.to_string() as &ToSql,
                    &tag.label,
                    &tag.slug,
                    &synced,
                ],
            )
            .map(|i| ())
    }

    pub fn load_tags(&self) -> SQLResult<()> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("select id, label, slug from tags")?;

        let results = stmt.query_map(NO_PARAMS, |row| Tag {
            id: ID(row.get(0)),
            label: row.get(1),
            slug: row.get(2),
        })?;

        for tag in results {
            println!("{}", tag?.label);
        }

        Ok(())
    }
}
