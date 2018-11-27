
use serde::Serializer;
use serde_derive::{Deserialize, Serialize};

/// Used to serialize the boolean values to pseudo-bool integers. The api
/// appears to support actual bool, but probably should follow the api docs just
/// in case.
pub(crate) fn bool_to_int<S>(x: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(x) => serializer.serialize_some(if *x { &1 } else { &0 }),
        None => serializer.serialize_none(),
    }
}

