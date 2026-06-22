//! Pagination query parameters and the list-response envelope.

use serde::{Deserialize, Serialize};

/// The maximum number of items a list request returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Limit {
    /// Up to 10 items.
    #[serde(rename = "10")]
    Ten,
    /// Up to 50 items.
    #[serde(rename = "50")]
    Fifty,
    /// Up to 100 items.
    #[serde(rename = "100")]
    OneHundred,
    /// Up to 200 items.
    #[serde(rename = "200")]
    TwoHundred,
    /// Up to 500 items.
    #[serde(rename = "500")]
    FiveHundred,
    /// Up to 1000 items.
    #[serde(rename = "1000")]
    OneThousand,
}

/// The number of items a list request skips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Offset {
    /// Skip none.
    #[serde(rename = "0")]
    Zero,
    /// Skip 10.
    #[serde(rename = "10")]
    Ten,
    /// Skip 50.
    #[serde(rename = "50")]
    Fifty,
    /// Skip 100.
    #[serde(rename = "100")]
    OneHundred,
    /// Skip 200.
    #[serde(rename = "200")]
    TwoHundred,
    /// Skip 500.
    #[serde(rename = "500")]
    FiveHundred,
    /// Skip 1000.
    #[serde(rename = "1000")]
    OneThousand,
}

/// Common pagination query parameters accepted by list endpoints.
///
/// Build with [`PaginationParams::new`] and the chainable setters; only the
/// fields you set are sent on the wire.
///
/// ```
/// use blindpay::{Limit, PaginationParams};
/// let params = PaginationParams::new()
///     .limit(Limit::Fifty)
///     .starting_after("cus_123");
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct PaginationParams {
    /// Maximum number of items to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<Limit>,
    /// Number of items to skip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<Offset>,
    /// Cursor: return items after this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starting_after: Option<String>,
    /// Cursor: return items before this object ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending_before: Option<String>,
}

impl PaginationParams {
    /// Creates an empty set of pagination parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of items to return.
    pub fn limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the number of items to skip.
    pub fn offset(mut self, offset: Offset) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the `starting_after` cursor (an object ID).
    pub fn starting_after(mut self, cursor: impl Into<String>) -> Self {
        self.starting_after = Some(cursor.into());
        self
    }

    /// Sets the `ending_before` cursor (an object ID).
    pub fn ending_before(mut self, cursor: impl Into<String>) -> Self {
        self.ending_before = Some(cursor.into());
        self
    }
}

/// Pagination metadata included in a paginated list response.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Pagination {
    /// Whether more items are available after this page.
    pub has_more: bool,
    /// Object ID of the first item in the next page, if any.
    #[serde(default)]
    pub next_page: Option<String>,
    /// Object ID of the first item in the previous page, if any.
    #[serde(default)]
    pub prev_page: Option<String>,
}

/// A list response.
///
/// The API returns either a paginated envelope
/// (`{ "data": [...], "pagination": {...} }`) or a bare array (`[...]`, when no
/// pagination parameters were sent). Use [`data`](ListResponse::data) /
/// [`into_data`](ListResponse::into_data) to read the items uniformly and
/// [`pagination`](ListResponse::pagination) for the metadata when present.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ListResponse<T> {
    /// The paginated envelope form.
    Paginated {
        /// The items in this page.
        data: Vec<T>,
        /// Pagination metadata.
        pagination: Pagination,
    },
    /// The bare-array form (no pagination metadata).
    Bare(Vec<T>),
}

impl<T> ListResponse<T> {
    /// Returns the items as a slice, regardless of the response form.
    pub fn data(&self) -> &[T] {
        match self {
            ListResponse::Paginated { data, .. } => data,
            ListResponse::Bare(data) => data,
        }
    }

    /// Consumes the response, returning the owned items.
    pub fn into_data(self) -> Vec<T> {
        match self {
            ListResponse::Paginated { data, .. } => data,
            ListResponse::Bare(data) => data,
        }
    }

    /// Returns the pagination metadata, if the response was paginated.
    pub fn pagination(&self) -> Option<&Pagination> {
        match self {
            ListResponse::Paginated { pagination, .. } => Some(pagination),
            ListResponse::Bare(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_and_offset_serialize_to_strings() {
        assert_eq!(
            serde_json::to_string(&Limit::OneThousand).unwrap(),
            "\"1000\""
        );
        assert_eq!(serde_json::to_string(&Offset::Zero).unwrap(), "\"0\"");
    }

    #[test]
    fn pagination_params_skip_unset_fields() {
        let params = PaginationParams::new()
            .limit(Limit::Fifty)
            .starting_after("cus_123");
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "limit": "50", "starting_after": "cus_123" })
        );
    }

    #[test]
    fn list_response_handles_both_shapes() {
        let bare: ListResponse<String> = serde_json::from_str(r#"["a","b"]"#).unwrap();
        assert_eq!(bare.data(), ["a".to_string(), "b".to_string()]);
        assert!(bare.pagination().is_none());

        let paged: ListResponse<String> = serde_json::from_str(
            r#"{"data":["a"],"pagination":{"has_more":true,"next_page":"x","prev_page":null}}"#,
        )
        .unwrap();
        assert_eq!(paged.data(), ["a".to_string()]);
        assert!(paged.pagination().unwrap().has_more);
        assert_eq!(paged.pagination().unwrap().next_page.as_deref(), Some("x"));
        assert!(paged.pagination().unwrap().prev_page.is_none());
    }
}
