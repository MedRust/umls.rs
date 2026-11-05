//! Search the UMLS Metathesaurus and decode search results.
//!
//! The module exposes [`Search`] for executing a validated
//! [`crate::search::Request`] and decoding the wire response.
//! Enable the `reqwest` feature and attach an `umls_http::reqwest::Client`
//! to execute requests. The executor can be reused across searches.

#[cfg(feature = "reqwest")]
use std::num::NonZeroU64;

#[cfg(feature = "reqwest")]
use serde::Deserialize;

use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use crate::{
    config::Config,
    search::{Request, core::DisplayName},
};

#[cfg(feature = "reqwest")]
use umls_http::reqwest;

#[derive(Debug, thiserror::Error)]
/// An error produced while constructing, sending, or decoding a search.
pub enum Error<T> {
    /// The HTTP client failed to send the request, validate its status, or decode
    /// the response. The underlying client error is retained as the source.
    #[error("client error: {0}")]
    Client(#[source] T),

    /// Represents the request construction error.
    #[error("Request error: {0}")]
    Request(http::Error),

    /// Represents uri construction error.
    #[error("Uri construction Error")]
    Uri(#[source] http::uri::InvalidUri),
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for Error<reqwest::Error> {
    fn from(value: reqwest::Error) -> Self {
        Error::Client(value)
    }
}

impl<T> From<http::uri::InvalidUri> for Error<T> {
    fn from(value: http::uri::InvalidUri) -> Self {
        Error::Uri(value)
    }
}

/// A reusable raw-search executor with explicit HTTP client state.
///
/// Start with `Search::default()` and attach an `umls_http::reqwest::Client`
/// created with `Client::default()` or `Client::new(inner)` for a configured
/// underlying reqwest client.
/// Its `search()` method borrows the executor, configuration, and request.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "reqwest")]
/// use umls_http::reqwest;
/// use umls_client::{config::Config, search::{Request, raw::Search}};
/// # #[cfg(feature = "reqwest")]
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
/// let request = Request::builder("cough".parse()?).build();
/// let executor = Search::default().with_client(reqwest::Client::default());
/// let response = executor.search(&config, &request).await?;
/// # Ok(())
/// # }
/// ```
pub struct Search<C> {
    client: C,
}

impl<C> Search<C> {
    /// Returns a reference to the attached client or the unconfigured client marker.
    pub fn client(&self) -> &C {
        &self.client
    }
}

impl Default for Search<WithoutClient> {
    fn default() -> Self {
        Self {
            client: Default::default(),
        }
    }
}

impl Search<WithoutClient> {
    /// Attaches an owned HTTP client without sending a request.
    ///
    /// Execution requires the `reqwest` feature and an `umls_http::reqwest::Client`.
    /// Clone an existing client to share its connection pool across requests.
    pub fn with_client<C>(self, client: C) -> Search<C> {
        Search { client }
    }
}

#[cfg(feature = "reqwest")]
impl Search<reqwest::Client> {
    /// Executes a UMLS Metathesaurus search.
    ///
    /// The request is encoded as query parameters and sent using the HTTP client
    /// attached with [`Search::with_client`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Client`] when the request fails, the server returns an
    /// unsuccessful status, or the response body cannot be deserialized.
    pub async fn search(
        &self,
        config: &Config,
        request: &Request,
    ) -> Result<SearchResponse, Error<reqwest::Error>> {
        let url = config.search_url(request).url();

        let response = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<SearchResponse>()
            .await?;

        tracing::debug!(
            page_size = response.page_size,
            page_number = response.page_number,
            result_count = response.result.results.len(),
            total_count = ?response.result.rec_count,
            "received UMLS search response"
        );

        Ok(response)
    }
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The top-level response returned by the UMLS search endpoint.
pub struct SearchResponse {
    /// Number of results requested or returned for the page.
    pub page_size: NonZeroU64,
    /// One-based number of the returned page.
    pub page_number: NonZeroU64,
    /// Search metadata and matching records.
    pub result: SearchResults,
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Search metadata and records contained in a [`SearchResponse`].
pub struct SearchResults {
    /// UMLS response-class discriminator.
    #[serde(rename = "classType")]
    pub class_type: DisplayName,
    /// Records matching the search request.
    pub results: Vec<RawSearchDto>,
    /// Total number of matching records, when supplied by the endpoint.
    pub rec_count: Option<u64>,
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A raw record returned by the UMLS search endpoint.
pub struct RawSearchDto {
    /// Identifier returned according to the request's return identifier type.
    pub ui: DisplayName,
    /// Source vocabulary associated with the record, when supplied.
    pub root_source: Option<DisplayName>,
    /// UMLS API URI for retrieving the corresponding record.
    pub uri: url::Url,
    /// Human-readable name of the matching record.
    pub name: DisplayName,
    /// Semantic types assigned to the record, when supplied.
    pub semantic_types: Option<Vec<DisplayName>>,
}

#[cfg(feature = "reqwest")]
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn deserializes_zero_result_count() {
        let response: SearchResponse = serde_json::from_value(serde_json::json!({
            "pageSize": 25,
            "pageNumber": 1,
            "result": {
                "classType": "searchResults",
                "results": [],
                "recCount": 0
            }
        }))
        .unwrap();

        assert_eq!(response.result.rec_count, Some(0));
    }

    #[test]
    fn omitted_result_count_means_no_results() {
        let response: SearchResponse = serde_json::from_value(serde_json::json!({
            "pageSize": 25,
            "pageNumber": 1,
            "result": {
                "classType": "searchResults",
                "results": [],
            }
        }))
        .unwrap();

        assert_eq!(response.result.rec_count, None);
    }
}
