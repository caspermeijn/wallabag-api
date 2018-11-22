use std::fmt;

pub enum EndPoint {
    Token,
}

impl fmt::Display for EndPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token => "/oauth/v2/token",
            }
        )
    }
}

pub struct UrlBuilder {
    base_url: String,
}

impl UrlBuilder {
    pub fn new(base_url: String) -> Self {
        UrlBuilder { base_url }
    }

    pub fn build(&self, end_point: EndPoint) -> String {
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
