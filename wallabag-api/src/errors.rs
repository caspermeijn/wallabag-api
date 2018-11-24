pub type ClientResult<T> = std::result::Result<T, ClientError>;

#[derive(Debug)]
pub enum ClientError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::error::Error),

    OtherError,
}

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
