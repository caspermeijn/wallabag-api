use serde_derive::Deserialize;

pub type ClientResult<T> = std::result::Result<T, ClientError>;

/// Type for the json error data returned on error from the http api
#[derive(Deserialize, Debug)]
pub struct ResponseError {
    pub error: String,
    pub error_description: String,
}

#[derive(Debug)]
pub enum ClientError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::error::Error),
    Unauthorized(ResponseError),
    ExpiredToken,
    UnexpectedJsonStructure, // eg returned valid json but didn't fit model
    NotFound,  // 404
}

// TODO: extract reqwest errors and turn them into more useful ClientErrors

// so we can use ? with reqwest in methods and still return ClientError
impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> ClientError {
        ClientError::ReqwestError(err)
    }
}

impl From<serde_json::error::Error> for ClientError {
    fn from(err: serde_json::error::Error) -> ClientError {
        ClientError::SerdeJsonError(err)
    }
}