use std::path::PathBuf;

use chrono::{DateTime, Utc};
use failure::Fallible;
use rusqlite::types::ToSql;
use rusqlite::Result as SQLResult;
use rusqlite::{Connection, Row, NO_PARAMS};
use serde_json;

use log::debug;

use wallabag_api::types::{
    Annotation, Annotations, Entries, EntriesFilter, Entry, NewAnnotation, Range, Tag,
    Tags, ID,
};

pub struct DbNewUrl {
    pub id: i64,
    pub url: String,
}

pub struct DB {
    db_file: PathBuf,
}

impl DB {
    pub fn new<T: Into<PathBuf>>(db_file: T) -> Self {
        DB {
            db_file: db_file.into(),
        }
    }

    /// opens a new connection to the db, turns on foreign keys support, and returns the connection
    fn conn(&self) -> SQLResult<Connection> {
        let conn = Connection::open(&self.db_file)?;
        conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;
        Ok(conn)
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
        // TODO: return err if exists with warning
        if self.db_file.exists() {
            return Ok(());
        }

        self.up()
    }

    /// Create tables in the database, creating the
    pub fn up(&self) -> SQLResult<()> {
        let query = include_str!("../sql/up.sql");
        self.conn()?.execute_batch(query)
    }

    pub fn down(&self) -> SQLResult<()> {
        let query = include_str!("../sql/down.sql");
        self.conn()?.execute_batch(query)
    }

