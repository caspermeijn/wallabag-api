//! Custom utility functions to help with serializing/deserializing values from the API that want
//! to be difficult.

use std::collections::HashMap;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serializer};

/// Used to serialize the boolean values to pseudo-bool integers. The api appears to support actual
/// bool, but probably should follow the api docs just in case.
///
/// Note: needs to accept a reference even though it's more efficient to copy. It's how serde
/// works.
#[allow(clippy::trivially_copy_pass_by_ref)]
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

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum StringInt {
    Int(u32),
    String(String),
}

/// Parser for converting integers or integers stored as strings to integers.
pub(crate) fn parse_stringint<'de, D>(d: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let x = StringInt::deserialize(d)?;

    match x {
        StringInt::Int(i) => Ok(i),
        StringInt::String(s) => match s.parse::<u32>() {
            Ok(i) => Ok(i),
            Err(_) => Err(DeError::custom(format!(
                "Could not deserialize {} as u32",
                s
            ))),
        },
    }
}

/// Parser for a hashmap where null values are skipped
pub(crate) fn parse_hashmap_with_null_values<'de, D>(d: D) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let x = Option::<HashMap::<String, Option<String>>>::deserialize(d)?;

    Ok(x.map(|hash_map| {
        hash_map.iter().filter_map(|(key, value)| {
            if let Some(value) = value {
                Some((key.clone(), value.clone()))
            } else {
                None
            }
        }).collect()
    }))
}
