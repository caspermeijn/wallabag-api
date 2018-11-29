// backend module that links the api client to a local database and provides
// sync

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::connection::SimpleConnection;

fn establish_connection() -> SqliteConnection {
    let database_url = "test.sqlite3";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

// table! {
//     posts (id) {
//         id -> Integer,
//         title -> Text,
//         body -> Text,
//         published -> Bool,
//     }
// }

pub fn init_db() -> QueryResult<()> {
    let query = include_str!("./sql/up-sqlite.sql");
    let conn = establish_connection();
    conn.batch_execute(query)
}
