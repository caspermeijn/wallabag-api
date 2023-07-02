// Copyright 2018 Samuel Walladge <samuel@swalladge.net>
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::env;
// use std::thread;
// use std::time::Duration;

// use wallabag_api::types::{
//     Config, Entry, Format, NewAnnotation, PatchEntry, Range, RegisterInfo, TagString, ID,
// };
use wallabag_api::types::Config;
use wallabag_api::Client;

async fn run_example() {
    let config = Config {
        client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
        client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
        username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
        password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
        base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
    };

    println!("{:#?}", config);

    let mut client = Client::new(config);

    let res = client.get_user().await;
    println!("{:#?}", res);

    // let res = client.get_entry(1801067).await;
    // println!("{:#?}", res);

    // thread::sleep(Duration::from_secs(5));

    // let res = client.get_entry(1801067u32).await;
    // println!("{:#?}", res);

    // let entry = Entry {
    //     annotations: None,
    //     content: None,
    //     created_at: "2018-11-24T10:09:43+0100".to_owned(),
    //     domain_name: Some(
    //         "example.com".to_owned()
    //     ),
    //     headers: None,
    //     http_status: Some(
    //         "200".to_owned()
    //     ),
    //     id: 1801067,
    //     is_archived: 0,
    //     is_public: false,
    //     is_starred: 0,
    //     language: None,
    //     mimetype: Some(
    //         "text/html".to_owned()
    //     ),
    //     origin_url: None,
    //     preview_picture: None,
    //     published_at: None,
    //     published_by: None,
    //     reading_time: 0,
    //     starred_at: None,
    //     tags: vec![],
    //     title: Some(
    //         "Example Domain".to_owned()
    //     ),
    //     uid: None,
    //     updated_at: "2018-11-26T05:17:24+0100".to_owned(),
    //     url: Some(
    //         "https://example.com/".to_owned()
    //     ),
    //     user_email: "".to_owned(),
    //     user_id: 1,
    //     user_name: "".to_owned()
    // };

    // let res = client.get_entry(entry);
    // println!("{:#?}", res);

    // let res = client.get_entries();
    // println!("{:#?}", res);

    // let res = client.delete_annotation(2);
    // println!("{:#?}", res);

    // let res = client.delete_annotation(904);
    // println!("{:#?}", res);

    // let res = client.get_annotations(1798248u32);
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

    // let res = client.delete_tags_by_label(vec![
    //     TagString::new("tag1").unwrap(),
    //     TagString::new("\"wat").unwrap(),
    //     TagString::new("tag2").unwrap(),
    // ]);
    // println!("{:#?}", res);

    // let res = client.delete_tag_by_label("".to_owned());
    // println!("{:#?}", res);

    // let res = client.get_tags();
    // println!("{:#?}", res);

    // let res = client.export_entry(1798248, Format::TXT).expect("wat");
    // println!("{}", res);

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

    // let ann = NewAnnotation {
    //     quote: "quote".to_owned(),
    //     ranges: vec![Range {
    //         end: None,
    //         start: None,
    //         end_offset: "3",
    //         start_offset: "1",
    //     }],
    //     text: "wat".to_owned(),
    // };

    // let res = client.create_annotation(1803171, ann);
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

    // let res = client.add_tags_to_entry(
    //     1801932u32,
    //     vec![
    //         "".to_owned(),
    //         "".to_owned(),
    //         "".to_owned(),
    //         "".to_owned(),
    //     ],
    // );
    // println!("{:#?}", res);

    // let res = client.delete_tag_from_entry(
    //     1801932,
    //     20398,
    // );
    // println!("{:#?}", res);
}

fn main() {
    async_std::task::block_on(run_example())
}
