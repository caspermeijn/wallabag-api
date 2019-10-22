//! # Wallabag API
//!
//! Provides full type-safe async access to a [Wallabag](https://wallabag.org/) API server.
//! Contains methods for creating, reading, modifying, and deleting entries, annotations, and tags.
//!
//!
//! Example code to retrieve and print all starred entries.
//! ```notrust
//! use std::env;
//!
//! use wallabag_api::types::{Config, EntriesFilter, SortBy, SortOrder};
//! use wallabag_api::Client;
//!
//! #[tokio::main]
//! pub fn main() {
//!     let config = Config {
//!         client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
//!         client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
//!         username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
//!         password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
//!         base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
//!     };
//!
//!     println!("{:#?}", config);
//!
//!     let mut client = Client::new(config);
//!
//!     let filter = EntriesFilter {
//!         archive: None,
//!         starred: Some(true),
//!         sort: SortBy::Created,
//!         order: SortOrder::Desc,
//!         tags: vec![],
//!         since: 0,
//!         public: None,
//!     };
//!
//!     let response = client.get_entries_with_filter(&filter).await;
//!     match response {
//!         Err(e) => {
//!             println!("Error: {}", e);
//!         }
//!         Ok(entries) => {
//!             // do something with the entries!
//!             for entry in entries {
//!                 println!(
//!                     "{} | {} | Starred at {}",
//!                     entry.id,
//!                     entry.title.unwrap_or("Untitled".to_owned()),
//!                     entry.starred_at.unwrap()
//!                 );
//!             }
//!         }
//!     }
//! }
//! ```

mod client;
pub mod errors;
pub mod types;
mod utils;

pub use crate::client::Client;
pub use crate::errors::ClientError;
