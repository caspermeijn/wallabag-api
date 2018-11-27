use chrono::{DateTime, Utc};

use serde_derive::{Deserialize, Serialize};

use crate::utils::serde::bool_to_int;

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
