//! HTTP transport implementations and the marker for builders without a client.
//!
//! Enable the optional `reqwest` feature to use `umls_http::reqwest::Client`
//! for JSON requests and streamed byte responses. Without it, only
//! [`WithoutClient`] is available. Cloned clients share their connection pool.
//!
//! Add `umls-http` as a direct dependency when using its types with
//! `umls-downloader`; the downloader’s `reqwest` feature enables this transport.

#[cfg(feature = "reqwest")]
pub mod reqwest;

/// Marks a request builder that has no HTTP client attached.
///
/// Builders in this state can accept a client before executing requests.
///
/// # Examples
///
/// ```
/// use umls_http::WithoutClient;
/// let state = WithoutClient;
/// ```
pub struct WithoutClient;
