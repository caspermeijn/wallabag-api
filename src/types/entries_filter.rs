use serde::Serializer;
use serde::Serialize;
use std::result::Result;

/// Used in `EntriesFilter` for ordering results.
#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Used in `EntriesFilter` for sorting results.
#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Created,
    Updated,
}

/// Represents possible filters to apply to `get_entries_filtered`. To use the
/// default for a filter, set the value to `None`.
#[derive(Serialize, Debug, Clone)]
pub struct EntriesFilter {
    /// None = all entries; true/false filters by archived or not archived only
    pub archive: Option<bool>,

    /// None = all entries; true/false filters by starred or not starred only
    pub starred: Option<bool>,

    /// Criteria to sort by.
    pub sort: SortBy,

    /// Sort order.
    pub order: SortOrder,

    /// Return entries that match _all_ tags given. If vec empty, then no
    /// filtering is done. (currently not method to get only untagged entries)
    ///
    /// Warning: do not supply tags with a comma in the name.
    /// TODO: make tags with comma in name impossible (how?)
    #[serde(serialize_with = "vec_to_str")]
    pub tags: Vec<String>,

    /// timestamp (in seconds) since when you want entries updated. This would
    /// be useful when implementing a sync method. Default is 0 (ie entries from
    /// the beginning of epoch).
    pub since: i64,

    /// None = all entries; true/false = entries which do or do not have a public link
    pub public: Option<bool>,
}

/// Used to serialize the tags list as a comma separated string.
fn vec_to_str<S>(vec: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&vec.join(","))
}

/// Use this to get an instance of `EntriesFilter` ready to go. The defaults
/// here reflect the defaults that the server uses if the entries aren't
/// specified.
impl Default for EntriesFilter {
    fn default() -> Self {
        Self {
            archive: None,
            starred: None,
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: 0,
            public: None,
        }
    }
}

/// Internal entries filter wrapper for adding additional data to a request.
#[derive(Serialize, Debug)]
pub(crate) struct RequestEntriesFilter<'a> {
    /// page number; for pagination
    pub page: u32,

    /// Embedded actual entries filter. Flatten for serialization so we can seemlessly use it in
    /// requests.
    #[serde(flatten)]
    pub filter: &'a EntriesFilter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_filter_init_test() {
        let filter = EntriesFilter {
            archive: None,
            starred: Some(true),
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: 0,
            public: None,
        };
        assert_eq!(filter.since, 0);
    }
}
