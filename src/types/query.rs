use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
// Represents parameters for the entry checking endpoint.
pub(crate) struct EntriesExistParams {
    pub return_id: usize,
    pub urls: Vec<String>,
}
