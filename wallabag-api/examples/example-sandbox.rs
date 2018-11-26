use std::env;

use wallabag_api::types::{AuthInfo, Config, NewAnnotation, PatchEntry, Range, RegisterInfo};
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

    // let res = client.get_tags();
    // println!("{:#?}", res);

    // // let res = client.delete_tag(404);
    // let res = client.delete_tag(20382);
    // println!("{:#?}", res);

    // let res = client.get_tags();
    // println!("{:#?}", res);

    // let res = client.delete_tags_by_label(vec!["tag1".to_owned(), "tag2".to_owned()]);
    // println!("{:#?}", res);

    // let res = client.delete_tag_by_label("test".to_owned());
    // println!("{:#?}", res);

    // let res = client.get_tags();
    // println!("{:#?}", res);

    // let res = client.export_entry(1800725, Format::XML);
    // println!("{:#?}", res);

    // let res = client.get_tags_for_entry(1800725);
    // println!("{:#?}", res);

    // let res = client.delete_entry(1801020);
    // println!("{:#?}", res);

    // let res = client.get_entry(1801067);
    // match res {
    //     Ok(entry) => {
    //         println!("content: {:?}", entry.content);
    //     }
    //     Err(e) => {
    //         println!("{:?}", e);
    //     }
    // }

    // let res = client.update_entry(1801067, &PatchEntry {
    //     title: Some("EDITED :)".to_owned()),
    //     tags: None,
    //     archive: None,
    //     starred: Some(0),
    //     content: Some("lol wat".to_owned()),
    //     language: None,
    //     preview_picture: None,
    //     published_at: None,
    //     authors: None,
    //     public: None,
    //     origin_url: None,

    // });
    // println!("{:#?}", res);

    // let res = client.get_entry(1801067);
    // match res {
    //     Ok(entry) => {
    //         println!("content: {:?}", entry.content);
    //     }
    //     Err(e) => {
    //         println!("{:?}", e);
    //     }
    // }

    // let res = client.reload_entry(1801932);
    // println!("{:#?}", res);

    // let res = client.get_entry(1801067);
    // match res {
    //     Ok(entry) => {
    //         println!("content: {:?}", entry.content);
    //     }
    //     Err(e) => {
    //         println!("{:?}", e);
    //     }
    // }

    let res = client.add_tags_to_entry(
        1801932,
        vec![
            "tag1".to_owned(),
            "jktjbucjraontebjtaneteu".to_owned(),
            "deja".to_owned(),
            "wat,dis".to_owned(),
        ],
    );
    println!("{:#?}", res);
}
