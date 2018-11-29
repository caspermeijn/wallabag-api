//! All types used by the client. Some are returned by the client and shouldn't
//! need to be created manually, while others are designed to be created and
//! passed to client methods (eg. creating new entries).
use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

mod annotations;
mod common;
mod entries_filter;
mod entry;
mod format;
mod new_entry;
mod patch_entry;
mod tags;
mod user;

// re-export submodule types
pub(crate) use self::annotations::AnnotationRows;
pub use self::annotations::{Annotation, Annotations, NewAnnotation, Range};
pub use self::common::ID;
pub use self::entries_filter::{EntriesFilter, SortBy, SortOrder};
pub(crate) use self::entry::{DeletedEntry, PaginatedEntries};
pub use self::entry::{Entries, Entry};
pub use self::format::Format;
pub use self::new_entry::NewEntry;
pub use self::patch_entry::PatchEntry;
pub use self::tags::{DeletedTag, Tag, TagString, Tags};
pub use self::user::{NewlyRegisteredInfo, RegisterInfo, User};

/// used internally to store information about the oauth token
#[derive(Deserialize, Debug)]
pub(crate) struct TokenInfo {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
    pub scope: Option<String>,
    pub refresh_token: String,
}

/// configuration to use to init a `Client`.
#[derive(Debug)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
    pub base_url: String,
}

/// The type returned from `check_exists`. The format is URL: ID. If ID is None,
/// then that url doesn't exist in the db.
pub type ExistsInfo = HashMap<String, Option<ID>>;

/// Internal struct for deserializing a response upon checking the existance of
/// a url.
#[derive(Deserialize, Debug)]
pub(crate) struct ExistsResponse {
    pub exists: Option<ID>,
}

/// a little trick to get around having to provide type annotations for a unit or
/// none value when passing to url serializer
#[derive(Serialize, Debug)]
pub(crate) struct Unit {}
pub(crate) static UNIT: &Unit = &Unit {};
