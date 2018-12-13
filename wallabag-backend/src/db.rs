use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use failure::Fallible;
use rusqlite::types::ToSql;
use rusqlite::{Connection, OpenFlags, Row, NO_PARAMS};
use serde_json;

use log::debug;

use wallabag_api::types::{
    Annotation, Annotations, Entries, Entry, NewAnnotation, Range, Tag, Tags, ID,
};

pub struct NewUrl {
    pub id: i64,
    pub url: String,
}

#[derive(Debug)]
pub enum DBClientError {
    DBExists,
}
impl fmt::Display for DBClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for DBClientError {}

#[derive(Debug)]
pub struct DB {
    db_file: PathBuf,
}

impl DB {
    pub fn new<T: Into<PathBuf>>(db_file: T) -> Self {
        Self {
            db_file: db_file.into(),
        }
    }

    /// Opens a new connection to the db, turns on foreign keys support, and returns the
    /// connection.
    ///
    /// If the database file doesn't already exist, the db will be created and inited.
    fn conn(&self) -> Fallible<Connection> {
        if !self.db_file.exists() {
            debug!("DB file does not exist; initializing");
            self.init()?;
        }

        let conn = Connection::open_with_flags(&self.db_file, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;
        Ok(conn)
    }

    /// Reset the database to a clean state. Database file will be created if
    /// not existing.
    ///
    /// Warning: this means all data in database will be lost and open connections will be
    /// disrupted due to the file being deleted.
    pub fn reset(&self) -> Fallible<()> {
        if self.db_file.exists() {
            fs::remove_file(&self.db_file)?;
        }

        self.up()
    }

    /// Initiates the database if the database file doesn't exist. If the
    /// database file does exist but is in a broken state, then you should
    /// manually delete the file and start again.
    pub fn init(&self) -> Fallible<()> {
        if self.db_file.exists() {
            debug!("DB file already exists, not initing");
            Err(DBClientError::DBExists)?;
        }

        self.up()
    }

    /// Create tables/indices/etc. in the database.
    ///
    /// This also creates the file in the process of making the connection.
    pub fn up(&self) -> Fallible<()> {
        // manually set up the connection because we don't want the magic that self.conn() does.
        let conn = Connection::open(&self.db_file)?;
        conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)?;

        let query = include_str!("../sql/up.sql");
        conn.execute_batch(query)?;

        Ok(())
    }

