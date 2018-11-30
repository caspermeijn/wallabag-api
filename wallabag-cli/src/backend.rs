// backend module that links the api client to a local database and provides
// sync

use std::env;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::connection::SimpleConnection;

use wallabag_api::Client;
use wallabag_api::types::{ID, Config, Tag};

use crate::schema::tags;

#[derive(Queryable, Insertable)]
#[table_name="tags"]
pub struct DbTag {
    pub id: i32,
    pub label: String,
    pub slug: String,
}

/// For convenience.
impl Into<Tag> for DbTag {
    fn into(self) -> Tag {
        Tag {
            id: ID(self.id as u32),
            label: self.label,
            slug: self.slug,
        }
    }
}

/// For convenience.
impl From<Tag> for DbTag {
    fn from(tag: Tag) -> Self {
        DbTag {
            id: *tag.id as i32,
            label: tag.label,
            slug: tag.slug,
        }
    }
}


fn establish_connection() -> SqliteConnection {
    let database_url = "db.sqlite3";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn init_db() -> QueryResult<()> {
    let query = include_str!("../migrations/create_tables/up.sql");
    let conn = establish_connection();
    conn.batch_execute(query)
}

pub fn load_tags() -> () {
    use crate::schema::tags::dsl::*;

    let config = Config {
        client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
        client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
        username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
        password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
        base_url:  env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
    };

    let mut client = Client::new(config);

    let server_tags = client.get_tags().expect("failed to retrieve tags from server");

    let conn = establish_connection();

    // load tags into the db

    for tag_ in server_tags {

        let new_tag = DbTag::from(tag_);

        diesel::insert_into(tags)
            .values(&new_tag)
            .execute(&conn)
            .expect("Error saving new tag");
    }

    // query and display the tags
    let results = tags
        .load::<DbTag>(&conn)
        .expect("Error loading tags");

    println!("Displaying {} tags", results.len());
    for tag in results {
        let tag: Tag = tag.into();
        println!("{}", tag.label);
    }
}
