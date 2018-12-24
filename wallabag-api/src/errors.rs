//! Client error types.

use std::error::Error;
use std::fmt;

use reqwest::StatusCode;
use serde::Deserialize;

pub type ClientResult<T> = std::result::Result<T, ClientError>;

/// Type for the JSON error data returned on error from the HTTP API
/// The API seems a bit unreliable on the format of errors returned...
#[derive(Deserialize, Debug)]
pub struct ResponseError {
    pub error: String,
    pub error_description: String,
}

/// Type for the JSON error data returned on forbidden error from HTTP API
#[derive(Deserialize, Debug)]
pub struct ResponseCodeMessageError {
    pub error: CodeMessage,
}

#[derive(Deserialize, Debug)]
pub struct CodeMessage {
    pub code: u32,
    pub message: String,
}

/// Represents all error possibilities that could be returned by the client.
#[derive(Debug)]
pub enum ClientError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::error::Error),
    Unauthorized(ResponseError),
    Forbidden(ResponseCodeMessageError),
    ExpiredToken,
    UnexpectedJsonStructure, // eg returned valid json but didn't fit model
    NotFound(ResponseCodeMessageError), // 404
    NotModified,             // 304
    Other(StatusCode, String), // ¯\_(ツ)_/¯
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ClientError::*;
        write!(
            f,
            "{}",
            match self {
                SerdeJsonError(e) => format!("Error deserializing json: {}", e),
                e => format!("{:?}", e),
            }
        )
    }
}

impl Error for ClientError {}

// TODO: extract reqwest errors and turn them into more useful ClientErrors
// TODO: maybe impl Error::cause to get the underlying reqwest or serde errors?

// so we can use ? with reqwest in methods and still return ClientError
impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        ClientError::ReqwestError(err)
    }
}

impl From<serde_json::error::Error> for ClientError {
    fn from(err: serde_json::error::Error) -> Self {
        ClientError::SerdeJsonError(err)
    }
}

/// Represents possible errors building a `TagString`.
#[derive(Debug)]
pub enum TagStringError {
    ContainsComma,
}

impl fmt::Display for TagStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TagStringError::ContainsComma => "Contains comma (invalid character)",
            }
        )
    }
}

impl Error for TagStringError {}
