use std::fmt;

use serde_derive::{Deserialize, Serialize};

use super::common::ID;
use crate::errors::TagStringError;

/// List of tags declared for clarity.
pub type Tags = Vec<Tag>;

/// Represents a tag from the API.
#[derive(Deserialize, Serialize, Debug)]
pub struct Tag {
    /// The unique tag ID.
    pub id: ID,

    /// The label aka name. The API accepts tags with commas in the label, but this is discouraged
    /// as some API methods require tags to be supplied as a comma separated string (eg.
    /// "tag1,tag2") and commas in the labels causes the label to be split into multiple tags.
    pub label: String,

    /// The url-friendly unique label for the tag to be used in web links. Usually derived from the
    /// tag label. Eg.  `https://framabag.org/tag/list/vim-1` for a tag labelled `vim`.
    pub slug: String,
}

/// Convenience method to use an ID or Tag interchangeably in client methods.
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

/// Represents a valid tag name for tags when sent to the API as a comma separated string. (eg.
/// "tag1,tag2") Only client methods that need to format a list of tags in this way will use
/// this.
#[derive(Debug)]
pub struct TagString {
    label: String,
}

impl fmt::Display for TagString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

impl TagString {
    /// Create a new tag label, guaranteed to be a valid tag label. Returns None
    /// if invalid tag label.
    pub fn try_from<T: Into<String>>(label: T) -> Result<Self, TagStringError> {
        let label = label.into();

        if label.as_str().contains(',') {
            Err(TagStringError::ContainsComma)
        } else {
            Ok(Self { label })
        }
    }

    /// Get a reference to the tag label.
    pub fn as_str(&self) -> &str {
        &self.label
    }

    /// Consume the tag and convert into a plain String.
    pub fn into_string(self) -> String {
        self.label
    }
}
