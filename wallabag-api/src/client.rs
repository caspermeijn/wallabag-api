// std libs
use std::collections::HashMap;

// extern crates
use reqwest::{self, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{from_value, Value};

// local imports
use crate::errors::{ClientError, ClientResult, ResponseError};
use crate::types::{Annotations, AuthInfo, Config, Entries, Entry, PaginatedEntries, TokenInfo};
use crate::utils::{EndPoint, UrlBuilder};

/// The main thing that provides all the methods for interacting with the
/// wallabag api.
pub struct Client {
    auth_info: AuthInfo,
    token_info: Option<TokenInfo>,
    url: UrlBuilder,
    client: reqwest::Client,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            auth_info: config.auth_info,
            token_info: None,
            url: UrlBuilder::new(config.base_url),
            client: reqwest::Client::new(),
        }
    }

    /// Internal method to get a valid access token. If no access token loaded
    /// yet, then get a new one.
    fn get_token(&mut self) -> ClientResult<String> {
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
    fn load_token(&mut self) -> ClientResult<()> {

        let mut fields = HashMap::new();
        fields.insert("grant_type".to_owned(), "password".to_owned());
        fields.insert("client_id".to_owned(), self.auth_info.client_id.clone());
        fields.insert(
            "client_secret".to_owned(),
            self.auth_info.client_secret.clone(),
        );
        fields.insert("username".to_owned(), self.auth_info.username.clone());
        fields.insert("password".to_owned(), self.auth_info.password.clone());

        let token_info: TokenInfo = self.json_q(
            Method::POST,
            EndPoint::Token,
            &HashMap::new(),
            &fields,
            false,
        )?;
        self.token_info = Some(token_info);

        Ok(())
    }

    /// Use saved token if present to get a fresh access token.
    fn refresh_token(&mut self) -> ClientResult<()> {
        if self.token_info.is_none() {
            return self.load_token();
        }

        let mut fields = HashMap::new();
        fields.insert("grant_type".to_owned(), "refresh_token".to_owned());
        fields.insert("client_id".to_owned(), self.auth_info.client_id.clone());
        fields.insert(
            "client_secret".to_owned(),
            self.auth_info.client_secret.clone(),
        );
        fields.insert(
            "refresh_token".to_owned(),
            self.token_info.as_ref().unwrap().refresh_token.clone(),
        );

        let token_info: TokenInfo = self.json_q(
            Method::POST,
            EndPoint::Token,
            &HashMap::new(),
            &fields,
            false,
        )?;
        self.token_info = Some(token_info);

        Ok(())
    }

    /// Smartly run a request that expects to receive json back. Handles adding
    /// authorization headers, and retry on expired token.
    /// TODO: more abstract types for query and json
    fn smart_json_q<T>(
        &mut self,
        method: Method,
        end_point: EndPoint,
        query: &HashMap<String, String>,
        json: &HashMap<String, String>,
    ) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        let response_result = self.json_q(method.clone(), end_point, &query, json, true);

        match response_result {
            Err(ClientError::Unauthorized(ref err)) => {
                if err.error_description.as_str().contains("expired") {
                    // let's just try refreshing the token
                    self.refresh_token()?;

                    // try the request again now
                    return Ok(self.json_q(method, end_point, &query, json, true)?);
                }
            }
            _ => (),
        }

        Ok(response_result?)
    }

    /// Just build and send a single request.
    fn json_q<T>(
        &mut self,
        method: Method,
        end_point: EndPoint,
        query: &HashMap<String, String>,
        json: &HashMap<String, String>,
        use_token: bool,
    ) -> ClientResult<T>
    where
        T: DeserializeOwned,
    {
        let url = self.url.build(end_point);

        let mut request = self.client.request(method, &url).query(&query).json(&json);

        if use_token {
            request = request.header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.get_token()?),
            );
        }

        let mut response = request.send()?;

        // main error handling here
        // TODO: handle more cases
        match response.status() {
            StatusCode::UNAUTHORIZED => {
                let info: ResponseError = response.json()?;
                if info.error_description.as_str().contains("expired") {
                    return Err(ClientError::ExpiredToken);
                } else {
                    return Err(ClientError::Unauthorized(info));
                }
            }
            _ => (),
        }

        Ok(response.error_for_status()?.json()?)
    }

    /// Get an entry by id.
    pub fn get_entry(&mut self, id: u32) -> ClientResult<Entry> {
        let json: Value = self.smart_json_q(
            Method::GET,
            EndPoint::Entry(id),
            &HashMap::new(),
            &HashMap::new(),
        )?;

        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Get all annotations for an entry (by id).
    pub fn get_annotations(&mut self, id: u32) -> ClientResult<Annotations> {
        let json: Value = self.smart_json_q(
            Method::GET,
            EndPoint::Annotations(id),
            &HashMap::new(),
            &HashMap::new(),
        )?;

        // extract the embedded annotations vec from the Value
        match json {
            Value::Object(map) => {
                if let Some(Value::Array(vec)) = map.get("rows") {
                    return Ok(from_value(Value::Array(vec.to_vec()))?);
                }
            }
            _ => (),
        }
        Err(ClientError::UnexpectedJsonStructure)
    }

    /// Get all entries. TODO: filters
    pub fn get_entries(&mut self) -> ClientResult<Entries> {
        let mut params = HashMap::new();

        let mut entries = Entries::new();

        // loop to handle pagination. No other api endpoints paginate so it's
        // fine here.
        loop {
            let json: PaginatedEntries =
                self.smart_json_q(Method::GET, EndPoint::Entries, &params, &HashMap::new())?;
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
