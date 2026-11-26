use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use crate::dto::{content::Content, page::Page};

/// Represents search results.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults<T> {
    #[serde(rename = "results")]
    content: Content<T>,
    #[serde(rename = "recCount")]
    total_count: Option<NonZeroU64>,
}

impl<T> SearchResults<T> {
    /// Creates new [`SearchResults`] object.
    pub fn new(content: Content<T>, total_count: Option<NonZeroU64>) -> Self {
        Self {
            content,
            total_count,
        }
    }

    /// Returns content.
    pub fn content(&self) -> &Content<T> {
        &self.content
    }

    /// Returns total count if search present else return [`None`].
    pub fn total_count(&self) -> Option<std::num::NonZero<u64>> {
        self.total_count
    }

    /// Consumes self and return content.
    pub fn into_content(self) -> Content<T> {
        self.content
    }
}

impl<T> Page<SearchResults<T>> {
    /// Returns page count if total count is present else [`None`].
    pub fn count(&self) -> Option<NonZeroU64> {
        let records = self.content().total_count()?;
        let count = records.div_ceil(self.size());
        Some(count)
    }
}
