use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// The type used as an ID for all data structures. Declared for clarity.
#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ID(pub i64);

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ID {
    pub fn as_int(self) -> i64 {
        self.0
    }
}

/// For convenience.
impl From<i32> for ID {
    fn from(x: i32) -> Self {
        ID(i64::from(x))
    }
}

/// For convenience.
impl From<i64> for ID {
    fn from(x: i64) -> Self {
        ID(x)
    }
}

/// For convenience. Automatic type coercion means that an `&ID` can be passed
/// as an argument to a function that takes a `u32`. Hopefully will make it easier
/// to work with `ID` values in the structs.
impl Deref for ID {
    type Target = i64;

    fn deref(&self) -> &i64 {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foo(arg: &i64) {}

    #[test]
    fn test_flexible_id() {
        assert_eq!(*ID(234), 234);
        assert_eq!(ID(234), 234.into());
        foo(&ID(234));
    }
}
