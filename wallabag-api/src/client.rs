// std libs
use std::collections::HashMap;

// extern crates
use reqwest::{self, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::{from_value, Value};

// local imports
use crate::errors::{ClientError, ClientResult, ResponseCodeMessageError, ResponseError};
use crate::types::{
    Annotation, Annotations, Config, DeletedEntry, DeletedTag, Entries, EntriesFilter, Entry,
    ExistsInfo, ExistsResponse, Format, NewAnnotation, NewEntry, NewlyRegisteredInfo,
    PaginatedEntries, PatchEntry, RegisterInfo, Tag, Tags, TokenInfo, User, ID, UNIT,
};
use crate::utils::{EndPoint, UrlBuilder};

/// The main thing that provides all the methods for interacting with the
/// wallabag api.
pub struct Client {
    client_id: String,
    client_secret: String,
    username: String,
    password: String,
    token_info: Option<TokenInfo>,
    url: UrlBuilder,
    client: reqwest::Client,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            client_id: config.client_id,
            client_secret: config.client_secret,
            username: config.username,
            password: config.password,
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
        fields.insert("client_id".to_owned(), self.client_id.clone());
        fields.insert("client_secret".to_owned(), self.client_secret.clone());
        fields.insert("username".to_owned(), self.username.clone());
        fields.insert("password".to_owned(), self.password.clone());

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
        fields.insert("client_id".to_owned(), self.client_id.clone());
        fields.insert("client_secret".to_owned(), self.client_secret.clone());
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
            Err(ClientError::ExpiredToken) => {
                self.refresh_token()?;

                // try the request again now
                return Ok(self.q(method, end_point, query, json, true)?);
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

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                let info: ResponseError = response.json()?;
                if info.error_description.as_str().contains("expired") {
                    Err(ClientError::ExpiredToken)
                } else {
                    Err(ClientError::Unauthorized(info))
                }
            }
            StatusCode::FORBIDDEN => {
                let info: ResponseCodeMessageError = response.json()?;
                Err(ClientError::Forbidden(info))
            }
            StatusCode::NOT_FOUND => {
                let info: ResponseCodeMessageError = response.json()?;
                Err(ClientError::NotFound(info))
            }
            StatusCode::NOT_MODIFIED => {
                // reload entry returns this if no changes on re-crawl url or if failed to reload
                Err(ClientError::NotModified)
            }
            status if status.is_success() => Ok(response),
            status => Err(ClientError::Other(status, response.text()?)),
        }
    }

    /// Check if a list of urls already have entries. This is more efficient if
    /// you want to batch check urls since only a single request is required.
    pub fn check_exists_vec(&mut self, urls: Vec<String>) -> ClientResult<ExistsInfo> {
        let mut params = vec![];
        params.push(("return_id".to_owned(), "1".to_owned()));

        // workaround: need to structure the params as a list of pairs since Vec
        // values are unsupported:
        // https://github.com/nox/serde_urlencoded/issues/46
        for url in urls.into_iter() {
            params.push(("urls[]".to_owned(), url));
        }

        self.smart_json_q(Method::GET, EndPoint::Exists, &params, UNIT)
    }

    /// check if a url already has an entry recorded.
    pub fn check_exists(&mut self, url: &str) -> ClientResult<Option<ID>> {
        let mut params = HashMap::new();
        params.insert("url".to_owned(), url.to_owned());
        params.insert("return_id".to_owned(), "1".to_owned());

        let exists_info: ExistsResponse =
            self.smart_json_q(Method::GET, EndPoint::Exists, &params, UNIT)?;

        // extract and return the entry id
        Ok(exists_info.exists)
    }

    /// Add a new entry
    pub fn create_entry(&mut self, new_entry: &NewEntry) -> ClientResult<Entry> {
        let json: Value = self.smart_json_q(Method::POST, EndPoint::Entries, UNIT, new_entry)?;

        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Update entry. To leave an editable field unchanged, set to `None`.
    pub fn update_entry<T: Into<ID>>(&mut self, id: T, entry: &PatchEntry) -> ClientResult<Entry> {
        let json: Value =
            self.smart_json_q(Method::PATCH, EndPoint::Entry(id.into()), UNIT, entry)?;

        println!("{:#?}", json);
        let entry = from_value(json)?;

        Ok(entry)
    }

    /// Reload entry.
    pub fn reload_entry<T: Into<ID>>(&mut self, id: T) -> ClientResult<Entry> {
        let entry: Entry =
            self.smart_json_q(Method::PATCH, EndPoint::EntryReload(id.into()), UNIT, UNIT)?;

        Ok(entry)
    }

    /// Get an entry by id.
    pub fn get_entry<T: Into<ID>>(&mut self, id: T) -> ClientResult<Entry> {
        self.smart_json_q(Method::GET, EndPoint::Entry(id.into()), UNIT, UNIT)
    }

    /// Delete an entry by id.
    /// TODO: allow passing a u32 id or an Entry interchangably
    pub fn delete_entry<T: Into<ID>>(&mut self, id: T) -> ClientResult<Entry> {
        let id = id.into();
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
    pub fn create_annotation<T: Into<ID>>(
        &mut self,
        entry_id: T,
        annotation: NewAnnotation,
    ) -> ClientResult<Annotation> {
        let json: Annotation = self.smart_json_q(
            Method::POST,
            EndPoint::Annotation(entry_id.into()),
            UNIT,
            &annotation,
        )?;

        Ok(json)
    }

    /// Delete an annotation by id
    pub fn delete_annotation<T: Into<ID>>(&mut self, id: T) -> ClientResult<Annotation> {
        let json: Annotation =
            self.smart_json_q(Method::DELETE, EndPoint::Annotation(id.into()), UNIT, UNIT)?;

        Ok(json)
    }

    /// Get all annotations for an entry (by id).
    pub fn get_annotations<T: Into<ID>>(&mut self, id: T) -> ClientResult<Annotations> {
        let json: Value =
            self.smart_json_q(Method::GET, EndPoint::Annotation(id.into()), UNIT, UNIT)?;

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

    /// Get all entries.
    pub fn get_entries(&mut self) -> ClientResult<Entries> {
        self._get_entries(EntriesFilter::default())
    }

    /// Get all entries, filtered by filter parameters.
    pub fn get_entries_filtered(&mut self, filter: EntriesFilter) -> ClientResult<Entries> {
        self._get_entries(filter)
    }

    fn _get_entries(&mut self, filter: EntriesFilter) -> ClientResult<Entries> {
        let mut entries = Entries::new();

        let mut filter = filter.clone();
        filter.page = 1; // just to make sure

        // loop to handle pagination. No other api endpoints paginate so it's
        // fine here.
        loop {
            let json: Value = self.smart_json_q(Method::GET, EndPoint::Entries, &filter, UNIT)?;

            println!("{:#?}", json);
            let json: PaginatedEntries = from_value(json)?;
            // unimplemented!();

            entries.extend(json._embedded.items.into_iter());

            if json.page >= json.pages {
                break;
            } else {
                // otherwise next page
                filter.page = json.page + 1;
            }
        }

        Ok(entries)
    }

    /// Get an export of an entry in a particular format.
    pub fn export_entry<T: Into<ID>>(&mut self, entry_id: T, fmt: Format) -> ClientResult<String> {
        let data = self.smart_text_q(
            Method::GET,
            EndPoint::Export(entry_id.into(), fmt),
            UNIT,
            UNIT,
        )?;
        Ok(data)
    }

    /// Get a list of all tags for an entry by entry id.
    pub fn get_tags_for_entry<T: Into<ID>>(&mut self, entry_id: T) -> ClientResult<Tags> {
        self.smart_json_q(
            Method::GET,
            EndPoint::EntryTags(entry_id.into()),
            UNIT,
            UNIT,
        )
    }

    /// Add tags to an entry by entry id. Idempotent operation. No problems if
    /// tags list is empty.
    /// TODO: use types to restrict chars in tags; if a tag contains a comma,
    /// then it will be saved as two tags (eg. 'wat,dis' becomes 'wat' and 'dis'
    /// tags on the server.
    pub fn add_tags_to_entry<T: Into<ID>>(
        &mut self,
        entry_id: ID,
        tags: Vec<String>,
    ) -> ClientResult<Entry> {
        let mut data = HashMap::new();
        data.insert("tags".to_owned(), tags.join(","));

        self.smart_json_q(Method::POST, EndPoint::EntryTags(entry_id), UNIT, &data)
    }

    /// Delete a tag (by id) from an entry (by id). Returns err 404 if entry or
    /// tag not found. Idempotent. Removing a tag that exists but doesn't exist
    /// on the entry completes without error.
    pub fn delete_tag_from_entry<T: Into<ID>, U: Into<ID>>(
        &mut self,
        entry_id: T,
        tag_id: U,
    ) -> ClientResult<Entry> {
        self.smart_json_q(
            Method::DELETE,
            EndPoint::DeleteEntryTag(entry_id.into(), tag_id.into()),
            UNIT,
            UNIT,
        )
    }

    /// Get a list of all tags.
    pub fn get_tags(&mut self) -> ClientResult<Tags> {
        self.smart_json_q(Method::GET, EndPoint::Tags, UNIT, UNIT)
    }

    /// Permanently delete a tag by id. This removes the tag from all entries.
    /// Appears to return success if attempting to delete a tag by id that
    /// exists on the server but isn't accessible to the user. TODO: log
    /// security ticket.
    pub fn delete_tag<T: Into<ID>>(&mut self, id: T) -> ClientResult<Tag> {
        let id = id.into();

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
