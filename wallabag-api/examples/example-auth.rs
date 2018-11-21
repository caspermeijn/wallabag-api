use std::env;

use wallabag_api::{AuthInfo, Config, API};

pub fn main() {
    let config = Config {
        auth_info: AuthInfo {
            // TODO, XXX: don't hardcode
            client_id: env::var("CLIENT_ID").expect("CLIENT_ID not set"),
            client_secret: env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set"),
            username: env::var("USERNAME").expect("USERNAME not set"),
            password: env::var("PASSWORD").expect("PASSWORD not set"),
        },
    };

    let api = API::new(config);

    let res = api.get_entries();
    println!("{:?}", res);
}
