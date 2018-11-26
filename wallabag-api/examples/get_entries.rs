use std::env;

use wallabag_api::types::{AuthInfo, Config, EntriesFilter};
use wallabag_api::utils::Format;
use wallabag_api::Client;

pub fn main() {
    let config = Config {
        auth_info: AuthInfo {
            client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
            client_secret: env::var("WALLABAG_CLIENT_SECRET")
                .expect("WALLABAG_CLIENT_SECRET not set"),
            username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
            password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
        },
        base_url: "https://framabag.org".to_owned(),
    };

    println!("{:#?}", config);

    let mut client = Client::new(config);

    let filter = EntriesFilter {
        archive: None,
        starred: Some(1),
        sort: None,
        order: None,
        tags: None,
        since: None,
        public: None,
    };

    let res = client.get_entries_filtered(filter);
    println!("{:#?}", res);
}
