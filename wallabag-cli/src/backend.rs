// backend module that links the api client to a local database and provides
// sync

use std::env;
use std::path::PathBuf;

use rusqlite::Result as SQLResult;
use rusqlite::{Connection, NO_PARAMS};

use wallabag_api::types::{Config, Tag, ID};
use wallabag_api::Client;

pub struct Backend {
    db_file: PathBuf,
}

impl Backend {
    pub fn new<T: Into<PathBuf>>(db_file: T) -> Self {
        Backend {
            db_file: db_file.into(),
        }
    }

    fn conn(&self) -> SQLResult<Connection> {
        Connection::open(&self.db_file)
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

    pub fn save_tag(tag: Tag) -> () {}

    pub fn load_tags(&self) -> SQLResult<()> {

        let conn = self.conn()?;

        // query and display the tags
        let mut stmt = conn.prepare("select id, label, slug from tags")?;

        let results = stmt
            .query_map(NO_PARAMS, |row| Tag {
                id: ID(row.get(0)),
                label: row.get(1),
                slug: row.get(2),
            })?;

        for tag in results {
            println!("{}", tag?.label);
        }

        Ok(())
    }

    pub fn sync(&self) -> SQLResult<()> {
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

        let tags = client
            .get_tags()
            .expect("failed to retrieve tags from server");

        let conn = self.conn()?;

        for tag in tags {
            conn.execute(
                "insert into tags (id, label, slug) values (?1, ?2, ?3)",
                &[&tag.id.to_string(), &tag.label, &tag.slug],
            )?;
        }
        Ok(())
    }
}
