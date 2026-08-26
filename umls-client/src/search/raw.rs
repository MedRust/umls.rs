//! Search the UMLS Metathesaurus and decode search results.
//!
//! The module exposes [`crate::search::raw::search`] for executing a validated
//! [`crate::search::Request`] and decoding the wire response.

use std::num::NonZeroU64;

use secrecy::ExposeSecret;

use serde::Deserialize;
use url::Url;

use crate::{
    config::Config,
    search::{Request, core::DisplayName},
};

#[derive(Debug, thiserror::Error)]
/// An error produced while constructing, sending, or decoding a search.
pub enum Error {
    /// The search endpoint or its query parameters could not be represented
    /// as a valid URL.
    #[error("URL parser error: {0}")]
    Url(url::ParseError),

    /// The HTTP request failed, returned an unsuccessful status, or contained
    /// a response body that could not be decoded.
    #[error("Reqwest error: {0}")]
    Reqwest(reqwest::Error),
}

/// Executes a UMLS Metathesaurus search.
///
/// The request is encoded as query parameters and sent using the HTTP client
/// held by `config`.
///
/// # Errors
///
/// Returns [`Error::Url`] when the endpoint URL cannot be constructed.
/// Returns [`Error::Reqwest`] when the request fails, the server returns an
/// unsuccessful status, or the response body cannot be deserialized.
pub async fn search(config: &Config, request: &Request) -> Result<SearchResponse, Error> {
    let path = format!("search/{}", config.version());
    let url_string = config
        .base_url()
        .join(&path)
        .map_err(Error::Url)?
        .to_string();

    let source_vocabularies_params = request
        .source_vocabularies()
        .iter()
        .map(|x| x.as_ref())
        .collect::<Vec<&str>>()
        .join(",");

    let semantic_types_params = request
        .semantic_types()
        .iter()
        .map(|x| x.as_ref())
        .collect::<Vec<&str>>()
        .join(",");

    let semantic_groups_params = request
        .semantic_groups()
        .iter()
        .map(|x| x.as_ref())
        .collect::<Vec<&str>>()
        .join(",");

    let url = Url::parse_with_params(
        &url_string,
        &[
            ("apiKey", config.api_key().expose_secret()),
            ("string", request.term().as_ref()),
            ("inputType", request.input_type().as_ref()),
            (
                "includeObsolete",
                &request.obsolete().is_included().to_string(),
            ),
            (
                "includeSuppressible",
                &request.suppressible().is_included().to_string(),
            ),
            ("returnIdType", request.return_id_type.as_ref()),
            ("sabs", &source_vocabularies_params),
            ("searchType", request.search_type().as_ref()),
            ("semanticTypes", &semantic_types_params),
            ("semanticGroups", &semantic_groups_params),
            (
                "partialSearch",
                &request.term_match().allows_partial().to_string(),
            ),
            ("pageSize", &request.page().size().to_string()),
            ("pageNumber", &request.page().number().to_string()),
        ],
    )
    .map_err(Error::Url)?;

    tracing::debug!(
        input_type = request.input_type().as_ref(),
        return_id_type = request.return_id_type.as_ref(),
        search_type = request.search_type().as_ref(),
        include_obsolete = request.obsolete().is_included(),
        include_suppressible = request.suppressible().is_included(),
        allows_partial_match = request.term_match().allows_partial(),
        source_vocabulary_count = request.source_vocabularies().len(),
        semantic_type_count = request.semantic_types().len(),
        semantic_group_count = request.semantic_groups().len(),
        page_size = request.page().size(),
        page_number = request.page().number(),
        "sending UMLS search request"
    );

    let response = config
        .client()
        .get(url)
        .send()
        .await
        .map_err(Error::Reqwest)?
        .error_for_status()
        .map_err(Error::Reqwest)?;

    let response: SearchResponse = response.json().await.map_err(Error::Reqwest)?;

    tracing::debug!(
        page_size = response.page_size,
        page_number = response.page_number,
        result_count = response.result.results.len(),
        total_count = ?response.result.rec_count,
        "received UMLS search response"
    );

    Ok(response)
}

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A raw record returned by the UMLS search endpoint.
pub struct RawSearchDto {
    /// Identifier returned according to the request's return identifier type.
    pub ui: DisplayName,
    /// Source vocabulary associated with the record, when supplied.
    pub root_source: Option<DisplayName>,
    /// UMLS API URI for retrieving the corresponding record.
    pub uri: Url,
    /// Human-readable name of the matching record.
    pub name: DisplayName,
    /// Semantic types assigned to the record, when supplied.
    pub semantic_types: Option<Vec<DisplayName>>,
}

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
