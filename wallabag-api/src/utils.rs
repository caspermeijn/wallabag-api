use std::fmt;

#[derive(Debug, Clone, Copy)]
pub(crate) enum EndPoint {
    Token,
    Entries,
    Exists,
    Version,
    User,
    Tags,
    TagsLabel,
    Tag(u32),
    Entry(u32),
    Annotation(u32),
}

impl fmt::Display for EndPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::EndPoint::*;
        write!(
            f,
            "{}",
            match self {
                Token => "/oauth/v2/token".to_owned(),
                Entries => "/api/entries.json".to_owned(),
                Exists => "/api/entries/exists.json".to_owned(),
                Version => "/api/version.json".to_owned(),
                User => "/api/user.json".to_owned(),
                Tags => "/api/tags.json".to_owned(),
                TagsLabel => "/api/tags/label.json".to_owned(),
                Tag(i) => format!("/api/tags/{}.json", i),
                Entry(i) => format!("/api/entries/{}.json", i),
                Annotation(i) => format!("/api/annotations/{}.json", i),
            }
        )
    }
}

pub(crate) struct UrlBuilder {
    base_url: String,
}

impl UrlBuilder {
    pub(crate) fn new(base_url: String) -> Self {
        UrlBuilder { base_url }
    }

    pub(crate) fn build(&self, end_point: EndPoint) -> String {
        format!("{}{}", self.base_url, end_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_token_url() {
        let builder = UrlBuilder::new("https://example.com".to_owned());
        assert_eq!(
            "https://example.com/oauth/v2/token",
            builder.build(EndPoint::Token)
        );
    }
}
