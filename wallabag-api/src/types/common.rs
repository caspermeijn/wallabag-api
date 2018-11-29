use std::fmt;

use serde_derive::{Deserialize, Serialize};

/// The type used as an ID for all data structures. Declared for clarity.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ID(pub u32);

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// For convenience.
impl From<u32> for ID {
    fn from(x: u32) -> Self {
        ID(x)
    }
}
