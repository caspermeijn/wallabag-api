// std libs
use std::collections::HashMap;

// extern crates
use reqwest::{self, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::{from_value, Value};

// local imports
use crate::errors::{ClientError, ClientResult, ResponseError};
use crate::types::{
    Annotation, Annotations, AuthInfo, Config, DeletedEntry, DeletedTag, Entries, Entry,
    ExistsResponse, NewAnnotation, NewEntry, NewlyRegisteredInfo, PaginatedEntries, PatchEntry,
    RegisterInfo, Tag, Tags, TokenInfo, User, UNIT,
};
use crate::utils::{EndPoint, Format, UrlBuilder};

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
    fn smart_text_q<J, Q>(
        &mut self,
        method: Method,
        end_point: EndPoint,
        query: &Q,
        json: &J,
    ) -> ClientResult<String>
    where
        J: Serialize + ?Sized,
        Q: Serialize + ?Sized,
    {
        Ok(self.smart_q(method, end_point, query, json)?.text()?)
    }

    /// Smartly run a request that expects to receive json back. Handles adding
    /// authorization headers, and retry on expired token.
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
        Ok(self.smart_q(method, end_point, query, json)?.json()?)
    }

    /// Smartly run a request that expects to receive json back. Handles adding
    /// authorization headers, and retry on expired token.
    fn smart_q<J, Q>(
        &mut self,
        method: Method,
        end_point: EndPoint,
        query: &Q,
        json: &J,
    ) -> ClientResult<Response>
    where
        J: Serialize + ?Sized,
        Q: Serialize + ?Sized,
    {
        let response_result = self.q(method.clone(), end_point, query, json, true);

        match response_result {
            Err(ClientError::Unauthorized(ref err)) => {
                if err.error_description.as_str().contains("expired") {
                    // let's just try refreshing the token
                    self.refresh_token()?;

                    // try the request again now
                    return Ok(self.q(method, end_point, query, json, true)?);
                }
            }
            _ => (),
        }

        Ok(response_result?)
    }

    /// Just build and send a single request. Returns a json deserializable
    /// response.
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
        Ok(self.q(method, end_point, query, json, use_token)?.json()?)
    }

    /// Just build and send a single request.
    fn q<J, Q>(
        &mut self,
        method: Method,
        end_point: EndPoint,
        query: &Q,
        json: &J,
        use_token: bool,
    ) -> ClientResult<Response>
    where
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
                println!("{:#?}", response.text());
                return Err(ClientError::NotFound);
            }
            _ => (),
            // TODO: try to parse json error message on error status
        }

        // TODO: don't error for status
        Ok(response.error_for_status()?)
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

    /// Update entry. To leave an editable field unchanged, set to `None`.
    pub fn update_entry(&mut self, id: u32, entry: &PatchEntry) -> ClientResult<Entry> {

        let json: Value =
            self.smart_json_q(Method::PATCH, EndPoint::Entry(id), UNIT, entry)?;

        println!("{:#?}", json);
        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Get an entry by id.
    pub fn get_entry(&mut self, id: u32) -> ClientResult<Entry> {
        let json: Value = self.smart_json_q(Method::GET, EndPoint::Entry(id), UNIT, UNIT)?;

        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Delete an entry by id.
    /// TODO: allow passing a u32 id or an Entry interchangably
    pub fn delete_entry(&mut self, id: u32) -> ClientResult<Entry> {
        let json: DeletedEntry =
            self.smart_json_q(Method::DELETE, EndPoint::Entry(id), UNIT, UNIT)?;

        // build an entry composed of the deleted entry returned and the id
        let entry = Entry {
            id,
            annotations: json.annotations,
            content: json.content,
            created_at: json.created_at,
            domain_name: json.domain_name,
            headers: json.headers,
            http_status: json.http_status,
            is_archived: json.is_archived,
            is_public: json.is_public,
            is_starred: json.is_starred,
            language: json.language,
            mimetype: json.mimetype,
            origin_url: json.origin_url,
            preview_picture: json.preview_picture,
            published_at: json.published_at,
            published_by: json.published_by,
            reading_time: json.reading_time,
            starred_at: json.starred_at,
            tags: json.tags,
            title: json.title,
            uid: json.uid,
            updated_at: json.updated_at,
            url: json.url,
            user_email: json.user_email,
            user_id: json.user_id,
            user_name: json.user_name,
        };

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

        Ok(entries)
    }

    /// Get an export of an entry in a particular format.
    pub fn export_entry(&mut self, entry_id: u32, fmt: Format) -> ClientResult<String> {
        let data = self.smart_text_q(Method::GET, EndPoint::Export(entry_id, fmt), UNIT, UNIT)?;
        Ok(data)
    }

    /// Get a list of all tags for an entry by entry id.
    pub fn get_tags_for_entry(&mut self, entry_id: u32) -> ClientResult<Tags> {
        self.smart_json_q(Method::GET, EndPoint::EntryTags(entry_id), UNIT, UNIT)
    }

    /// Get a list of all tags.
    pub fn get_tags(&mut self) -> ClientResult<Tags> {
        self.smart_json_q(Method::GET, EndPoint::Tags, UNIT, UNIT)
    }

    /// Permanently delete a tag by id. This removes the tag from all entries.
    /// Appears to return success if attempting to delete a tag by id that
    /// exists on the server but isn't accessible to the user. TODO: log
    /// security ticket.
    pub fn delete_tag(&mut self, id: u32) -> ClientResult<Tag> {
        // api does not return id of deleted tag, hence the temporary struct
        let dt: DeletedTag = self.smart_json_q(Method::DELETE, EndPoint::Tag(id), UNIT, UNIT)?;

        Ok(Tag {
            id,
            label: dt.label,
            slug: dt.slug,
        })
    }

    /// Permanently delete a tag by label (tag names). This also exhibits the
    /// privacy breaching behaviour of returning tag info of other users' tags.
    /// Also, labels aren't necessarily unique across a wallabag installation.
    /// The server should filter by tags belonging to a user in the same db
    /// query.
    pub fn delete_tag_by_label(&mut self, label: String) -> ClientResult<DeletedTag> {
        let mut params = HashMap::new();
        params.insert("tag".to_owned(), label);

        let deleted_tag: DeletedTag =
            self.smart_json_q(Method::DELETE, EndPoint::TagLabel, &params, UNIT)?;
        Ok(deleted_tag)
    }

    /// Permanently batch delete tags by labels (tag names). Returns not found
    /// if _all_ labels not found. If at least one found, then returns ok. For
    /// some reason, (at least the framabag instance) the server returns success
    /// and the tag data on attempting to delete for innaccessible tags (tags by
    /// other users?).
    ///
    /// Returns a list of tags that were deleted.
    pub fn delete_tags_by_label(&mut self, tags: Vec<String>) -> ClientResult<Vec<DeletedTag>> {
        let tags = tags.join(",");
        let mut params = HashMap::new();
        params.insert("tags".to_owned(), tags);

        // note: api doesn't return tag ids and no way to obtain since deleted
        // by label
        let json: Vec<DeletedTag> =
            self.smart_json_q(Method::DELETE, EndPoint::TagsLabel, &params, UNIT)?;
        Ok(json)
    }

    /// Get the API version. Probably not useful because if the version isn't v2
    /// then this library won't work anyway.
    pub fn get_api_version(&mut self) -> ClientResult<String> {
        let version: String = self.smart_json_q(Method::GET, EndPoint::Version, UNIT, UNIT)?;
        Ok(version)
    }

    /// Get the currently logged in user information.
    pub fn get_user(&mut self) -> ClientResult<User> {
        let user: User = self.smart_json_q(Method::GET, EndPoint::User, UNIT, UNIT)?;
        Ok(user)
    }

    /// Register a user and create a client.
    pub fn register_user(&mut self, info: &RegisterInfo) -> ClientResult<NewlyRegisteredInfo> {
        let info: NewlyRegisteredInfo =
            self.json_q(Method::PUT, EndPoint::User, UNIT, info, false)?;
        Ok(info)
    }
}
