use std::env;
use std::result::Result;

use wallabag_api::types::{Config, NewEntry};
use wallabag_api::Client;

async fn run_example() -> Result<(), ()> {
    let config = Config {
        client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
        client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
        username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
        password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
        base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
    };

    println!("{:#?}", config);

    let mut client = Client::new(config);

    let url = std::env::args().nth(1).ok_or_else(|| {
        println!("Usage: save_url <url>");
        ()
    })?;

    let entry = NewEntry::new_with_url(url);
    // entry.tags = Some(vec!["wat,thing".to_owned(), "console".to_owned()]);
    // entry.archive = Some(true);

    let res = client.create_entry(&entry).await;

    match res {
        Err(e) => {
            println!("Failed to add entry: {:?}", e);
            Err(())
        }
        Ok(entry) => {
            println!("Success!");
            println!("{:#?}", entry);
            Ok(())
        }
    }
}

fn main() -> Result<(), ()> {
    async_std::task::block_on(run_example())
}
