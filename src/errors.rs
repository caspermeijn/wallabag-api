//! Client error types.

use std::error::Error;
use std::fmt;

use serde::Deserialize;
use serde_urlencoded;
use surf::http::StatusCode;
use surf::{self, http::url};

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
    SurfError(surf::Error),
    SerdeJsonError(serde_json::error::Error),
    Unauthorized(ResponseError),
    Forbidden(ResponseCodeMessageError),
    ExpiredToken,
    IOError(std::io::Error),
    UrlParseError(url::ParseError),
    UrlEncodeError(serde_urlencoded::ser::Error),
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

// TODO: extract surf errors and turn them into more useful ClientErrors
// TODO: maybe impl Error::cause to get the underlying surf or serde errors?

impl From<serde_json::error::Error> for ClientError {
    fn from(err: serde_json::error::Error) -> Self {
        ClientError::SerdeJsonError(err)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::IOError(err)
    }
}

impl From<url::ParseError> for ClientError {
    fn from(err: url::ParseError) -> Self {
        ClientError::UrlParseError(err)
    }
}

impl From<surf::Error> for ClientError {
    fn from(err: surf::Error) -> Self {
        ClientError::SurfError(err)
    }
}

impl From<serde_urlencoded::ser::Error> for ClientError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        ClientError::UrlEncodeError(err)
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
