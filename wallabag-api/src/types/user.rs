use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::types::ID;

/// A struct representing a user.
#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A struct representing a user to be registered. Includes the name for the
/// client to be registered along with.
#[derive(Deserialize, Serialize, Debug)]
pub struct RegisterInfo {
    pub username: String,
    pub password: String,
    pub email: String,
    pub client_name: String,
}

/// A struct representing a newly created user and associated client info.
#[derive(Deserialize, Serialize, Debug)]
pub struct NewlyRegisteredInfo {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub default_client: ClientInfo,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientInfo {
    client_id: String,
    client_secret: String,
    name: String,
}
