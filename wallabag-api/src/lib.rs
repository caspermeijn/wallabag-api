mod types;
mod utils;

// std libs
use std::collections::HashMap;
use std::result::Result;
use std::sync::Mutex;
use std::thread;
use std::time;

// crates
use reqwest;
use serde_derive::Deserialize;
use serde_json::Value;

use crate::utils::{EndPoint, UrlBuilder};

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    ReqwestError(reqwest::Error),
    TokenExpired,
}

// so we can use ? with reqwest in methods and still return ApiError
impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> ApiError {
        ApiError::ReqwestError(err)
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

    /// Add the authorization headers and attempt to send. Will retry if token
    /// expired and able to refresh the token.
    /// TODO: at the moment this only works with json responses
    fn json_q(
        &mut self,
        verb: Verb,
        end_point: EndPoint,
        query: &Option<String>,
        json: &Option<String>,
    ) -> ApiResult<Value> {
        let parsed_response: Value = self.json_q_(verb, end_point, query, json)?;

        match parsed_response {
            Value::Object(ref map) => {
                if map.get("error") == Some(&Value::String("invalid_grant".to_owned()))
                    && map.get("error_description")
                        == Some(&Value::String("The access token expired".to_owned()))
                {
                    self.refresh_token()?;
                    // try again
                    let parsed_response: Value = self.json_q_(verb, end_point, query, json)?;
                    return Ok(parsed_response);
                }
            }
            _ => (),
        }

        Ok(parsed_response)
    }

    fn json_q_(
        &mut self,
        verb: Verb,
        end_point: EndPoint,
        query: &Option<String>,
        json: &Option<String>,
    ) -> ApiResult<Value> {
        let url = self.build_url(end_point);

        let mut request = match verb {
            Verb::Get => self.client.get(url.as_str()),
            _ => self.client.post(url.as_str()),
            // TODO: handle all
        };

        // TODO: query params and json body

        let mut response = request.header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.get_token()?),
        ).send()?;

        Ok(response.json()?)
    }

    fn build_url(&self, end_point: EndPoint) -> String {
        self.url.build(end_point)
    }

    pub fn get_entry(&mut self, id: u32) -> ApiResult<Entry> {
        let json: Value = self.json_q(Verb::Get, EndPoint::Entry(id), &None, &None)?;

        println!("{:?}", json);

        // TODO
        Err(ApiError::TokenExpired)
    }

    pub fn get_entries(&mut self) -> ApiResult<Entries> {
        let url = self.url.build(EndPoint::Entries);

        // TODO
        Ok(vec![])
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

pub type Entries = Vec<Entry>;

#[derive(Deserialize, Debug)]
pub struct Entry {
    todo: String,
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
