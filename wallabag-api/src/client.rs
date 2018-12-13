//! Client.

// std libs
use std::collections::HashMap;

// extern crates
use log::{debug, max_level, trace, LevelFilter};
use reqwest::{self, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

// local imports
use crate::errors::{
    ClientError, ClientResult, CodeMessage, ResponseCodeMessageError, ResponseError,
};
use crate::types::{
    Annotation, AnnotationRows, Annotations, Config, DeletedEntry, DeletedTag, Entries,
    EntriesFilter, Entry, ExistsInfo, ExistsResponse, Format, NewAnnotation, NewEntry,
    NewlyRegisteredInfo, PaginatedEntries, PatchEntry, RegisterInfo, Tag, TagString, Tags,
    TokenInfo, User, ID, UNIT,
};
use crate::utils::{EndPoint, UrlBuilder};

/// The main thing that provides all the methods for interacting with the
/// Wallabag API.
#[derive(Debug)]
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
    /// Build a new client given the configuration.
    pub fn new(config: Config) -> Self {
        Self {
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
        if let Some(ref t) = self.token_info {
            Ok(t.access_token.clone())
        } else {
            debug!("No api token loaded yet");
            self.load_token()
        }
    }

    /// Use credentials in the config to obtain an access token.
    fn load_token(&mut self) -> ClientResult<String> {
        debug!("Requesting auth token");
        let mut fields = HashMap::new();
        fields.insert("grant_type".to_owned(), "password".to_owned());
        fields.insert("client_id".to_owned(), self.client_id.clone());
        fields.insert("client_secret".to_owned(), self.client_secret.clone());
        fields.insert("username".to_owned(), self.username.clone());
        fields.insert("password".to_owned(), self.password.clone());

        let token_info: TokenInfo =
            self.json_q(Method::POST, EndPoint::Token, UNIT, &fields, false)?;
        self.token_info = Some(token_info);

        Ok(self.token_info.as_ref().unwrap().access_token.clone())
    }

    /// Use saved token if present to get a fresh access token.
    fn refresh_token(&mut self) -> ClientResult<String> {
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

        Ok(self.token_info.as_ref().unwrap().access_token.clone())
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
        if max_level() >= LevelFilter::Debug {
            let text = self.smart_q(method, end_point, query, json)?.text()?;
            match serde_json::from_str(&text) {
                Ok(j) => {
                    debug!("Deserialized json response body: {}", text);
                    Ok(j)
                }
                Err(e) => {
                    debug!("Deserialize json failed for: {}", text);
                    Err(ClientError::SerdeJsonError(e))
                }
            }
        } else {
            Ok(self.smart_q(method, end_point, query, json)?.json()?)
        }
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

        if let Err(ClientError::ExpiredToken) = response_result {
            debug!("Token expired; refreshing");
            self.refresh_token()?;

            // try the request again now
            Ok(self.q(method, end_point, query, json, true)?)
        } else {
            Ok(response_result?)
        }
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
        if max_level() >= LevelFilter::Debug {
            let text = self.q(method, end_point, query, json, use_token)?.text()?;
            match serde_json::from_str(&text) {
                Ok(j) => {
                    debug!("Deserialized json response body: {}", text);
                    Ok(j)
                }
                Err(e) => {
                    debug!("Deserialize json failed for: {}", text);
                    Err(ClientError::SerdeJsonError(e))
                }
            }
        } else {
            Ok(self.q(method, end_point, query, json, use_token)?.json()?)
        }
    }

    /// Build and send a single request. Does most of the heavy lifting.
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
        trace!("Sending request to {}", url);

        let mut request = self.client.request(method, &url).query(query).json(json);

        if use_token {
            request = request.header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.get_token()?),
            );
        }

        let mut response = request.send()?;

        trace!("response status: {:?}", response.status());
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
                println!("NOT FOUND {:?}", response.text());
                let info: ResponseCodeMessageError = match response.json() {
                    Ok(info) => info,
                    Err(_) => ResponseCodeMessageError {
                        error: CodeMessage {
                            code: 404,
                            message: "Not supplied".to_owned(),
                        },
                    },
                };
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
    pub fn batch_check_exists<T: Into<String>>(
        &mut self,
        urls: Vec<T>,
    ) -> ClientResult<ExistsInfo> {
        let mut params = vec![];
        params.push(("return_id".to_owned(), "1".to_owned()));

        // workaround: need to structure the params as a list of pairs since Vec
        // values are unsupported:
        // https://github.com/nox/serde_urlencoded/issues/46
        for url in urls {
            params.push(("urls[]".to_owned(), url.into()));
        }

        self.smart_json_q(Method::GET, EndPoint::Exists, &params, UNIT)
    }

    /// check if a url already has an entry recorded.
    pub fn check_exists<T: Into<String>>(&mut self, url: T) -> ClientResult<Option<ID>> {
        let mut params = HashMap::new();
        params.insert("url".to_owned(), url.into());
        params.insert("return_id".to_owned(), "1".to_owned());

        let exists_info: ExistsResponse =
            self.smart_json_q(Method::GET, EndPoint::Exists, &params, UNIT)?;

        // extract and return the entry id
        Ok(exists_info.exists)
    }

    /// Add a new entry
    pub fn create_entry(&mut self, new_entry: &NewEntry) -> ClientResult<Entry> {
        self.smart_json_q(Method::POST, EndPoint::Entries, UNIT, new_entry)
    }

    /// Update entry. To leave an editable field unchanged, set to `None`.
    pub fn update_entry<T: Into<ID>>(&mut self, id: T, entry: &PatchEntry) -> ClientResult<Entry> {
        self.smart_json_q(Method::PATCH, EndPoint::Entry(id.into()), UNIT, entry)
    }

    /// Reload entry. This tells the server to re-fetch content from the url (or
    /// origin url?) and use the result to refresh the entry contents.
    ///
    /// This returns `Err(ClientError::NotModified)` if the server either could
    /// not refresh the contents, or the content does not get modified.
    pub fn reload_entry<T: Into<ID>>(&mut self, id: T) -> ClientResult<Entry> {
        self.smart_json_q(Method::PATCH, EndPoint::EntryReload(id.into()), UNIT, UNIT)
    }

    /// Get an entry by id.
    pub fn get_entry<T: Into<ID>>(&mut self, id: T) -> ClientResult<Entry> {
        self.smart_json_q(Method::GET, EndPoint::Entry(id.into()), UNIT, UNIT)
    }

    /// Delete an entry by id.
    pub fn delete_entry<T: Into<ID>>(&mut self, id: T) -> ClientResult<Entry> {
        let id = id.into();
        let json: DeletedEntry =
            self.smart_json_q(Method::DELETE, EndPoint::Entry(id), UNIT, UNIT)?;

        // build an entry composed of the deleted entry returned and the id,
        // because the entry returned does not include the id.
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
        self.smart_json_q(
            Method::PUT,
            EndPoint::Annotation(annotation.id),
            UNIT,
            annotation,
        )
    }

    /// Create a new annotation on an entry.
    pub fn create_annotation<T: Into<ID>>(
        &mut self,
        entry_id: T,
        annotation: &NewAnnotation,
    ) -> ClientResult<Annotation> {
        self.smart_json_q(
            Method::POST,
            EndPoint::Annotation(entry_id.into()),
            UNIT,
            annotation,
        )
    }

    /// Delete an annotation by id
    pub fn delete_annotation<T: Into<ID>>(&mut self, id: T) -> ClientResult<Annotation> {
        self.smart_json_q(Method::DELETE, EndPoint::Annotation(id.into()), UNIT, UNIT)
    }

    /// Get all annotations for an entry (by id).
    pub fn get_annotations<T: Into<ID>>(&mut self, id: T) -> ClientResult<Annotations> {
        let json: AnnotationRows =
            self.smart_json_q(Method::GET, EndPoint::Annotation(id.into()), UNIT, UNIT)?;
        Ok(json.rows)
    }

    /// Get all entries.
    pub fn get_entries(&mut self) -> ClientResult<Entries> {
        self._get_entries(&EntriesFilter::default())
    }

    /// Get all entries, filtered by filter parameters.
    pub fn get_entries_with_filter(&mut self, filter: &EntriesFilter) -> ClientResult<Entries> {
        self._get_entries(filter)
    }

    /// Does the actual work of retrieving the entries. Handles pagination.
    fn _get_entries(&mut self, filter: &EntriesFilter) -> ClientResult<Entries> {
        let mut entries = Entries::new();

        // TODO: should change the number per page?

        // we want to take control so that we can manage the hidden fields and
        // handle pagination
        let mut filter = filter.clone();
        filter.page = 1; // just to make sure

        // loop to handle pagination. No other api endpoints paginate so it's
        // fine here.
        loop {
            debug!("retrieving PaginatedEntries page {}", filter.page);
            let json: PaginatedEntries =
                self.smart_json_q(Method::GET, EndPoint::Entries, &filter, UNIT)?;

            entries.extend(json.embedded.items.into_iter());

            if json.page < json.pages {
                filter.page = json.page + 1;
            } else {
                break;
            }
        }

        Ok(entries)
    }

    /// Get an export of an entry in a particular format.
    pub fn export_entry<T: Into<ID>>(&mut self, entry_id: T, fmt: Format) -> ClientResult<String> {
        self.smart_text_q(
            Method::GET,
            EndPoint::Export(entry_id.into(), fmt),
            UNIT,
            UNIT,
        )
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
    pub fn add_tags_to_entry<T: Into<ID>, U: Into<String>>(
        &mut self,
        entry_id: T,
        tags: Vec<U>,
    ) -> ClientResult<Entry> {
        let mut data = HashMap::new();
        data.insert(
            "tags",
            tags.into_iter().map(|x| x.into()).collect::<Vec<String>>(),
        );

        self.smart_json_q(
            Method::POST,
            EndPoint::EntryTags(entry_id.into()),
            UNIT,
            &data,
        )
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
    /// exists on the server but isn't accessible to the user.
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
    ///
    /// Note: this allows deleting a tag with a comma by label.
    pub fn delete_tag_by_label<T: Into<String>>(&mut self, label: T) -> ClientResult<DeletedTag> {
        let mut params = HashMap::new();
        params.insert("tag".to_owned(), label.into());

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
    /// This method requires that tag names not contain commas. If you need to
    /// delete a tag containing a comma, use `delete_tag_by_label` instead.
    ///
    /// Returns a list of tags that were deleted (sans IDs). Returns 404 not
    /// found _only_ if _all_ tags were not found.
    pub fn delete_tags_by_label(&mut self, tags: Vec<TagString>) -> ClientResult<Vec<DeletedTag>> {
        let mut params = HashMap::new();
        params.insert(
            "tags",
            tags.into_iter()
                .map(|x| x.into_string())
                .collect::<Vec<String>>()
                .join(","),
        );

        // note: api doesn't return tag ids and no way to obtain since deleted
        // by label
        self.smart_json_q(Method::DELETE, EndPoint::TagsLabel, &params, UNIT)
    }

    /// Get the API version. Probably not useful because if the version isn't v2
    /// then this library won't work anyway.
    pub fn get_api_version(&mut self) -> ClientResult<String> {
        self.smart_json_q(Method::GET, EndPoint::Version, UNIT, UNIT)
    }

    /// Get the currently logged in user information.
    pub fn get_user(&mut self) -> ClientResult<User> {
        self.smart_json_q(Method::GET, EndPoint::User, UNIT, UNIT)
    }

    /// Register a user and create a client.
    pub fn register_user(&mut self, info: &RegisterInfo) -> ClientResult<NewlyRegisteredInfo> {
        self.json_q(Method::PUT, EndPoint::User, UNIT, info, false)
    }
}
