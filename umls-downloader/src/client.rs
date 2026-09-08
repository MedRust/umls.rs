//! Provides clients for JSON requests and streamed HTTP responses.
//!
//! The `reqwest` feature enables [`ReqwestClient`].

use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt};
#[cfg(feature = "reqwest")]
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{BoxError, ByteStream};

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

#[cfg(feature = "reqwest")]
#[derive(Clone, Default)]
/// Sends HTTP requests using reqwest.
///
/// Cloning this client shares its connection pool. JSON requests and streamed responses use the
/// same client configuration. This type is available with the `reqwest` feature.
///
/// # Examples
///
/// ```
/// use umls_downloader::client::ReqwestClient;
/// let client: ReqwestClient = Default::default();
/// let shared = client.clone();
/// ```
pub struct ReqwestClient(reqwest::Client);

#[cfg(feature = "reqwest")]
impl ReqwestClient {
    /// Creates an HTTP client with reqwest's default configuration.
    ///
    /// The client reuses connections across requests and can be cloned for independent request
    /// builders that share a connection pool.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::client::ReqwestClient;
    /// use reqwest::Client;
    /// let reqwest_client = Client::new();
    /// let client = ReqwestClient::new(reqwest_client);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the TLS backend cannot be initialized or the system resolver configuration
    /// cannot be loaded.
    pub fn new(client: reqwest::Client) -> Self {
        Self(client)
    }

    async fn send<T: Serialize + Send>(
        &self,
        request: http::Request<T>,
    ) -> Result<reqwest::Response, http::Error> {
        let (parts, body) = request.into_parts();
        let body = serde_json::to_vec(&body).map_err(as_http_error)?;
        let mut builder = self.0.request(parts.method, parts.uri.to_string());
        for (name, value) in &parts.headers {
            builder = builder.header(name, value);
        }
        if !body.is_empty() && body != b"null" {
            builder = builder.body(body);
        }
        let response = builder.send().await.map_err(as_http_error)?;
        response.error_for_status().map_err(as_http_error)
    }
}

fn as_http_error(error: impl std::fmt::Display) -> http::Error {
    // `HttpClient` currently exposes `http::Error`, so create one while retaining
    // the transport message in the invalid URI value.
    let invalid = format!("{}\n", error);
    http::Request::builder().uri(invalid).body(()).unwrap_err()
}

#[cfg(feature = "reqwest")]
impl ReqwestClient {
    /// Sends a request and deserializes its JSON response.
    ///
    /// The response retains the HTTP status and headers. The entire response body is decoded as
    /// `U`.
    /// The request body is serialized as JSON; a JSON `null` body is omitted. The caller supplies
    /// any required content-type header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::client::ReqwestClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client: ReqwestClient = Default::default();
    /// let request = http::Request::get("https://uts-ws.nlm.nih.gov/releases").body(())?;
    /// let response = client.execute::<(), serde_json::Value>(request).await?;
    /// let body = response.into_body();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if request serialization, transport, HTTP 4xx or 5xx status validation,
    /// JSON decoding, or
    /// response construction fails.
    /// Request-stage failures are reported as [`http::Error`] without preserving their original
    /// error type or source chain.
    pub async fn execute<T: Serialize + Send, U: DeserializeOwned>(
        &self,
        request: http::Request<T>,
    ) -> Result<http::Response<U>, http::Error> {
        let response = self.send(request).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.json::<U>().await.map_err(as_http_error)?;
        let mut output = http::Response::builder().status(status);
        *output.headers_mut().expect("response builder has headers") = headers;
        output.body(body)
    }

    /// Sends a request and returns its response body as a byte stream.
    ///
    /// The response retains the HTTP status and headers. Body chunks are read as the returned
    /// stream
    /// is polled; subsequent read failures are yielded as stream errors.
    /// The request body is serialized as JSON; a JSON `null` body is omitted. The caller supplies
    /// any required content-type header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::client::ReqwestClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client: ReqwestClient = Default::default();
    /// let request = http::Request::get("https://uts-ws.nlm.nih.gov/releases").body(())?;
    /// let response = client.execute_stream(request).await?;
    /// let stream = response.into_body();
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if request serialization, transport, HTTP 4xx or 5xx status validation, or
    /// response
    /// construction fails. Errors while reading body chunks are returned by the stream.
    /// Request-stage failures are reported as [`http::Error`] without preserving their original
    /// error type or source chain.
    pub async fn execute_stream<T: Serialize + Send>(
        &self,
        request: http::Request<T>,
    ) -> Result<http::Response<ByteStream>, http::Error> {
        let response = self.send(request).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let stream = response
            .bytes_stream()
            .map_err(|error| Box::new(error) as BoxError);
        let body: ByteStream = Box::pin(stream.map(|chunk: Result<Bytes, BoxError>| chunk));
        let mut output = http::Response::builder().status(status);
        *output.headers_mut().expect("response builder has headers") = headers;
        output.body(body)
    }
}
