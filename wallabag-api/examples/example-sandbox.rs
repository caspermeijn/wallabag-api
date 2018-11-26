use std::env;

use wallabag_api::types::{AuthInfo, Config, NewAnnotation, Range, RegisterInfo};
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
    // println!("{:#?}", res);

    // let res = client.delete_annotation(2);
    // println!("{:#?}", res);

    // let res = client.delete_annotation(904);
    // println!("{:#?}", res);

    // let res = client.get_annotations(1798248);
    // println!("{:#?}", res);

    // let mut annotations = res.unwrap();

    // let mut annotation = &mut annotations[0]; // assume we have one
    // annotation.text = "HAHAHA HIJACKED".to_owned();

    // let res = client.update_annotation(&annotation);
    // println!("{:#?}", res);

    // let res = client.get_api_version();
    // println!("{:#?}", res);

    // let res = client.register_user(&RegisterInfo {
    //     username: "placeholder".to_owned(),
    //     password: "placeholder".to_owned(),
    //     email: "placeholder@example.com".to_owned(),
    //     client_name: "placeholder".to_owned(),
    // });
    // println!("{:#?}", res);


    let res = client.get_tags();
    println!("{:#?}", res);


}
