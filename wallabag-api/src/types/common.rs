use std::fmt;
use std::ops::Deref;

use serde_derive::{Deserialize, Serialize};

/// The type used as an ID for all data structures. Declared for clarity.
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ID(pub u32);

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ID {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// For convenience.
impl From<u32> for ID {
    fn from(x: u32) -> Self {
        ID(x)
    }
}

/// For convenience. Automatic type coercion means that an `&ID` can be passed
/// as an argument to a function that takes a `u32`. Hopefully will make it easier
/// to work with `ID` values in the structs.
impl Deref for ID {
    type Target = u32;

    fn deref(&self) -> &u32 {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foo(arg: &u32) {}

    #[test]
    fn test_flexible_id() {
        assert_eq!(*ID(234), 234);
        assert_eq!(ID(234), 234.into());
        foo(&ID(234));
    }
}
