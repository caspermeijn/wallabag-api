use std::env;

use wallabag_api::types::{Config, EntriesFilter, SortBy, SortOrder};
use wallabag_api::Client;

async fn run_example() {
    let config = Config {
        client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
        client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
        username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
        password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
        base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
    };

    let mut client = Client::new(config);

    let filter = EntriesFilter {
        archive: None,
        starred: None,
        sort: SortBy::Created,
        order: SortOrder::Desc,
        tags: vec![],
        since: 0,
        public: None,
        per_page: Some(2),
    };

    let response = client.get_entries_page(&filter, 1).await;
    match response {
        Err(e) => {
            println!("Error: {}", e);
        }
        Ok(page) => {
            println!(
                "Fetched page {} of {}.",
                page.current_page, page.total_pages
            );
            println!(
                "{} entries per page, {} entries in total.",
                page.per_page, page.total_entries
            );
        }
    }
}

fn main() {
    async_std::task::block_on(run_example())
}
