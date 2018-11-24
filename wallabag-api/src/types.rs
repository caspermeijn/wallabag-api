use serde_derive::{Deserialize, Serialize};

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
    pub annotations: Annotations,
    pub content: Option<String>,
    pub created_at: String,
    pub domain_name: Option<String>,
    pub headers: Option<String>, // TODO: probably not string
    pub http_status: Option<String>,
    pub id: u32,
    pub is_archived: u32,
    pub is_public: bool,
    pub is_starred: u32,
    pub language: Option<String>, // TODO: probably not string
    pub mimetype: Option<String>,
    pub origin_url: Option<String>,
    pub preview_picture: Option<String>,
    pub published_at: Option<String>,
    pub published_by: Option<String>,
    pub reading_time: u32,
    pub starred_at: Option<String>,
    pub tags: Vec<Tag>,
    pub title: Option<String>,
    pub uid: Option<String>,
    pub updated_at: String,
    pub url: Option<String>,
    pub user_email: String,
    pub user_id: u32,
    pub user_name: String,
}

pub type Annotations = Vec<Annotation>;

#[derive(Deserialize, Serialize, Debug)]
pub struct Annotation {
    pub annotator_schema_version: String,
    pub created_at: String,
    pub id: u32,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Range {
    pub end: String,
    pub endOffset: String,
    pub start: String,
    pub startOffset: String,
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub id: u32,
    pub label: String,
    pub slug: String,
}

#[derive(Deserialize, Debug)]
pub struct PaginatedEntries {
    pub limit: u32,
    pub page: u32,
    pub pages: u32,
    pub total: u32,
    // TODO: _links ?
    pub _embedded: EmbeddedEntries,
}

#[derive(Deserialize, Debug)]
pub struct EmbeddedEntries {
    pub items: Entries,
}

// a little trick to get aronud having to provide type annotations for a unit or
// none value when passing to url serializer
#[derive(Serialize, Debug)]
pub struct Unit {}
pub static UNIT: &Unit = &Unit {};

