use std::fmt;

use serde_derive::{Deserialize, Serialize};

use super::common::ID;

/// List of tags declared for clarity.
pub type Tags = Vec<Tag>;

/// Represents a tag from the api.
#[derive(Deserialize, Serialize, Debug)]
pub struct Tag {
    pub id: ID,
    pub label: String,
    pub slug: String,
}

/// Convenience method to use an ID or Tag interchangably in client methods.
impl From<Tag> for ID {
    fn from(tag: Tag) -> Self {
        tag.id
    }
}

/// Represents a deleted tag, since deleted tags don't come with IDs.
#[derive(Deserialize, Debug)]
pub struct DeletedTag {
    pub label: String,
    pub slug: String,
}

/// Represents a valid tag name
#[derive(Debug)]
pub struct TagString {
    label: String,
}

// so we can use to_string
impl fmt::Display for TagString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl TagString {
    /// Create a new tag label, guaranteed to be a valid tag label. Returns None
    /// if invalid tag label.
    pub fn new<T: Into<String>>(label: T) -> Option<Self> {
        let label = label.into();

        if label.as_str().contains(",") {
            None
        } else {
            Some(TagString { label })
        }
    }
}
