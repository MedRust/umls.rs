//! Provides clients for JSON requests and streamed HTTP responses.
//!
//! The `reqwest` feature enables [`Client`].

use bytes::Bytes;
use futures_core::Stream;
use futures_util::TryStreamExt;
use serde::Serialize;
use serde::de::DeserializeOwned;

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
/// use umls_http::reqwest::Client;
/// let client: Client = Default::default();
/// let shared = client.clone();
/// ```
pub struct Client(reqwest::Client);

#[cfg(feature = "reqwest")]
#[derive(Debug, thiserror::Error)]
/// Reports failures from the reqwest-backed client.
pub enum Error {
    /// Reports a JSON serialization or deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[source] serde_json::Error),

    /// Reports a transport, HTTP-status, response decoding, or body stream failure from reqwest.
    #[error("Reqwest error: {0}")]
    Reqwest(#[source] reqwest::Error),

    /// Reports failure while constructing an HTTP response.
    #[error("Http error: {0}")]
    Http(#[source] http::Error),
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Reqwest(value)
    }
}

#[cfg(feature = "reqwest")]
impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json(value)
    }
}

#[cfg(feature = "reqwest")]
impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Error::Http(value)
    }
}

#[cfg(feature = "reqwest")]
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
    /// use umls_http::reqwest::Client;
    /// let inner = reqwest::Client::builder()
    ///     .timeout(Duration::from_secs(30))
    ///     .build()?;
    /// let client = Client::new(inner);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(client: reqwest::Client) -> Self {
        Self(client)
    }

    async fn send<T: Serialize + Send>(
        &self,
        request: http::Request<T>,
    ) -> Result<reqwest::Response, Error> {
        let (parts, body) = request.into_parts();
        let body = serde_json::to_vec(&body).map_err(Error::Json)?;
        let mut builder = self.0.request(parts.method, parts.uri.to_string());
        for (name, value) in &parts.headers {
            builder = builder.header(name, value);
        }
        if !body.is_empty() && body != b"null" {
            builder = builder.body(body);
        }
        let response = builder.send().await.map_err(Error::Reqwest)?;
        response.error_for_status().map_err(Error::Reqwest)
    }
}

#[cfg(feature = "reqwest")]
impl Client {
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
    /// use umls_http::reqwest::Client;
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client: Client = Default::default();
    /// let request = http::Request::get("https://uts-ws.nlm.nih.gov/releases").body(())?;
    /// let response = client.execute::<(), serde_json::Value>(request).await?;
    /// let body = response.into_body();
    /// Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if request serialization, transport, HTTP 4xx or 5xx status validation,
    /// JSON decoding, or response construction fails. The returned error preserves the original
    /// error as its source where the underlying library provides one.
    pub async fn execute<T: Serialize + Send, U: DeserializeOwned>(
        &self,
        request: http::Request<T>,
    ) -> Result<http::Response<U>, Error> {
        let response = self.send(request).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.json::<U>().await?;
        let mut output = http::Response::builder().status(status);
        *output.headers_mut().expect("response builder has headers") = headers;
        output.body(body).map_err(Error::Http)
    }

    /// Sends a request and returns its response body as a byte stream.
    ///
    /// The response retains the HTTP status and headers. The returned stream owns the response
    /// body and reads chunks as it is polled. Read failures are yielded as [`Error::Reqwest`].
    /// Pin the stream before consuming it with [`next`](futures_util::StreamExt::next).
    /// The request body is serialized as JSON; a JSON `null` body is omitted. The caller supplies
    /// any required content-type header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_http::reqwest::Client;
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client: Client = Default::default();
    /// let request = http::Request::get("https://uts-ws.nlm.nih.gov/releases").body(())?;
    /// let response = client.execute_stream(request).await?;
    /// use futures_util::StreamExt;
    /// let mut stream = std::pin::pin!(response.into_body());
    /// while let Some(chunk) = stream.next().await {
    ///     println!("Received {} bytes", chunk?.len());
    /// }
    /// Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if request serialization, transport, HTTP 4xx or 5xx status validation,
    /// or response construction fails. Errors while reading body chunks are returned by the
    /// stream. The returned error preserves the original error as its source where the underlying
    /// library provides one.
    pub async fn execute_stream<T: Serialize + Send>(
        &self,
        request: http::Request<T>,
    ) -> Result<
        http::Response<impl Stream<Item = Result<Bytes, Error>> + Send + 'static + use<T>>,
        Error,
    > {
        let response = self.send(request).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes_stream().map_err(Error::Reqwest);
        let mut output = http::Response::builder().status(status);
        *output.headers_mut().expect("response builder has headers") = headers;
        output.body(body).map_err(Error::Http)
    }
}
