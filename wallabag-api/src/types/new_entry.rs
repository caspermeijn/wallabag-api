use chrono::{DateTime, Utc};

use serde_derive::{Deserialize, Serialize};

use crate::utils::serde::bool_to_int;

/// A struct representing an entry to be created.
/// At least `url` must be provided. If you wish to provide the HTML content you
/// must also provide `content` and `title` to prevent the wallabag server from
/// fetching it from the url.
#[derive(Deserialize, Serialize, Debug)]
pub struct NewEntry {
    pub url: String,
    pub title: Option<String>,

    /// Tags containing a comma are valid but discouraged...
    /// Also note that these are tag labels as strings, not Tag objects.
    pub tags: Option<Vec<String>>,

    #[serde(serialize_with = "bool_to_int")]
    pub archive: Option<bool>,
    #[serde(serialize_with = "bool_to_int")]
    pub starred: Option<bool>,
    #[serde(serialize_with = "bool_to_int")]
    pub public: Option<bool>,

    pub content: Option<String>,
    pub language: Option<String>,
    pub preview_picture: Option<String>,
    pub published_at: Option<DateTime<Utc>>,

    /// Formatted as "name 1, name 2"
    pub authors: Option<String>,

    pub origin_url: Option<String>,
}

impl NewEntry {
    /// Create a new entry with a url (url is the only mandatory field). The
    /// rest of the fields will be populated with `None`.
    pub fn new_with_url(url: String) -> Self {
        Self {
            url,
            title: None,
            tags: None,
            archive: None,
            starred: None,
            content: None,
            language: None,
            preview_picture: None,
            published_at: None,
            authors: None,
            public: None,
            origin_url: None,
        }
    }
}
