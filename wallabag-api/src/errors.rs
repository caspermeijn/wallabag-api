use std::error::Error;
use std::fmt;

use reqwest::StatusCode;
use serde_derive::Deserialize;

pub type ClientResult<T> = std::result::Result<T, ClientError>;

/// Type for the json error data returned on error from the http api
#[derive(Deserialize, Debug)]
pub struct ResponseError {
    pub error: String,
    pub error_description: String,
}

/// Type for the json error data returned on forbidden error from http api
#[derive(Deserialize, Debug)]
pub struct ResponseCodeMessageError {
    pub error: CodeMessage,
}

#[derive(Deserialize, Debug)]
pub struct CodeMessage {
    pub code: u32,
    pub message: String,
}

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
        write!(f, "{}", "¯\\_(ツ)_/¯")
    }
}

impl Error for ClientError {}

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
