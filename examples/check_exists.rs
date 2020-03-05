use std::env;
use std::result::Result;

use wallabag_api::types::Config;
use wallabag_api::Client;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let config = Config {
        client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
        client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
        username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
        password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
        base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
    };

    println!("{:#?}", config);

    let mut client = Client::new(config);

    // example check multiple urls at once.
    let res = client
        .check_urls_exist(vec!["https://example.com/".to_owned(), "bla".to_owned()])
        .await;
    println!("{:#?}", res);

    let url = std::env::args().nth(1).ok_or_else(|| {
        println!("Usage: check_exists <url>");
        ()
    })?;

    let res = client.check_url_exists(url.clone()).await;

    match res {
        Err(e) => {
            println!("Request failed: {:?}", e);
            Err(())
        }
        Ok(exists) => {
            match exists {
                Some(id) => {
                    println!("Exists. id: {}", id);
                }
                None => {
                    println!("Url does not exist: {}", url);
                }
            }
            Ok(())
        }
    }
}
