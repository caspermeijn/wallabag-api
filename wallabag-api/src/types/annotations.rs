use serde_derive::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

use super::common::ID;

/// Type alias for clarity.
pub type Annotations = Vec<Annotation>;

/// Represents an annotation as returned from the api.
#[derive(Deserialize, Serialize, Debug)]
pub struct Annotation {
    pub annotator_schema_version: String,
    pub created_at: DateTime<Utc>,
    pub id: ID,
    pub quote: String,
    pub ranges: Vec<Range>,
    pub text: String,
    pub updated_at: DateTime<Utc>,
    pub user: Option<String>,
}

/// Intermediary struct for deserializing a list of annotations.
#[derive(Deserialize, Debug)]
pub(crate) struct AnnotationRows {
    pub rows: Annotations,
}

/// Represents an annotation to be created (hence no ID yet).
#[derive(Serialize, Debug)]
pub struct NewAnnotation {
    pub quote: String,
    pub ranges: Vec<Range>,
    pub text: String,
    pub user: Option<String>,
}
/// Range as used in an `Annotation`. Shows where the annotation is in the
/// content.
///
/// TODO: research what the fields mean.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub end: Option<String>,
    pub end_offset: String,
    pub start: Option<String>,
    pub start_offset: String,
}
