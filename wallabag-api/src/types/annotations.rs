use serde_derive::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

use super::common::ID;
use crate::utils::serde::parse_stringint;

/// Type alias for clarity.
pub type Annotations = Vec<Annotation>;

/// Represents an annotation as returned from the api.
#[derive(Deserialize, Serialize, Debug)]
pub struct Annotation {
    pub id: ID,
    pub annotator_schema_version: String,
    pub created_at: DateTime<Utc>,
    pub quote: Option<String>,
    pub ranges: Vec<Range>,
    pub text: String,
    pub updated_at: DateTime<Utc>,
    pub user: Option<String>,
}

/// This is implemented so that an Annotation can be used interchangably with an ID
/// for some client methods. For convenience.
impl From<Annotation> for ID {
    fn from(ann: Annotation) -> Self {
        ann.id
    }
}

/// This is implemented so that an &Annotation can be used interchangably with an ID
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
#[derive(Serialize, Debug)]
pub struct NewAnnotation {
    pub quote: String, // TODO: quote must not be empty string?!
    pub ranges: Vec<Range>,
    pub text: String,
}
/// Range as used in an `Annotation`. Shows where the annotation is in the
/// content.
///
/// TODO: research what the fields mean.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub end: Option<String>,
    pub start: Option<String>,

    // these values have been observed as literal strings and integers
    #[serde(deserialize_with = "parse_stringint")]
    pub end_offset: u32,
    #[serde(deserialize_with = "parse_stringint")]
    pub start_offset: u32,
}
