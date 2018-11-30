// backend module that links the api client to a local database and provides
// sync

use std::env;

use rusqlite::{Connection, NO_PARAMS};
use rusqlite::Result as SQLResult;

use wallabag_api::types::{Config, Tag, ID};
use wallabag_api::Client;

fn establish_connection() -> Connection {
    let database_url = "db.sqlite3";
    Connection::open(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn init_db() -> SQLResult<()> {
    let query = include_str!("../sql/up.sql");
    let conn = establish_connection();
    conn.execute_batch(query)
}

pub fn load_tags() -> () {
    let config = Config {
        client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
        client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
        username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
        password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
        base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
    };

    let mut client = Client::new(config);

    let tags = client
        .get_tags()
        .expect("failed to retrieve tags from server");

    let conn = establish_connection();

    for tag in tags {
        conn.execute(
            "insert into tags (id, label, slug) values (?1, ?2, ?3)",
            &[&tag.id.to_string(), &tag.label, &tag.slug],
        )
        .unwrap();
    }

    // query and display the tags
    let mut stmt = conn
        .prepare("select id, label, slug from tags")
        .unwrap();

    let results = stmt
        .query_map(NO_PARAMS, |row| Tag {
            id: ID(row.get(0)),
            label: row.get(1),
            slug: row.get(2),
        })
        .unwrap();

    for tag in results {
        println!("{}", tag.unwrap().label);
    }
}
