use std::fmt;

/// Use to represent a format to export to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    XML,
    JSON,
    TXT,
    CSV,
    PDF,
    EPUB,
    MOBI,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Format::*;
        write!(
            f,
            "{}",
            match self {
                XML => "xml",
                JSON => "json",
                TXT => "txt",
                CSV => "csv",
                PDF => "pdf",
                EPUB => "epub",
                MOBI => "mobi",
            }
        )
    }
}

impl Format {
    /// Create a `Format` from a str. Case-insensitive. Returns `None` if could
    /// not determine format.
    ///
    /// ```
    /// # use wallabag_api::types::Format;
    /// assert_eq!(Format::try_from("json").unwrap(), Format::JSON);
    /// ```
    pub fn try_from(s: &str) -> Option<Self> {
        use self::Format::*;
        match s.to_lowercase().as_ref() {
            "xml" => Some(XML),
            "json" => Some(JSON),
            "txt" => Some(TXT),
            "csv" => Some(CSV),
            "pdf" => Some(PDF),
            "epub" => Some(EPUB),
            "mobi" => Some(MOBI),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_format_from_str() {
        assert_eq!(Format::try_from("csv"), Some(Format::CSV));
        assert_eq!(Format::try_from("Csv"), Some(Format::CSV));
        assert_eq!(Format::try_from("EPUB"), Some(Format::EPUB));
        assert_eq!(Format::try_from("EPUB_"), None);
        assert_eq!(Format::try_from("wat"), None);
        assert_eq!(Format::try_from(""), None);
    }
}
