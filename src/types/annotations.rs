use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

use super::common::ID;
use crate::utils::serde::parse_stringint;

/// Type alias for clarity.
pub type Annotations = Vec<Annotation>;

/// Represents an annotation as returned from the API.
///
/// Annotations are in Annotatorjs format. <https://annotatorjs.org/>
/// See <http://docs.annotatorjs.org/en/v1.2.x/annotation-format.html> for documentation on the
/// format.
///
#[derive(Deserialize, Serialize, Debug)]
pub struct Annotation {
    /// The unique integral id of the annotation.
    pub id: ID,

    /// A schema version to presumably support updates in the future. Currently all annotations
    /// appear to be `v1.0`. Hopefully this isn't going to get breaking changes any time soon.
    pub annotator_schema_version: String,

    /// When the annotation was created on the server.
    pub created_at: DateTime<Utc>,

    /// The quoted (or highlighted) text from the entry.
    pub quote: Option<String>,

    /// A list of ranges from the entry that the annotation covers. Most annotations cover a single
    /// range.
    pub ranges: Vec<Range>,

    /// The content of the annotation - any text the user added to annotate the entry.
    pub text: String,

    /// Timestamp of when the annotation was last updated. This is independent of the associated
    /// entry.
    pub updated_at: DateTime<Utc>,

    /// Possibly part of wallabag planning on supporting sharing between users. Currently this
    /// field is always `None`.
    pub user: Option<String>,
}

/// This is implemented so that an Annotation can be used interchangeably with an ID for some
/// client methods. For convenience.
impl From<Annotation> for ID {
    fn from(ann: Annotation) -> Self {
        ann.id
    }
}

/// This is implemented so that an &Annotation can be used interchangeably with an ID
/// for some client methods. For convenience.
impl From<&Annotation> for ID {
    fn from(ann: &Annotation) -> Self {
        ann.id
    }
}

/// Intermediary struct for deserializing a list of annotations.
#[derive(Deserialize, Debug)]
pub(crate) struct AnnotationRows {
    pub rows: Annotations,
}

/// Represents an annotation to be created (hence no ID yet).
/// Fields are defined as in a full annotation.
#[derive(Serialize, Debug)]
pub struct NewAnnotation {
    /// TODO, XXX: quote must not be an empty string.
    pub quote: String,
    pub ranges: Vec<Range>,
    pub text: String,
}

/// Range as used in an `Annotation`. Shows where the annotation is in the
/// content. Part of Annotationjs annotation format. I quote from their docs for the field
/// descriptions.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    /// (relative) XPath to start element.
    pub start: Option<String>,

    /// (relative) XPath to end element.
    pub end: Option<String>,

    /// Character offset within start element.  Note: these offset values have been observed as
    /// literal strings and integers. Grrr loosely typed languages with coercion...
    #[serde(deserialize_with = "parse_stringint")]
    pub start_offset: u32,

    /// Character offset within end element.
    #[serde(deserialize_with = "parse_stringint")]
    pub end_offset: u32,
}
