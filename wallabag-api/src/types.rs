use serde_derive::Deserialize;

pub type Entries = Vec<Entry>;

#[derive(Deserialize, Debug)]
pub struct Entry {
    annotations: Annotations,
    content: String,
    created_at: String,
    domain_name: String,
    headers: Option<String>, // TODO: probably not string
    http_status: String,
    id: u32,
    is_archived: u32,
    is_public: bool,
    is_starred: u32,
    language: Option<String>, // TODO: probable not string
    mimetype: String,
    origin_url: Option<String>,
    preview_picture: Option<String>,
    published_at: Option<String>,
    published_by: Option<String>,
    reading_time: u32,
    starred_at: Option<String>,
    tags: Vec<Tag>,
    title: String,
    uid: Option<String>,
    updated_at: String,
    url: String,
    user_email: String,
    user_id: u32,
    user_name: String,
}

pub type Annotations = Vec<Annotation>;

#[derive(Deserialize, Debug)]
pub struct Annotation {
    annotator_schema_version: String,
    created_at: String,
    id: u32,
    quote: String,
    ranges: Vec<Range>,
    text: String,
    updated_at: String,
    user: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Range {
    end: String,
    endOffset: String,
    start: String,
    startOffset: String,
}


#[derive(Deserialize, Debug)]
pub struct Tag {
    id: u32,
    label: String,
    slug: String,
}


