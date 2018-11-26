use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

mod new_entry;
mod patch_entry;
mod user;

pub use self::new_entry::NewEntry;
pub use self::patch_entry::PatchEntry;
pub use self::user::{NewlyRegisteredInfo, RegisterInfo, User};

pub type ID = u32;

impl From<Entry> for ID {
    fn from(entry: Entry) -> Self {
        entry.id
    }
}

pub type ExistsInfo = HashMap<String, Option<ID>>;

/// Represents possible filters to apply to `get_entries_filtered`. To use the
/// default for a filter, set the value to `None`.
#[derive(Serialize, Debug)]
pub struct EntriesFilter {
    /// 1 or 0, all entries by default
    pub archive: Option<u32>, // 1 or 0

    /// 1 or 0, all entries by default
    pub starred: Option<u32>, // 1 or 0

    /// 'created' or 'updated', default 'created'
    pub sort: Option<String>,

    /// 'asc' or 'desc', default 'desc'
    pub order: Option<String>, // 'asc' or 'desc'

    /// "tag1,tag2"; return entries that match _all_ tags. default all entries.
    pub tags: Option<String>, // 'tag1,tag2' should be urlencoded

    /// timestamp since when you want entries updated. This would be useful when
    /// implementing a sync method.
    pub since: Option<u32>, // timestamp. default 0

    /// 1 or 0, all entries by default. Whether or not the entries have a public
    /// link.
    pub public: Option<u32>, // 1 or 0
}

/// Represents possible filters to apply to get_entries.
#[derive(Serialize, Debug)]
pub(crate) struct _EntriesFilter {
    pub archive: Option<u32>,  // 1 or 0
    pub starred: Option<u32>,  // 1 or 0
    pub sort: Option<String>,  // 'created' or 'updated', default 'created'
    pub order: Option<String>, // 'asc' or 'desc'
    pub tags: Option<String>,  // 'tag1,tag2' should be urlencoded
    pub since: Option<u32>,    // timestamp. default 0
    pub public: Option<u32>,   // 1 or 0
    pub page: u32,             // page number; for pagination
}

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
    pub is_archived: u32, // 1 or 0 TODO: encode in enum or cast to bool
    pub is_public: bool,
    pub is_starred: u32,          // same as is_archived
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
    pub headers: Option<String>, // TODO: probably not string?
    pub http_status: Option<String>,
    pub is_archived: u32, // 1 or 0 TODO: encode in enum or cast to bool
    pub is_public: bool,
    pub is_starred: u32,          // same as is_archived
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
    pub exists: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Range {
    pub end: String,
    pub endOffset: String,
    pub start: String,
    pub startOffset: String,
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
