//! Provides clients for JSON requests and streamed HTTP responses.
//!
//! The `reqwest` feature enables [`Client`].

use reqwest::IntoUrl;

pub use reqwest::Error;

#[derive(Clone, Default)]
/// Sends HTTP requests using reqwest.
///
/// Cloning this client shares its connection pool. JSON requests and streamed responses use the
/// same client configuration. This type is available with the `reqwest` feature.
///
/// # Examples
///
/// ```
/// use umls_http::reqwest;
/// let client: reqwest::Client = Default::default();
/// let shared = client.clone();
/// ```
pub struct Client(reqwest::Client);

impl Client {
    /// Wraps a configured reqwest client for JSON and streaming requests.
    ///
    /// The client reuses connections across requests and can be cloned for independent request
    /// builders that share a connection pool.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use umls_http::reqwest;
    /// let inner = ::reqwest::Client::builder()
    ///     .timeout(Duration::from_secs(30))
    ///     .build()?;
    /// let client = reqwest::Client::new(inner);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(client: reqwest::Client) -> Self {
        Self(client)
    }
}

impl Client {
    /// Returns a [`reqwest::RequestBuilder`] object for given url.
    pub fn get<T: IntoUrl>(&self, url: T) -> reqwest::RequestBuilder {
        self.0.get(url)
    }
}
