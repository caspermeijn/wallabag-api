use std::env;

use wallabag_api::types::{AuthInfo, Config, NewAnnotation, Range};
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

    // let res = api.get_entry(1798248);
    // let res = client.get_entries();
    // let res = client.delete_annotation(2);
    // println!("{:#?}", res);

    // let res = client.delete_annotation(904);
    // println!("{:#?}", res);

    // let res = client.get_annotations(1798248);
    // println!("{:#?}", res);

    let res = client.create_annotation(0, NewAnnotation {
        quote: "Below is a snippet from main.c (source):".to_owned(), 
        ranges: vec![Range {
            end: "/p[4]".to_owned(),
            endOffset: "253".to_owned(),
            start: "/p[4]".to_owned(),
            startOffset: "213".to_owned(),
        }],
        text: "Thing".to_owned(),
        user: None,
    });

    println!("{:#?}", res);
}