    // get an annotation from the db by id
    pub fn get_annotation<T: Into<ID>>(&self, id: T) -> Fallible<Option<Annotation>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"select id, annotator_schema_version, created_at, ranges, text,
            updated_at, quote, user from annotations where id = ?"#,
        )?;

        let mut results = stmt.query_map(&[&id.into().as_u32()], row_to_ann)?;

        match results.next() {
            Some(thing) => match thing {
                Ok(err_ann) => Ok(Some(err_ann?)),
                Err(e) => Err(e.into()),
            },
            None => Ok(None),
        }
    }

    pub fn get_annotations_since(&self, since: DateTime<Utc>) -> Fallible<Annotations> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"select id, annotator_schema_version, created_at, ranges, text,
            updated_at, quote, user from annotations where updated_at >= ?"#,
        )?;

        let mut results = stmt.query_map(&[since.to_rfc3339()], row_to_ann)?;

        results.flatten().collect::<Fallible<Annotations>>()
    }

    pub fn get_new_annotations(&self) -> Fallible<Vec<(ID, i64, NewAnnotation)>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt =
            conn.prepare("SELECT id, quote, ranges, text, entry_id from new_annotations")?;

        let mut results = stmt.query_map(NO_PARAMS, |row| {
            Ok((
                ID(row.get(4)),
                row.get(0),
                NewAnnotation {
                    quote: row.get(1),
                    ranges: serde_json::from_str::<Vec<Range>>(&row.get::<usize, String>(2))?,
                    text: row.get(3),
                },
            ))
        })?;

        // TODO: does flatten hide Err because it just doesn't iterate over them?
        results
            .flatten()
            .collect::<Fallible<Vec<(ID, i64, NewAnnotation)>>>()
    }

    /// Remove a new annotation entry from db, signifying that it has been synced.
    pub fn remove_new_annotation(&self, id: i64) -> Fallible<()> {
        self.conn()?
            .execute("DELETE FROM new_annotations WHERE id = ?", &[&id])?;

        Ok(())
    }

    pub fn get_entries_since(&self, since: DateTime<Utc>) -> Fallible<Entries> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"SELECT id, content, created_at, domain_name, http_status,
            is_archived, is_public, is_starred, language, mimetype, origin_url,
            preview_picture, published_at, published_by, reading_time,
            starred_at, title, uid, updated_at, url, headers, user_email,
            user_id, user_name, tags from entries WHERE updated_at >= ?"#,
        )?;

        let mut results = stmt.query_map(&[since.to_rfc3339()], row_to_entry)?;

        results.flatten().collect::<Fallible<Entries>>()
    }

    pub fn get_new_urls(&self) -> Fallible<Vec<DbNewUrl>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("SELECT id, url FROM new_urls")?;
        let mut results = stmt.query_map(NO_PARAMS, |row| DbNewUrl {
            id: row.get(0),
            url: row.get(1),
        })?;

        Ok(results.collect::<SQLResult<Vec<DbNewUrl>>>()?)
    }

    /// Remove a new url entry from the db, signifying that it has been successfully synced.
    pub fn remove_new_url(&self, id: i64) -> Fallible<()> {
        self.conn()?
            .execute("DELETE FROM new_urls WHERE id = ?", &[&id])?;

        Ok(())
    }

    /// Add a new url to be uploaded next sync.
    pub fn add_new_url(&self, url: &str) -> Fallible<()> {
        self.conn()?
            .execute("INSERT OR REPLACE INTO new_urls (url) VALUES (?)", &[url])?;

        Ok(())
    }

    pub fn get_annotation_deletes(&self) -> Fallible<Vec<ID>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("SELECT id FROM deleted_annotations")?;
        let mut results = stmt.query_map(NO_PARAMS, |row| ID(row.get(0)))?;

        Ok(results.collect::<SQLResult<Vec<ID>>>()?)
    }

    /// Remove an annotation from the delteed entries table. This marks a local delete as synced.
    pub fn remove_delete_annotation<T: Into<ID>>(&self, annotation_id: T) -> Fallible<()> {
        self.conn()?.execute(
            "DELETE FROM deleted_annotations WHERE id = ?",
            &[&annotation_id.into().as_u32()],
        )?;

        Ok(())
    }

    pub fn get_entry_deletes(&self) -> Fallible<Vec<ID>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("SELECT id FROM deleted_entries")?;
        let mut results = stmt.query_map(NO_PARAMS, |row| ID(row.get(0)))?;

        Ok(results.collect::<SQLResult<Vec<ID>>>()?)
    }

    /// Remove an entry from the delteed entries table. This marks a local delete as synced.
    pub fn remove_delete_entry<T: Into<ID>>(&self, entry_id: T) -> Fallible<()> {
        self.conn()?.execute(
            "DELETE FROM deleted_entries WHERE id = ?",
            &[&entry_id.into().as_u32()],
        )?;

        Ok(())
    }

    pub fn get_entry<T: Into<ID>>(&self, id: T) -> Fallible<Option<Entry>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"SELECT id, content, created_at, domain_name, http_status,
            is_archived, is_public, is_starred, language, mimetype, origin_url,
            preview_picture, published_at, published_by, reading_time,
            starred_at, title, uid, updated_at, url, headers, user_email,
            user_id, user_name, tags FROM entries WHERE id = ?"#,
        )?;

        let mut results = stmt.query_map(&[&id.into().as_u32()], row_to_entry)?;

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
            "SELECT (last_sync) FROM config WHERE id = 1",
            NO_PARAMS,
            |row| {
                DateTime::parse_from_rfc3339(&row.get::<usize, String>(0))
                    .map(|dt| dt.with_timezone(&Utc))
            },
        )
    }

    /// Sets the last sync time to now.
    pub fn touch_last_sync(&self) -> Fallible<()> {
        self.conn()?.execute(
            "UPDATE config SET last_sync = ? WHERE id = 1",
            &[&chrono::offset::Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// self-explanatory
    pub fn drop_tag_links_for_entry<T: Into<ID>>(&self, entry_id: T) -> Fallible<()> {
        self.conn()?.execute(
            "DELETE FROM taglinks WHERE entry_id = ?",
            &[&entry_id.into().as_u32()],
        )?;

        Ok(())
    }

    /// Save an entry to the database. If not existing (by id), it will be
    /// inserted; if existing, it will replace the old value.
    pub fn save_entry(&self, entry: &Entry) -> Fallible<()> {
        let conn = self.conn()?;

        conn.execute(
            r#"INSERT OR REPLACE INTO entries
            (id, content, created_at, domain_name, http_status, is_archived,
            is_public, is_starred, language, mimetype, origin_url,
            preview_picture, published_at, published_by, reading_time,
            starred_at, title, uid, updated_at, url, headers, user_email,
            user_id, user_name, tags) VALUES
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
                &serde_json::to_string(&entry.tags)?,
            ],
        )?;

        Ok(())
    }

    /// Save an annotation to the database. If not existing (by id), it will be
    /// inserted; if existing, it will replace the old value.
    pub fn save_annotation<T: Into<ID>>(&self, ann: &Annotation, entry_id: T) -> Fallible<()> {
        let conn = self.conn()?;

        conn.execute(
            r#"INSERT OR REPLACE INTO annotations
            (id, annotator_schema_version, created_at, ranges, text, updated_at,
            quote, user, entry_id) VALUES
             (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
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
            ],
        )?;

        Ok(())
    }

    pub fn save_tag(&self, tag: &Tag) -> Fallible<()> {
        self.conn()?.execute(
            "INSERT OR REPLACE INTO tags (id, label, slug) VALUES (?1, ?2, ?3)",
            &[&tag.id.to_string() as &ToSql, &tag.label, &tag.slug],
        )?;

        Ok(())
    }

    pub fn save_tag_link<T: Into<ID>>(&self, entry_id: T, tag: &Tag) -> Fallible<()> {
        self.conn()?.execute(
            "INSERT OR REPLACE INTO taglinks (entry_id, tag_id) VALUES (?1, ?2)",
            &[&entry_id.into().as_u32() as &ToSql, &tag.id.as_u32()],
        )?;

        Ok(())
    }

    /// delete all tags with no taglinks entries
    pub fn delete_unused_tags(&self) -> Fallible<()> {
        self.conn()?
            .execute("DELETE FROM tags WHERE NOT EXISTS (SELECT taglinks.tag_id FROM taglinks WHERE tags.id = taglinks.tag_id)", NO_PARAMS)?;

        Ok(())
    }

    /// get all tags from the db
    pub fn get_tags(&self) -> Fallible<Tags> {
        let conn = self.conn()?;

        let sql = "SELECT id, label, slug FROM tags";
        log_sql(sql);
        let mut stmt = conn.prepare(sql)?;
        let results = stmt.query_map(NO_PARAMS, |row| -> Fallible<Tag> {
            Ok(Tag {
                id: ID(row.get_checked(0)?),
                label: row.get_checked(1)?,
                slug: row.get_checked(2)?,
            })
        })?;

        let mut tags = vec![];
        for maybe_maybe_tag in results {
            tags.push(maybe_maybe_tag??);
        }

        Ok(tags)
    }
}

/// A temporary function used until `Option::transpose` is stabilized. Transposes Option and Result
/// so we can do something like `extract_result(optionally_result())?;`
fn extract_result<T, U>(x: Option<Result<T, U>>) -> Result<Option<T>, U> {
    match x {
        Some(Ok(t)) => Ok(Some(t)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

/// Parse an Entry from a rusqlite::Row. NOTE: this will only work with the correct row
/// ordering. See the queries where this is used for a template.
fn row_to_entry<'r, 's, 't0>(row: &'r Row<'s, 't0>) -> Fallible<Entry> {
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
        published_at: extract_result(
            row.get::<usize, Option<String>>(12)
                .map(|row| DateTime::parse_from_rfc3339(&row).map(|dt| dt.with_timezone(&Utc))),
        )?,
        published_by: extract_result(
            row.get::<usize, Option<String>>(13)
                .map(|row| serde_json::from_str::<Vec<String>>(&row)),
        )?,
        reading_time: row.get(14),
        starred_at: extract_result(
            row.get::<usize, Option<String>>(15)
                .map(|row| DateTime::parse_from_rfc3339(&row).map(|dt| dt.with_timezone(&Utc))),
        )?,
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
        annotations: None, // NOTE: annotations are not loaded on purpose
        tags: serde_json::from_str::<Tags>(&row.get::<usize, String>(24))?,
    })
}

/// Parse an Annotation from a rusqlite::Row. NOTE: this will only work with the correct row
/// ordering. See the queries where this is used for a template.
fn row_to_ann<'r, 's, 't0>(row: &'r Row<'s, 't0>) -> Fallible<Annotation> {
    Ok(Annotation {
        id: ID(row.get(0)),
        annotator_schema_version: row.get(1),
        created_at: DateTime::parse_from_rfc3339(&row.get::<usize, String>(2))
            .map(|dt| dt.with_timezone(&Utc))?,
        ranges: serde_json::from_str::<Vec<Range>>(&row.get::<usize, String>(3))?,
        text: row.get(4),
        updated_at: DateTime::parse_from_rfc3339(&row.get::<usize, String>(5))
            .map(|dt| dt.with_timezone(&Utc))?,
        quote: row.get(6),
        user: row.get(7),
    })
}

/// logs a sql query string. this function just for consistency
fn log_sql(sql: &str) {
    debug!("SQL {:?}", sql);
}
