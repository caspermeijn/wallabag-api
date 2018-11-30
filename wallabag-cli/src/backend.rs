// backend module that links the api client to a local database and provides
// sync

use std::env;
use std::path::PathBuf;
use std::result::Result;

use rusqlite::Result as SQLResult;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use serde_json;

use wallabag_api::types::{Annotation, Config, Tag, ID};
use wallabag_api::Client;

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

    pub fn sync(&self) -> Result<(), failure::Error> {
        unimplemented!();

        let config = Config {
            client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
            client_secret: env::var("WALLABAG_CLIENT_SECRET")
                .expect("WALLABAG_CLIENT_SECRET not set"),
            username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
            password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
            base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
        };

        let mut client = Client::new(config);

        let entry_id = 1798248;
        let annotations = client.get_annotations(entry_id)?;

        for ann in annotations.into_iter() {
            self.db.update_annotation(ann, entry_id)?;
        }

        // let tags = client
        //     .get_tags()
        //     .expect("failed to retrieve tags from server");

        // let conn = self.conn()?;

        // for tag in tags {
        //     conn.execute(
        //         "insert into tags (id, label, slug) values (?1, ?2, ?3)",
        //         &[&tag.id.to_string(), &tag.label, &tag.slug],
        //     )?;
        // }
        Ok(())
    }
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
        let query = include_str!("../sql/up.sql");
        self.conn()?.execute_batch(query)
    }

    pub fn down(&self) -> SQLResult<()> {
        let query = include_str!("../sql/down.sql");
        self.conn()?.execute_batch(query)
    }

    /// Save an annotation to the database. If not existing (by id), it will be
    /// inserted; if existing, it will replace the old value.
    pub fn update_annotation<T: Into<ID>>(&self, ann: Annotation, entry_id: T) -> SQLResult<()> {
        let conn = self.conn()?;

        conn.execute(
            r#"INSERT OR REPLACE INTO annotations 
            (id, annotator_schema_version, created_at, ranges, text, updated_at,
            quote, user, entry_id) VALUES 
             (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            &[
                &*ann.id as &ToSql,
                &ann.annotator_schema_version,
                &ann.created_at.to_rfc2822(),
                &serde_json::ser::to_string(&ann.ranges).unwrap(),
                &ann.text,
                &ann.updated_at.to_rfc2822(),
                &ann.quote,
                &ann.user,
                &*entry_id.into(),

            ],
        ).map(|e| ())
    }

    pub fn save_tag(tag: Tag) -> () {}

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
