use serde_derive::{Deserialize, Serialize};

/// A struct representing an entry to be created
/// note from docs: If you want to provide the HTML content (which means
/// wallabag won't fetch it from the url), you must provide content, title & url
/// fields non-empty.
#[derive(Deserialize, Serialize, Debug)]
pub struct NewEntry {
    pub url: String,
    pub title: Option<String>,
    pub tags: Option<String>, // format: "tag1,tag2,tag3" TODO: method to convert Tags to this
    pub archive: Option<u32>, // 0 or 1
    pub starred: Option<u32>, // 0 or 1  // TODO: enum for this?
    pub content: Option<String>,
    pub language: Option<String>,
    pub preview_picture: Option<String>,
    pub published_at: Option<String>, // datetime|integer as YYYY-MM-DDTHH:II:SS+TZ or a timestamp
    pub authors: Option<String>,      // format: "name 1,name2"
    pub public: Option<u32>,          // 0 or 1
    pub origin_url: Option<String>,   // not sure how this differs from url?
}

impl NewEntry {
    /// Create a new entry with a url (url is the only mandatory field). The
    /// rest of the fields will be populated with `None`.
    pub fn new_with_url(url: String) -> Self {
        NewEntry {
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
