use std::collections::HashMap;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use serde_derive::{Deserialize, Serialize};

mod new_entry;
mod patch_entry;
mod user;
mod entries_filter;

pub use self::new_entry::NewEntry;
pub use self::patch_entry::PatchEntry;
pub use self::user::{NewlyRegisteredInfo, RegisterInfo, User};
pub use self::entries_filter::{EntriesFilter, SortOrder, SortBy};

pub type ID = u32;

impl From<Entry> for ID {
    fn from(entry: Entry) -> Self {
        entry.id
    }
}

pub type ExistsInfo = HashMap<String, Option<ID>>;


#[derive(Deserialize, Debug)]
pub struct TokenInfo {
    pub access_token: String,
    pub expires_in: u32,
    pub token_type: String,
    pub scope: Option<String>,
    pub refresh_token: String,
}

#[derive(Debug)]
pub struct AuthInfo {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Config {
    pub auth_info: AuthInfo,
    pub base_url: String,
}

pub type Entries = Vec<Entry>;

/// Parser for converting pseudo-bool values from 0 or 1 integers to actual bool.
fn parse_intbool<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let x = u32::deserialize(d)?;

    match x {
        0 => Ok(false),
        1 => Ok(true),
        x => Err(DeError::custom(format!(
            "Could not deserialize {} as bool",
            x
        ))),
    }
}

/// A struct representing an entry from wallabag (a full saved article including
/// annotations and tags).
#[derive(Deserialize, Debug)]
pub struct Entry {
    pub annotations: Option<Annotations>,
    pub content: Option<String>,
    pub created_at: String,
    pub domain_name: Option<String>,
    pub headers: Option<String>, // TODO: probably not string?
    pub http_status: Option<String>,
    pub id: ID,

    #[serde(deserialize_with = "parse_intbool")]
    pub is_archived: bool,
    pub is_public: bool,

    #[serde(deserialize_with = "parse_intbool")]
    pub is_starred: bool,
    pub language: Option<String>, // TODO: probably not string
    pub mimetype: Option<String>,
    pub origin_url: Option<String>,
    pub preview_picture: Option<String>,
    pub published_at: Option<String>,
    pub published_by: Option<String>,
    pub reading_time: u32,
    pub starred_at: Option<String>,
    pub tags: Tags,
    pub title: Option<String>,
    pub uid: Option<String>,
    pub updated_at: String,
    pub url: Option<String>,
    pub user_email: String,
    pub user_id: ID,
    pub user_name: String,
}

/// A struct representing an entry from wallabag (a full saved article including
/// annotations and tags).
#[derive(Deserialize, Debug)]
pub(crate) struct DeletedEntry {
    pub annotations: Option<Annotations>,
    pub content: Option<String>,
    pub created_at: String,
    pub domain_name: Option<String>,
    pub headers: Option<String>,
    pub http_status: Option<String>,

    #[serde(deserialize_with = "parse_intbool")]
    pub is_archived: bool,
    pub is_public: bool,

    #[serde(deserialize_with = "parse_intbool")]
    pub is_starred: bool,
    pub language: Option<String>,
    pub mimetype: Option<String>,
    pub origin_url: Option<String>,
    pub preview_picture: Option<String>,
    pub published_at: Option<String>,
    pub published_by: Option<String>,
    pub reading_time: u32,
    pub starred_at: Option<String>,
    pub tags: Tags,
    pub title: Option<String>,
    pub uid: Option<String>,
    pub updated_at: String,
    pub url: Option<String>,
    pub user_email: String,
    pub user_id: ID,
    pub user_name: String,
}

pub type Annotations = Vec<Annotation>;

#[derive(Deserialize, Serialize, Debug)]
pub struct Annotation {
    pub annotator_schema_version: String,
    pub created_at: String,
    pub id: ID,
    pub quote: String,
    pub ranges: Vec<Range>,
    pub text: String,
    pub updated_at: String,
    pub user: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct NewAnnotation {
    pub quote: String,
    pub ranges: Vec<Range>,
    pub text: String,
    pub user: Option<String>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ExistsResponse {
    pub exists: Option<ID>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub end: Option<String>,
    pub end_offset: String,
    pub start: Option<String>,
    pub start_offset: String,
}

pub type Tags = Vec<Tag>;

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub id: ID,
    pub label: String,
    pub slug: String,
}

impl From<Tag> for ID {
    fn from(tag: Tag) -> Self {
        tag.id
    }
}

#[derive(Deserialize, Debug)]
pub struct DeletedTag {
    pub label: String,
    pub slug: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct PaginatedEntries {
    pub limit: u32,
    pub page: u32,
    pub pages: u32,
    pub total: u32,
    pub _embedded: EmbeddedEntries,
}

#[derive(Deserialize, Debug)]
pub(crate) struct EmbeddedEntries {
    pub items: Entries,
}

// a little trick to get around having to provide type annotations for a unit or
// none value when passing to url serializer
#[derive(Serialize, Debug)]
pub(crate) struct Unit {}
pub(crate) static UNIT: &Unit = &Unit {};
