use std::env;
use std::thread;

use wallabag_api::{AuthInfo, Config, API};

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

    let mut api = API::new(config);

    // let res = api.get_entry(1798248);
    let res = api.get_entries();

    println!("{:#?}", res);
}
