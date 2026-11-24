#![allow(missing_docs)]
//! Provides clients for JSON requests and streamed HTTP responses.
//!
//! The `reqwest` feature enables [`Client`].

use std::marker::PhantomData;

use http::{
    StatusCode,
    header::{ACCEPT, CONTENT_TYPE},
};
use reqwest::IntoUrl;

pub use reqwest::Error;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub type Client = ReqwestClient<Any>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Json;

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
pub struct ReqwestClient<T> {
    client: reqwest::Client,
    _type: PhantomData<T>,
}

impl ReqwestClient<Any> {
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
        Self {
            client,
            _type: PhantomData,
        }
    }

    pub fn with_type<T>(self) -> ReqwestClient<T> {
        ReqwestClient {
            client: self.client,
            _type: PhantomData,
        }
    }
}

impl ReqwestClient<Any> {
    /// Returns a [`reqwest::RequestBuilder`] object for given url.
    pub fn get<T: IntoUrl>(&self, url: T) -> reqwest::RequestBuilder {
        self.client.get(url)
    }
}

impl ReqwestClient<Json> {
    /// Returns a [`bytes::Bytes`] object for given url.
    pub async fn get<T: IntoUrl, V: DeserializeOwned>(&self, url: T) -> Result<V, ClientError> {
        let response = self
            .client
            .get(url)
            .header(ACCEPT, "application/json")
            .send()
            .await?;

        let headers = response.headers();
        match headers.get(CONTENT_TYPE) {
            Some(x) => {
                let str = x.to_str().map_err(ClientError::Opaque)?;
                if !["application/json", "application/json; charset=utf-8"].contains(&str) {
                    return Err(ClientError::UnsupportedContentTypeHeader);
                }
            }
            None => {
                return Err(ClientError::MissingContentTypeHeader);
            }
        }

        match response.status() {
            StatusCode::OK => response.json::<V>().await.map_err(ClientError::Client),
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::NOT_FOUND => {
                Err(response.json::<ErrorResponse>().await?.into())
            }
            other => Err(ClientError::UnsupportedStatusCode(other)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Client error: {0}")]
    Client(#[source] reqwest::Error),

    #[error("Opaque header value: {0}")]
    Opaque(#[source] http::header::ToStrError),

    #[error("Missing content type header")]
    MissingContentTypeHeader,

    #[error("Unsupported content type header")]
    UnsupportedContentTypeHeader,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Unsupported error name: {0}")]
    UnsupportedErrorName(ErrorName),

    #[error("Unsupported status code: {0}")]
    UnsupportedStatusCode(StatusCode),

    #[error("NotFound error: {0}")]
    NotFound(String),
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        ClientError::Client(value)
    }
}

impl From<ErrorResponse> for ClientError {
    fn from(value: ErrorResponse) -> Self {
        match value.name {
            ErrorName::Validation => ClientError::Validation(value.message),
            ErrorName::Unauthorized => ClientError::Unauthorized(value.message),
            ErrorName::NotFound => ClientError::NotFound(value.message),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ErrorName {
    #[serde(rename = "UnauthorizedError")]
    Unauthorized,
    #[serde(rename = "ValidationError")]
    Validation,
    #[serde(rename = "NotFoundError")]
    NotFound,
}

impl ErrorName {
    pub fn as_str(&self) -> &str {
        match self {
            ErrorName::Unauthorized => "UnauthorizedError",
            ErrorName::Validation => "ValidationError",
            ErrorName::NotFound => "NotFoundError",
        }
    }
}

impl std::fmt::Display for ErrorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub name: ErrorName,
    pub status: u16,
    pub message: String,
}