    /// Get an annotation from the db by id.
    pub fn get_annotation<T: Into<ID>>(&self, id: T) -> Fallible<Option<Annotation>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"select id, annotator_schema_version, created_at, ranges, text,
            updated_at, quote, user from annotations where id = ?"#,
        )?;

        let mut results = stmt.query_and_then(&[&id.into().as_int()], row_to_ann)?;

        extract_result(results.next())
    }

    /// Get all annotations updated since a given date. Useful for syncing purposes.
    pub fn get_annotations_since(&self, since: DateTime<Utc>) -> Fallible<Annotations> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"select id, annotator_schema_version, created_at, ranges, text,
            updated_at, quote, user from annotations where updated_at >= ?"#,
        )?;

        let results = stmt.query_and_then(&[since.to_rfc3339()], row_to_ann)?;

        results.collect()
    }

    /// Get a set of IDs of all annotations in the database.
    pub fn get_all_annotation_ids(&self) -> Fallible<HashSet<ID>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("SELECT id from annotations")?;

        let results = stmt.query_and_then(NO_PARAMS, |row| -> Fallible<ID> {
            Ok(ID(row.get_checked(0)?))
        })?;

        results.collect()
    }

    /// Get a set of IDs of all entries in the database.
    pub fn get_all_entry_ids(&self) -> Fallible<HashSet<ID>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("SELECT id from entries")?;

        let results = stmt.query_and_then(NO_PARAMS, |row| -> Fallible<ID> {
            Ok(ID(row.get_checked(0)?))
        })?;

        results.collect()
    }

    /// Get all new annotations from the DB. Returns a tuple of (entry_id as ID, local id of new
    /// annotation in db, NewAnnotation object).
    pub fn get_new_annotations(&self) -> Fallible<Vec<(ID, i64, NewAnnotation)>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt =
            conn.prepare("SELECT id, quote, ranges, text, entry_id from new_annotations")?;

        let results =
            stmt.query_and_then(NO_PARAMS, |row| -> Fallible<(ID, i64, NewAnnotation)> {
                Ok((
                    ID(row.get_checked(4)?),
                    row.get_checked(0)?,
                    NewAnnotation {
                        quote: row.get_checked(1)?,
                        ranges: serde_json::from_str::<Vec<Range>>(
                            &row.get_checked::<usize, String>(2)?,
                        )?,
                        text: row.get_checked(3)?,
                    },
                ))
            })?;

        results.collect()
    }

    /// Remove a new annotation entry from db, signifying that it has been synced.
    pub fn remove_new_annotation(&self, id: i64) -> Fallible<()> {
        self.conn()?
            .execute("DELETE FROM new_annotations WHERE id = ?", &[&id])?;

        Ok(())
    }

    /// Get all entries from the database. Does not include content or annotations. (entry.content
    /// and entry.annotations will always be None)
    pub fn get_all_entries(&self) -> Fallible<Entries> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare(
            r#"SELECT id, "", created_at, domain_name, http_status,
            is_archived, is_public, is_starred, language, mimetype, origin_url,
            preview_picture, published_at, published_by, reading_time,
            starred_at, title, uid, updated_at, url, headers, user_email,
            user_id, user_name, tags from entries"#,
        )?;

        let results = stmt.query_and_then(NO_PARAMS, row_to_entry)?;

        results.collect()
    }

    /// Get all entries from the database have updated_at greater than or equal to `since`.
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

        let results = stmt.query_and_then(&[since.to_rfc3339()], row_to_entry)?;

        results.collect()
    }

    /// Remove an entry from the database. Should only be called from a sync procedure otherwise
    /// this delete action will not be synced.
    pub fn delete_entry<T: Into<ID>>(&self, id: T) -> Fallible<()> {
        self.conn()?
            .execute("DELETE FROM entries WHERE id = ?", &[&id.into().as_int()])?;

        Ok(())
    }

    /// Remove an annotation from the database. Should only be called from a sync procedure
    /// otherwise this delete action will not be synced.
    pub fn delete_annotation<T: Into<ID>>(&self, id: T) -> Fallible<()> {
        self.conn()?.execute(
            "DELETE FROM annotations WHERE id = ?",
            &[&id.into().as_int()],
        )?;

        Ok(())
    }

    pub fn get_new_urls(&self) -> Fallible<Vec<NewUrl>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("SELECT id, url FROM new_urls")?;
        let results = stmt.query_and_then(NO_PARAMS, |row| -> Fallible<NewUrl> {
            Ok(NewUrl {
                id: row.get_checked(0)?,
                url: row.get_checked(1)?,
            })
        })?;

        results.collect()
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
        let results = stmt.query_and_then(NO_PARAMS, |row| -> Fallible<ID> {
            Ok(ID(row.get_checked(0)?))
        })?;

        results.collect()
    }

    /// Remove an annotation from the delteed entries table. This marks a local delete as synced.
    pub fn remove_delete_annotation<T: Into<ID>>(&self, annotation_id: T) -> Fallible<()> {
        self.conn()?.execute(
            "DELETE FROM deleted_annotations WHERE id = ?",
            &[&annotation_id.into().as_int()],
        )?;

        Ok(())
    }

    pub fn get_entry_deletes(&self) -> Fallible<Vec<ID>> {
        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("SELECT id FROM deleted_entries")?;
        let results = stmt.query_and_then(NO_PARAMS, |row| -> Fallible<ID> {
            Ok(ID(row.get_checked(0)?))
        })?;

        results.collect()
    }

    /// Remove an entry from the delteed entries table. This marks a local delete as synced.
    pub fn remove_delete_entry<T: Into<ID>>(&self, entry_id: T) -> Fallible<()> {
        self.conn()?.execute(
            "DELETE FROM deleted_entries WHERE id = ?",
            &[&entry_id.into().as_int()],
        )?;

        Ok(())
    }

    /// Get a single entry by ID. Returns None if not found.
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

        let mut results = stmt.query_and_then(&[&id.into().as_int()], row_to_entry)?;
        extract_result(results.next())
    }

    /// Get the last time a sync was performed. used for optimization by the
    /// for syncing.
    pub fn get_last_sync(&self) -> Fallible<DateTime<Utc>> {
        self.conn()?.query_row(
            "SELECT (last_sync) FROM config WHERE id = 1",
            NO_PARAMS,
            |row| -> Fallible<DateTime<Utc>> {
                Ok(
                    DateTime::parse_from_rfc3339(&row.get_checked::<usize, String>(0)?)
                        .map(|dt| dt.with_timezone(&Utc))?,
                )
            },
        )?
    }

    /// Sets the last sync time to now.
    pub fn touch_last_sync(&self) -> Fallible<()> {
        self.conn()?.execute(
            "UPDATE config SET last_sync = ? WHERE id = 1",
            &[&chrono::offset::Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// Self-explanatory.
    pub fn drop_tag_links_for_entry<T: Into<ID>>(&self, entry_id: T) -> Fallible<()> {
        self.conn()?.execute(
            "DELETE FROM taglinks WHERE entry_id = ?",
            &[&entry_id.into().as_int()],
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
                &entry.user_id.as_int(),
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
            &[&entry_id.into().as_int() as &ToSql, &tag.id.as_int()],
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
        let results = stmt.query_and_then(NO_PARAMS, |row| -> Fallible<Tag> {
            Ok(Tag {
                id: ID(row.get_checked(0)?),
                label: row.get_checked(1)?,
                slug: row.get_checked(2)?,
            })
        })?;

        results.collect()
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

/// Parse an Entry from a `rusqlite::Row`.
///
/// NOTE: this will only work with the correct row ordering. See the queries where this is used for
/// a template.
fn row_to_entry<'r, 's, 't0>(row: &'r Row<'s, 't0>) -> Fallible<Entry> {
    Ok(Entry {
        id: ID(row.get_checked(0)?),
        content: row.get_checked(1)?,
        created_at: DateTime::parse_from_rfc3339(&row.get_checked::<usize, String>(2)?)
            .map(|dt| dt.with_timezone(&Utc))?,
        domain_name: row.get_checked(3)?,
        http_status: row.get_checked(4)?,
        is_archived: row.get_checked(5)?,
        is_public: row.get_checked(6)?,
        is_starred: row.get_checked(7)?,
        language: row.get_checked(8)?,
        mimetype: row.get_checked(9)?,
        origin_url: row.get_checked(10)?,
        preview_picture: row.get_checked(11)?,
        published_at: extract_result(
            row.get_checked::<usize, Option<String>>(12)?
                .map(|row| DateTime::parse_from_rfc3339(&row).map(|dt| dt.with_timezone(&Utc))),
        )?,
        published_by: extract_result(
            row.get_checked::<usize, Option<String>>(13)?
                .map(|row| serde_json::from_str::<Vec<String>>(&row)),
        )?,
        reading_time: row.get_checked(14)?,
        starred_at: extract_result(
            row.get_checked::<usize, Option<String>>(15)?
                .map(|row| DateTime::parse_from_rfc3339(&row).map(|dt| dt.with_timezone(&Utc))),
        )?,
        title: row.get_checked(16)?,
        uid: row.get_checked(17)?,
        updated_at: DateTime::parse_from_rfc3339(&row.get_checked::<usize, String>(18)?)
            .map(|dt| dt.with_timezone(&Utc))?,
        url: row.get_checked(19)?,
        headers: extract_result(
            row.get_checked::<usize, Option<String>>(20)?
                .map(|row| serde_json::from_str::<Vec<String>>(&row)),
        )?,
        user_email: row.get_checked(21)?,
        user_id: ID(row.get_checked(22)?),
        user_name: row.get_checked(23)?,
        annotations: None, // NOTE: annotations are not loaded on purpose
        tags: serde_json::from_str::<Tags>(&row.get_checked::<usize, String>(24)?)?,
    })
}

/// Parse an Annotation from a `rusqlite::Row`.
///
/// NOTE: this will only work with the correct row ordering. See the queries where this is used for
/// a template.
fn row_to_ann<'r, 's, 't0>(row: &'r Row<'s, 't0>) -> Fallible<Annotation> {
    Ok(Annotation {
        id: ID(row.get_checked(0)?),
        annotator_schema_version: row.get_checked(1)?,
        created_at: DateTime::parse_from_rfc3339(&row.get_checked::<usize, String>(2)?)
            .map(|dt| dt.with_timezone(&Utc))?,
        ranges: serde_json::from_str::<Vec<Range>>(&row.get_checked::<usize, String>(3)?)?,
        text: row.get_checked(4)?,
        updated_at: DateTime::parse_from_rfc3339(&row.get_checked::<usize, String>(5)?)
            .map(|dt| dt.with_timezone(&Utc))?,
        quote: row.get_checked(6)?,
        user: row.get_checked(7)?,
    })
}

/// logs a sql query string. this function just for consistency
fn log_sql(sql: &str) {
    debug!("SQL {:?}", sql);
}
