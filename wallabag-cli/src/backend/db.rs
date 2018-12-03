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

    pub fn get_entry<T: Into<ID>>(&self, id: T) -> Fallible<Option<Entry>> {
        // TODO: for entry by id, set up so that returns None if not found
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"select id, content, created_at, domain_name, http_status,
            is_archived, is_public, is_starred, language, mimetype, origin_url,
            preview_picture, published_at, published_by, reading_time,
            starred_at, title, uid, updated_at, url, headers, user_email,
            user_id, user_name from entries where id = ?"#,
        )?;

        let mut results = stmt.query_map(&[&id.into().as_u32()], |row| -> Fallible<Entry> {
            Ok(Entry {
                id: ID(row.get(0)),
                content: row.get(1),
                created_at: DateTime::parse_from_rfc3339(&row.get::<usize, String>(2))
                    .map(|dt| dt.with_timezone(&Utc))?,
                domain_name: row.get(3),
                http_status: row.get(4),
                is_archived: row.get(5),
                is_public: row.get(6),
                is_starred: row.get(7),
                language: row.get(8),
                mimetype: row.get(9),
                origin_url: row.get(10),
                preview_picture: row.get(11),
                published_at: extract_result(row.get::<usize, Option<String>>(12).map(|row| {
                    DateTime::parse_from_rfc3339(&row).map(|dt| dt.with_timezone(&Utc))
                }))?,
                published_by: extract_result(
                    row.get::<usize, Option<String>>(13)
                        .map(|row| serde_json::from_str::<Vec<String>>(&row)),
                )?,
                reading_time: row.get(14),
                starred_at: extract_result(row.get::<usize, Option<String>>(15).map(|row| {
                    DateTime::parse_from_rfc3339(&row).map(|dt| dt.with_timezone(&Utc))
                }))?,
                title: row.get(16),
                uid: row.get(17),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<usize, String>(18))
                    .map(|dt| dt.with_timezone(&Utc))?,
                url: row.get(19),
                headers: extract_result(
                    row.get::<usize, Option<String>>(20)
                        .map(|row| serde_json::from_str::<Vec<String>>(&row)),
                )?,
                user_email: row.get(21),
                user_id: ID(row.get(22)),
                user_name: row.get(23),
                // TODO: load annotations and tags here?
                annotations: None,
                tags: vec![],
            })
        })?;

        match results.next() {
            Some(thing) => match thing {
                Ok(err_entry) => Ok(Some(err_entry?)),
                Err(e) => Err(e.into()),
            },
            None => Ok(None),
        }
    }

    /// get the last time a sync was performed. used for optimization by the
    /// backend
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
    pub fn touch_last_sync(&self) -> SQLResult<()> {
        self.conn()?
            .execute(
                "UPDATE config SET last_sync = ? where id = 1",
                &[&chrono::offset::Utc::now().to_rfc3339()],
            )
            .map(|i| ())
    }

    /// Save an entry to the database. If not existing (by id), it will be
    /// inserted; if existing, it will replace the old value.
    pub fn save_entry(&self, entry: &Entry, synced: bool) -> Fallible<()> {
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
                &extract_result(match entry.published_by {
                    None => None,
                    Some(ref x) => Some(serde_json::to_string(x)),
                })?,
                &entry.reading_time,
                &entry.starred_at.map(|dt| dt.to_rfc3339()),
                &entry.title,
                &entry.uid,
                &entry.updated_at.to_rfc3339(),
                &entry.url,
                &extract_result(match entry.headers {
                    None => None,
                    Some(ref x) => Some(serde_json::to_string(x)),
                })?,
                &entry.user_email,
                &entry.user_id.as_u32(),
                &entry.user_name,
                &synced,
            ],
        )
        .map(|i| ())?;

        Ok(())
    }

    /// Save an annotation to the database. If not existing (by id), it will be
    /// inserted; if existing, it will replace the old value.
    pub fn save_annotation<T: Into<ID>>(
        &self,
        ann: &Annotation,
        entry_id: T,
        synced: bool,
    ) -> Fallible<()> {
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
                &serde_json::ser::to_string(&ann.ranges)?,
                &ann.text,
                &ann.updated_at.to_rfc3339(),
                &ann.quote,
                &ann.user,
                &*entry_id.into(),
                &synced,
            ],
        )?;

        Ok(())
    }

    /// `synced` is whether or not any changes have been synced to the server. A
    /// sync method in the backend should set this to true, but other local
    /// actions should set this to false.
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

fn extract_result<T, U>(x: Option<Result<T, U>>) -> Result<Option<T>, U> {
    match x {
        Some(Ok(t)) => Ok(Some(t)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}
