use std::num::NonZeroU64;

use serde::Deserialize;

use crate::dto::content::Content;

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
    content: T,
}

impl<T> Page<T> {
    /// Create new [`Page`] object.
    pub fn new(size: NonZeroU64, number: NonZeroU64, count: NonZeroU64, content: T) -> Self {
        Self {
            size,
            number,
            count,
            content,
        }
    }

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

    /// Returns content object and consumes self.
    pub fn into_content(self) -> T {
        self.content
    }
}

impl<T> Page<Content<T>> {
    /// Try iterative map on each element.
    pub fn try_iter_map<U, E>(
        self,
        f: impl FnMut(T) -> Result<U, E>,
    ) -> Result<Page<Content<U>>, E> {
        let result = self
            .content
            .into_list()
            .into_iter()
            .map(f)
            .collect::<Result<Vec<U>, E>>()?;

        Ok(Page {
            size: self.size,
            number: self.number,
            count: self.count,
            content: Content::new(result),
        })
    }
}
