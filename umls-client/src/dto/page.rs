use std::num::NonZeroU64;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
/// A decoded page returned by the UMLS concept-atoms endpoint.
pub struct Page<T> {
    #[serde(rename = "pageSize")]
    size: NonZeroU64,
    #[serde(rename = "pageNumber")]
    number: NonZeroU64,
    #[serde(rename = "pageCount")]
    count: NonZeroU64,
    #[serde(rename = "result")]
    pub(crate) content: T,
}

impl<T> Page<T> {
    /// Returns the maximum number of atoms requested or returned per page.
    pub fn size(&self) -> std::num::NonZero<u64> {
        self.size
    }

    /// Returns the one-based number of this page.
    pub fn number(&self) -> std::num::NonZero<u64> {
        self.number
    }

    /// Returns the page count reported by the UMLS API.
    pub fn count(&self) -> std::num::NonZero<u64> {
        self.count
    }

    /// Returns this page's decoded content.
    pub fn content(&self) -> &T {
        &self.content
    }

    /// A conversion of page content type without affecting the size, number, count.
    pub fn try_map<U, E>(self, f: impl FnOnce(T) -> Result<U, E>) -> Result<Page<U>, E> {
        Ok(Page {
            size: self.size,
            number: self.number,
            count: self.count,
            content: f(self.content)?,
        })
    }
}
