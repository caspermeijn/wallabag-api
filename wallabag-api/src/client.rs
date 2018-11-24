// std libs
use std::collections::HashMap;

// extern crates
use reqwest::{self, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::{from_value, Value};

// local imports
use crate::errors::{ClientError, ClientResult, ResponseError};
use crate::types::{
    Annotation, Annotations, AuthInfo, Config, Entries, Entry, ExistsResponse, NewAnnotation,
    NewEntry, PaginatedEntries, TokenInfo, UNIT,
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
        let mut fields = HashMap::new();
        fields.insert("grant_type".to_owned(), "password".to_owned());
        fields.insert("client_id".to_owned(), self.auth_info.client_id.clone());
        fields.insert(
            "client_secret".to_owned(),
            self.auth_info.client_secret.clone(),
        );
        fields.insert("username".to_owned(), self.auth_info.username.clone());
        fields.insert("password".to_owned(), self.auth_info.password.clone());

        let token_info: TokenInfo =
            self.json_q(Method::POST, EndPoint::Token, UNIT, &fields, false)?;
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

        let token_info: TokenInfo =
            self.json_q(Method::POST, EndPoint::Token, UNIT, &fields, false)?;
        self.token_info = Some(token_info);

        Ok(())
    }

    /// Smartly run a request that expects to receive json back. Handles adding
    /// authorization headers, and retry on expired token.
    /// TODO: more abstract types for query and json
    fn smart_json_q<T, J, Q>(
        &mut self,
        method: Method,
        end_point: EndPoint,
        query: &Q,
        json: &J,
    ) -> ClientResult<T>
    where
        T: DeserializeOwned,
        J: Serialize + ?Sized,
        Q: Serialize + ?Sized,
    {
        let response_result = self.json_q(method.clone(), end_point, query, json, true);

        match response_result {
            Err(ClientError::Unauthorized(ref err)) => {
                if err.error_description.as_str().contains("expired") {
                    // let's just try refreshing the token
                    self.refresh_token()?;

                    // try the request again now
                    return Ok(self.json_q(method, end_point, query, json, true)?);
                }
            }
            _ => (),
        }

        Ok(response_result?)
    }

    /// Just build and send a single request.
    fn json_q<T, J, Q>(
        &mut self,
        method: Method,
        end_point: EndPoint,
        query: &Q,
        json: &J,
        use_token: bool,
    ) -> ClientResult<T>
    where
        T: DeserializeOwned,
        J: Serialize + ?Sized,
        Q: Serialize + ?Sized,
    {
        let url = self.url.build(end_point);

        let mut request = self.client.request(method, &url).query(query).json(json);

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
            StatusCode::NOT_FOUND => {
                return Err(ClientError::NotFound);
            }
            _ => (),
        }

        Ok(response.error_for_status()?.json()?)
    }

    /// Check if a list of urls already have entries. This is more efficient if
    /// you want to batch check urls since only a single request is required.
    pub fn check_exists_vec(
        &mut self,
        urls: Vec<String>,
    ) -> ClientResult<HashMap<String, Option<u32>>> {
        let mut params = vec![];
        params.push(("return_id".to_owned(), "1".to_owned()));

        // workaround: need to structure the params as a list of pairs since Vec
        // values are unsupported:
        // https://github.com/nox/serde_urlencoded/issues/46
        for url in urls.into_iter() {
            params.push(("urls[]".to_owned(), url));
        }

        let exists_info: HashMap<String, Option<u32>> =
            self.smart_json_q(Method::GET, EndPoint::Exists, &params, UNIT)?;

        Ok(exists_info)
    }

    /// check if a url already has an entry recorded.
    pub fn check_exists(&mut self, url: &str) -> ClientResult<Option<u32>> {
        let mut params = HashMap::new();
        params.insert("url".to_owned(), url.to_owned());
        params.insert("return_id".to_owned(), "1".to_owned());

        let exists_info: ExistsResponse =
            self.smart_json_q(Method::GET, EndPoint::Exists, &params, UNIT)?;

        Ok(exists_info.exists)
    }

    /// Add a new entry
    pub fn create_entry(&mut self, new_entry: &NewEntry) -> ClientResult<Entry> {
        let json: Value = self.smart_json_q(Method::POST, EndPoint::Entries, UNIT, new_entry)?;

        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Get an entry by id.
    pub fn get_entry(&mut self, id: u32) -> ClientResult<Entry> {
        let json: Value = self.smart_json_q(Method::GET, EndPoint::Entry(id), UNIT, UNIT)?;

        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Update an annotation.
    pub fn update_annotation(&mut self, annotation: &Annotation) -> ClientResult<Annotation> {
        let json: Annotation = self.smart_json_q(
            Method::PUT,
            EndPoint::Annotation(annotation.id),
            UNIT,
            annotation,
        )?;

        Ok(json)
    }

    /// Create a new annotation on an entry.
    pub fn create_annotation(
        &mut self,
        entry_id: u32,
        annotation: NewAnnotation,
    ) -> ClientResult<Annotation> {
        let json: Annotation = self.smart_json_q(
            Method::POST,
            EndPoint::Annotation(entry_id),
            UNIT,
            &annotation,
        )?;

        Ok(json)
    }

    /// Delete an annotation by id
    pub fn delete_annotation(&mut self, id: u32) -> ClientResult<Annotation> {
        let json: Annotation =
            self.smart_json_q(Method::DELETE, EndPoint::Annotation(id), UNIT, UNIT)?;

        Ok(json)
    }

    /// Get all annotations for an entry (by id).
    pub fn get_annotations(&mut self, id: u32) -> ClientResult<Annotations> {
        let json: Value = self.smart_json_q(Method::GET, EndPoint::Annotation(id), UNIT, UNIT)?;

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
                self.smart_json_q(Method::GET, EndPoint::Entries, &params, UNIT)?;
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

    /// Get the API version. Probably not useful because if the version isn't v2
    /// then this library won't work anyway.
    pub fn get_api_version(&mut self) -> ClientResult<String> {
        let version: String = self.smart_json_q(Method::GET, EndPoint::Version, UNIT, UNIT)?;
        Ok(version)
    }

}
