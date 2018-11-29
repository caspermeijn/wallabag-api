use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serializer};
use serde_derive::Deserialize;

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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum IntBool {
    Int(u32),
    Bool(bool),
}

/// Parser for converting pseudo-bool values from 0 or 1 integers to actual bool.
/// Also handles actual bool for robustness.
pub(crate) fn parse_intbool<'de, D>(d: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let x = IntBool::deserialize(d)?;

    match x {
        IntBool::Int(i) => match i {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(DeError::custom(format!(
                "Could not deserialize {} as bool",
                i
            ))),
        },
        IntBool::Bool(b) => Ok(b),
    }
}