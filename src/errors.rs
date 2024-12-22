// Copyright 2018 Samuel Walladge <samuel@swalladge.net>
// Copyright 2024 Casper Meijn <casper@meijn.net>
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Client error types.

use serde::Deserialize;
use serde_urlencoded;
use surf::http::StatusCode;
use surf::{self, http::url};
use thiserror::Error;

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
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP operation failed")]
    SurfError(surf::Error),
    #[error("Error deserializing json")]
    SerdeJsonError(#[from] serde_json::error::Error),
    #[error("Unauthorized")]
    Unauthorized(ResponseError),
    #[error("Forbidden")]
    Forbidden(ResponseCodeMessageError),
    #[error("Token is expired")]
    ExpiredToken,
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("URL parse error")]
    UrlParseError(#[from] url::ParseError),
    #[error("URL encode error")]
    UrlEncodeError(#[from] serde_urlencoded::ser::Error),
    #[error("Unexpected JSON structure, eg returned valid json but didn't fit model")]
    UnexpectedJsonStructure,
    #[error("Resource not found")]
    NotFound(ResponseCodeMessageError),
    #[error("Resource not modified")]
    NotModified,
    #[error("Unknown status code")]
    Other(StatusCode, String),
}

// TODO: extract surf errors and turn them into more useful ClientErrors
// TODO: maybe impl Error::cause to get the underlying surf or serde errors?

impl From<surf::Error> for ClientError {
    fn from(err: surf::Error) -> Self {
        ClientError::SurfError(err)
    }
}

/// Represents possible errors building a `TagString`.
#[derive(Error, Debug)]
pub enum TagStringError {
    #[error("Contains comma (invalid character)")]
    ContainsComma,
}
