use serde_derive::{Deserialize, Serialize};

/// A struct representing an entry to be changed.
///
/// TODO: document either fields
#[derive(Deserialize, Serialize, Debug)]
pub struct PatchEntry {
    pub title: Option<String>,

    // TODO: Into so that can pass a Tag struct or a string
    pub tags: Option<Vec<String>>,

    // TODO: research serde auto serialize 0/1 values into bool and viceverse
    pub archive: Option<u32>, // 0 or 1
    pub starred: Option<u32>, // 0 or 1  // TODO: enum for this?
    pub content: Option<String>,
    pub language: Option<String>,
    pub preview_picture: Option<String>,
    pub published_at: Option<String>, // datetime|integer as YYYY-MM-DDTHH:II:SS+TZ or a timestamp TODO: handle both
    pub authors: Option<String>,      // format: "name 1,name2"
    pub public: Option<u32>,          // 0 or 1
    pub origin_url: Option<String>,   // not sure how this differs from url?
}
