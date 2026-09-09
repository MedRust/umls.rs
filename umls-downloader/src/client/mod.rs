//! HTTP transport implementations and the marker for builders without a client.
//!
//! Enable the `reqwest` feature to use `reqwest::Client`.

#[cfg(feature = "reqwest")]
pub mod reqwest;

/// Marks a request builder that has no HTTP client attached.
///
/// Builders in this state can accept a client before executing requests.
///
/// # Examples
///
/// ```
/// use umls_downloader::client::WithoutClient;
/// let state = WithoutClient;
/// ```
pub struct WithoutClient;
