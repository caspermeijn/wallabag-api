use std::env;

use wallabag_api::types::{AuthInfo, Config};
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

    println!("{:?}", config);

    let mut client = Client::new(config);

    // let res = api.get_entry(1798248);
    let res = client.get_entries();

    println!("{:#?}", res);
}
