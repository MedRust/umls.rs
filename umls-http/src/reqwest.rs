#![allow(missing_docs)]
//! Provides clients for JSON requests and streamed HTTP responses.
//!
//! The `reqwest` feature enables [`Client`].

use std::marker::PhantomData;

use http::{
    HeaderMap, HeaderName, StatusCode,
    header::{ACCEPT, CONTENT_TYPE},
};
use reqwest::IntoUrl;

pub use reqwest::Error;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const JSON_HEADER: &str = "application/json";

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
        let response = self.get_request_builder(url).send().await?;

        validate_json_response(&response)?;
        let response = validate_response_body(response).await?;

        response.json::<V>().await.map_err(ClientError::Client)
    }

    fn get_request_builder<T: IntoUrl>(&self, url: T) -> reqwest::RequestBuilder {
        self.client.get(url).header(ACCEPT, JSON_HEADER)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Client error: {0}")]
    Client(#[source] reqwest::Error),

    #[error("Opaque header value: {0}")]
    Opaque(#[source] http::header::ToStrError),

    #[error("Missing header: {0}")]
    MissingHeader(String),

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
        ClientError::Client(value.without_url())
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

/// Returns ascii header value.
///
/// # Errors
///
/// - [`ClientError::Opaque`]: The header value contains one or more non ascii characters.
/// - [`ClientError::MissingHeader`]: The missing header.
fn header_value(header: &HeaderMap, key: HeaderName) -> Result<&str, ClientError> {
    match header.get(&key) {
        Some(header_value) => header_value.to_str().map_err(ClientError::Opaque),
        None => Err(ClientError::MissingHeader(key.as_str().to_owned())),
    }
}

/// Validates JSON content type.
///
/// # Errors
///
/// - [`ClientError::Opaque`]: The header value contains one or more non ascii characters.
/// - [`ClientError::MissingHeader`]: The missing header.
/// - [`ClientError::UnsupportedContentTypeHeader`]: The content type header is not a "application/json"
fn validate_json_response(response: &reqwest::Response) -> Result<(), ClientError> {
    let headers = response.headers();
    let content_type = header_value(headers, CONTENT_TYPE)?;
    let media_type = content_type.split(";").next().unwrap_or_default().trim();
    if !media_type.eq_ignore_ascii_case(JSON_HEADER) {
        return Err(ClientError::UnsupportedContentTypeHeader);
    }
    Ok(())
}

/// Validates response using status code.
///
/// The function return [`reqwest::Response`] object when status code is `200(OK)` else return [`ClientError`].
///
/// # Errors
///
/// - [`ClientError::UnsupportedStatusCode`]: Unhandled status code.
/// - [`ClientError::Validation`]: Bad request generation with message.
/// - [`ClientError::NotFound`]: Resource not found.
/// - [`ClientError::Unauthorized`]: Autthentication failed.
async fn validate_response_body(
    response: reqwest::Response,
) -> Result<reqwest::Response, ClientError> {
    match response.status() {
        StatusCode::OK => Ok(response),
        StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::NOT_FOUND => {
            Err(response.json::<ErrorResponse>().await?.into())
        }
        other => Err(ClientError::UnsupportedStatusCode(other)),
    }
}
