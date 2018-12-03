use chrono::{DateTime, Utc};

use serde_derive::{Deserialize, Serialize};

use crate::utils::serde::bool_to_int;
use super::Entry;

/// A struct representing an entry to be changed.
#[derive(Deserialize, Serialize, Debug)]
pub struct PatchEntry {
    pub title: Option<String>,

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
    pub authors: Option<String>,      // format: "name 1,name2"
    pub origin_url: Option<String>,   // not sure how this differs from url?
}

/// Convert an Entry to a set of changes ready for sending to the api.
impl From<&Entry> for PatchEntry {
    fn from(entry: &Entry) -> Self {
        let tags: Vec<String> = entry.tags.iter().map(|t| t.label.clone()).collect();
        PatchEntry {
            title: entry.title.clone(),
            tags: Some(tags),
            archive: Some(entry.is_archived),
            starred: Some(entry.is_starred),
            public: Some(entry.is_public),
            content: entry.content.clone(),
            language: entry.language.clone(),
            preview_picture: entry.preview_picture.clone(),
            published_at: entry.published_at.clone(),
            authors: None,
            origin_url: entry.origin_url.clone(),
        }
    }
}
