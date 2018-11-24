// std libs
use std::collections::HashMap;

// extern crates
use reqwest::{self, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{from_value, Value};

// local imports
use crate::errors::ClientResult;
use crate::types::{
    AuthInfo, Config, Entries, Entry, PaginatedEntries, ResponseError, TokenInfo, Verb,
};
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
        let url = self.url.build(EndPoint::Token);

        let mut fields = HashMap::new();

        fields.insert("grant_type", "password");
        fields.insert("client_id", &self.auth_info.client_id);
        fields.insert("client_secret", &self.auth_info.client_secret);
        fields.insert("username", &self.auth_info.username);
        fields.insert("password", &self.auth_info.password);

        let client = reqwest::Client::new();
        let mut res = client.post(&url).json(&fields).send()?;

        // println!("{:#?}", res.text()?);
        res = res.error_for_status()?;

        let token_info: TokenInfo = res.json()?;
        self.token_info = Some(token_info);

        Ok(())
    }

    /// Use saved token if present to get a fresh access token.
    fn refresh_token(&mut self) -> ClientResult<()> {
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
        res = res.error_for_status()?;

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
    ) -> ClientResult<T>
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
    ) -> ClientResult<Response> {
        let url = self.url.build(end_point);

        let request = match verb {
            Verb::Get => self.client.get(url.as_str()),
            // _ => self.client.post(url.as_str()),
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

    /// Get an entry by id.
    pub fn get_entry(&mut self, id: u32) -> ClientResult<Entry> {
        let json: Value = self.json_q(
            Verb::Get,
            EndPoint::Entry(id),
            &HashMap::new(),
            &HashMap::new(),
        )?;

        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Get all annotations for an entry (by id).
    pub fn get_annotations(&mut self, id: u32) -> ClientResult<Entry> {
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
    pub fn get_entries(&mut self) -> ClientResult<Entries> {
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
