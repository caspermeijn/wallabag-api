mod types;
mod utils;

// std libs
use std::collections::HashMap;
use std::result::Result;
use std::sync::Mutex;
use std::thread;
use std::time;

// crates
use reqwest::{self, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_derive::Deserialize;
use serde_json;
use serde_json::{from_value, Value};

use crate::types::{Entries, Entry, PaginatedEntries};
use crate::utils::{EndPoint, UrlBuilder};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::error::Error),

    OtherError,
}

// so we can use ? with reqwest in methods and still return ApiError
impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> ApiError {
        ApiError::ReqwestError(err)
    }
}

impl From<serde_json::error::Error> for ApiError {
    fn from(err: serde_json::error::Error) -> ApiError {
        ApiError::SerdeJsonError(err)
    }
}

pub struct API {
    auth_info: AuthInfo,
    token_info: Option<TokenInfo>,
    url: UrlBuilder,
    client: reqwest::Client,
}

impl API {
    pub fn new(config: Config) -> Self {
        API {
            auth_info: config.auth_info,
            token_info: None,
            url: UrlBuilder::new(config.base_url),
            client: reqwest::Client::new(),
        }
    }

    /// Internal method to get a valid access token. If no access token loaded
    /// yet, then get a new one.
    fn get_token(&mut self) -> ApiResult<String> {
        match self.token_info {
            Some(ref t) => Ok(t.access_token.clone()),
            None => {
                self.load_token()?;
                Ok(self
                    .token_info
                    .as_ref()
                    .expect("load_token() should have populated it")
                    .access_token
                    .clone())
            }
        }
    }

    /// Use credentials in the config to obtain an access token.
    fn load_token(&mut self) -> ApiResult<()> {
        let url = self.url.build(EndPoint::Token);

        let mut fields = HashMap::new();

        fields.insert("grant_type", "password");
        fields.insert("client_id", &self.auth_info.client_id);
        fields.insert("client_secret", &self.auth_info.client_secret);
        fields.insert("username", &self.auth_info.username);
        fields.insert("password", &self.auth_info.password);

        let client = reqwest::Client::new();
        let mut res = client.post(&url).json(&fields).send()?;

        let token_info: TokenInfo = res.json()?;
        self.token_info = Some(token_info);

        Ok(())
    }

    /// Use saved token if present to get a fresh access token.
    fn refresh_token(&mut self) -> ApiResult<()> {
        if self.token_info.is_none() {
            return self.load_token();
        }

        let url = self.url.build(EndPoint::Token);

        let mut fields = HashMap::new();

        fields.insert("grant_type", "refresh_token");
        fields.insert("client_id", &self.auth_info.client_id);
        fields.insert("client_secret", &self.auth_info.client_secret);
        fields.insert(
            "refresh_token",
            self.token_info.as_ref().unwrap().refresh_token.as_ref(),
        );

        let client = reqwest::Client::new();
        let mut res = client.post(&url).json(&fields).send()?;

        let token_info: TokenInfo = res.json()?;
        self.token_info = Some(token_info);

        Ok(())
    }

    /// Smartly run a request that expects to receive json back. Handles adding
    /// authorization headers, and retry on expired token.
    /// TODO: more abstract types for query and json
    fn json_q<T>(
        &mut self,
        verb: Verb,
        end_point: EndPoint,
        query: &HashMap<String, String>,
        json: &HashMap<String, String>,
    ) -> ApiResult<T>
    where
        T: DeserializeOwned,
    {
        let mut response = self.simple_json_q(verb, end_point, &query, json)?;

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                let err: ResponseError = response.json()?;
                if err.error_description.as_str().contains("expired") {
                    // let's just try refreshing the token
                    self.refresh_token()?;

                    // try the request again now
                    response = self.simple_json_q(verb, end_point, &query, json)?;
                }
            }
            _ => (),
        }

        Ok(response.error_for_status()?.json()?)
    }

    /// Just build and send a single request.
    fn simple_json_q(
        &mut self,
        verb: Verb,
        end_point: EndPoint,
        query: &HashMap<String, String>,
        json: &HashMap<String, String>,
    ) -> ApiResult<reqwest::Response> {
        let url = self.url.build(end_point);

        let request = match verb {
            Verb::Get => self.client.get(url.as_str()),
            _ => self.client.post(url.as_str()),
            // TODO: handle all verbs
        };

        let response = request
            .query(&query)
            .json(&json)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.get_token()?),
            )
            .send()?;

        Ok(response)
    }

    pub fn get_entry(&mut self, id: u32) -> ApiResult<Entry> {
        let json: Value = self.json_q(
            Verb::Get,
            EndPoint::Entry(id),
            &HashMap::new(),
            &HashMap::new(),
        )?;

        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Get all entries. TODO: filters
    pub fn get_entries(&mut self) -> ApiResult<Entries> {
        let mut params = HashMap::new();

        let mut entries = Entries::new();

        // loop to handle pagination. No other api endpoints paginate so it's
        // fine here.
        loop {
            let json: PaginatedEntries =
                self.json_q(Verb::Get, EndPoint::Entries, &params, &HashMap::new())?;
            println!("{}", json.page);

            entries.extend(json._embedded.items.into_iter());

            if json.page >= json.pages {
                break;
            } else {
                // otherwise next page
                params.insert("page".to_owned(), (json.page + 1).to_string());
            }
        }

        println!("{:#?}", entries);

        Ok(entries)
    }
}

#[derive(Debug, Clone, Copy)]
enum Verb {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

#[derive(Deserialize, Debug)]
struct TokenInfo {
    access_token: String,
    expires_in: u32,
    token_type: String,
    scope: Option<String>,
    refresh_token: String,
}

#[derive(Debug)]
pub struct AuthInfo {
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Config {
    pub auth_info: AuthInfo,
    pub base_url: String,
}

/// Type for the json error data returned on error from the http api
#[derive(Deserialize, Debug)]
pub struct ResponseError {
    pub error: String,
    pub error_description: String,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
