use chrono::{DateTime, Utc};

use serde_derive::{Deserialize, Serialize};

use super::Entry;
use crate::utils::serde::bool_to_int;

/// A struct representing an entry to be changed. Fields here are the only fields that can be
/// modified directly via the api.
///
/// Setting a field to `None` causes the field to not be modified.
#[derive(Deserialize, Serialize, Debug)]
pub struct PatchEntry {
    pub title: Option<String>,

    /// List of tag labels as strings. Commas in tag labels are valid but discouraged.
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

/// Convert an Entry to a set of changes ready for sending to the api.
impl From<&Entry> for PatchEntry {
    fn from(entry: &Entry) -> Self {
        let tags: Vec<String> = entry.tags.iter().map(|t| t.label.clone()).collect();
        Self {
            title: entry.title.clone(),
            tags: Some(tags),
            archive: Some(entry.is_archived),
            starred: Some(entry.is_starred),
            public: Some(entry.is_public),
            content: entry.content.clone(),
            language: entry.language.clone(),
            preview_picture: entry.preview_picture.clone(),
            published_at: entry.published_at,
            authors: None,
            origin_url: entry.origin_url.clone(),
        }
    }
}
