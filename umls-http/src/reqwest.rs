#![allow(missing_docs)]
//! Provides clients for JSON requests and streamed HTTP responses.
//!
//! The `reqwest` feature enables [`Client`].

use std::marker::PhantomData;

use http::{
    HeaderMap, HeaderName, StatusCode,
    header::{ACCEPT, CONTENT_TYPE},
};
use mime::Mime;
use reqwest::IntoUrl;

pub use reqwest::Error;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use umls_util::{Content, Page, SearchResults};

const JSON_HEADER: &str = "application/json";

/// An HTTP client used to query server.
pub type Client = ReqwestClient<Any>;

/// Typestate marker preserving backward-compatible, unvalidated request builders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Any;

/// Typestate marker that enables automatic JSON response validation and deserialization.
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

    /// Consumes `self` and returns a new [`ReqwestClient`] configured with the specified typestate marker.
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
    /// Constructs and sends a GET request to the target URL, validating and deserializing the JSON response.
    ///
    /// Sends a GET request with an `Accept: application/json` header, verifies that the response
    /// media type is JSON, and on success deserializes the response body into `V`.
    ///
    /// # Errors
    ///
    /// - [`ClientError::Opaque`]: A response header value contains one or more non-ASCII characters.
    /// - [`ClientError::MissingHeader`]: A required header (e.g. `Content-Type`) is missing from the response.
    /// - [`ClientError::UnsupportedContentTypeHeader`]: The response `Content-Type` is not `application/json`.
    /// - [`ClientError::UnsupportedStatusCode`]: An unhandled HTTP status code was received.
    /// - [`ClientError::Validation`]: The request failed upstream validation (HTTP 400).
    /// - [`ClientError::NotFound`]: The requested resource was not found (HTTP 404).
    /// - [`ClientError::Unauthorized`]: Authentication failed (HTTP 401).
    /// - [`ClientError::Client`]: Underlying HTTP transport error or JSON deserialization failure.
    pub async fn get<T: IntoUrl, V: DeserializeOwned>(&self, url: T) -> Result<V, ClientError> {
        let response = self.get_request_builder(url).send().await?;

        validate_json_response(&response)?;
        let response = validate_response_body(response).await?;

        response.json::<V>().await.map_err(ClientError::Client)
    }

    fn get_request_builder<T: IntoUrl>(&self, url: T) -> reqwest::RequestBuilder {
        self.client.get(url).header(ACCEPT, JSON_HEADER)
    }

    pub async fn fetch_page<T: IntoUrl, V: DeserializeOwned>(
        &self,
        url: T,
    ) -> Result<Page<V>, ClientError> {
        self.get::<_, Page<V>>(url).await
    }

    pub async fn fetch_page_content<T: IntoUrl, V: DeserializeOwned>(
        &self,
        url: T,
    ) -> Result<Page<Content<V>>, ClientError> {
        self.fetch_page::<_, Content<V>>(url).await
    }

    pub async fn search<T: IntoUrl, V: DeserializeOwned>(
        &self,
        url: T,
    ) -> Result<Page<SearchResults<V>>, ClientError> {
        self.fetch_page::<_, SearchResults<V>>(url).await
    }
}

/// Represents an error produced by [`ReqwestClient`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ClientError {
    /// Represents an underlying HTTP transport or JSON deserialization error.
    #[error("Client error: {0}")]
    Client(#[source] reqwest::Error),

    /// Represents a header value containing non-ASCII characters.
    #[error("Opaque header value: {0}")]
    Opaque(#[source] http::header::ToStrError),

    /// Represents a missing response header.
    #[error("Missing header: {0}")]
    MissingHeader(String),

    /// Represents unsupported `Content-Type` header.
    #[error("Unsupported content type header")]
    UnsupportedContentTypeHeader,

    /// Represents an upstream API validation error.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Represents an authentication failure.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Represents an unexpected or unhandled HTTP response status code.
    #[error("Unsupported status code: {0}")]
    UnsupportedStatusCode(u16),

    /// Represents a resource not found error.
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
enum ErrorName {
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
struct ErrorResponse {
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

fn is_json_content_type(header: &str) -> bool {
    let Ok(media_type) = header.parse::<Mime>() else {
        return false;
    };

    media_type.type_() == mime::APPLICATION && media_type.subtype() == mime::JSON
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
    if !is_json_content_type(content_type) {
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
        other => Err(ClientError::UnsupportedStatusCode(other.as_u16())),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn response(status_code: u16, body: Option<reqwest::Body>) -> reqwest::Response {
        let mut response = http::Response::builder().status(status_code);

        if body.is_some() {
            response = response.header(CONTENT_TYPE, "appllcation/json");
        }

        response
            .body(body.unwrap_or_default())
            .expect("Should build a response with 200 status code")
            .into()
    }

    #[tokio::test]
    async fn validate_response_body_accepts_200_ok() {
        let response = response(200, None);

        validate_response_body(response)
            .await
            .expect("For 200 status code should return the response object");
    }

    #[tokio::test]
    async fn validate_response_body_rejects_unsupported_status_code() {
        for status_code in 100..=599 {
            // Skip supported status code.
            if [200, 400, 401, 404].contains(&status_code) {
                continue;
            }

            let response = response(status_code, None);

            match validate_response_body(response).await {
                Err(ClientError::UnsupportedStatusCode(code)) => assert_eq!(code, status_code),
                other => panic!("Expected unsupported status code, but got: {:?}", other),
            }
        }
    }

    fn body(value: &serde_json::Value) -> reqwest::Body {
        let json_string = serde_json::to_string(&value)
            .expect("Valid JSON value object should be serialize to vector");
        reqwest::Body::wrap(json_string)
    }

    #[tokio::test]
    async fn validate_response_body_rejects_bad_request() {
        let value = serde_json::json!({
            "name": "ValidationError",
            "status": 400,
            "message": "Missing 'string'"
        });
        let body = body(&value);
        let response = response(400, Some(body));

        match validate_response_body(response).await {
            Err(ClientError::Validation(message)) => {
                assert_eq!(message, "Missing 'string'");
            }
            other => panic!("Expected validation error, but got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn validate_response_body_rejects_unauthorized_error() {
        let value = serde_json::json!({
            "name": "UnauthorizedError",
            "status": 401,
            "message": "401 - Invalid API Key - Documentation: https://documentation.uts.nlm.nih.gov/rest/authentication.html"
        });
        let body = body(&value);
        let response = response(401, Some(body));

        match validate_response_body(response).await {
            Err(ClientError::Unauthorized(message)) => {
                assert_eq!(
                    message,
                    "401 - Invalid API Key - Documentation: https://documentation.uts.nlm.nih.gov/rest/authentication.html"
                );
            }
            other => panic!("Expected Unauthorized error, but got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn validate_response_body_rejects_not_found_error() {
        let value = serde_json::json!({
            "name": "NotFoundError",
            "status": 404,
            "message": "404 Resource Not Found"
        });
        let body = body(&value);
        let response = response(404, Some(body));

        match validate_response_body(response).await {
            Err(ClientError::NotFound(message)) => {
                assert_eq!(message, "404 Resource Not Found");
            }
            other => panic!("Expected not found error, but got: {:?}", other),
        }
    }

    #[test]
    fn validate_json_response_accepts_on_json_content_type() {
        let headers = ["application/json", "application/json; charset=utf-8"];

        for header in headers {
            let response: reqwest::Response = http::Response::builder()
                .status(200)
                .header("CONTENT-TYPE", header)
                .body(reqwest::Body::default())
                .expect("Valid object should build the response object")
                .into();

            validate_json_response(&response)
                .expect("The valid json content type response should validate successfully.");
        }
    }

    #[test]
    fn returns_opaque_error_on_non_ascii_header() {
        let response: reqwest::Response = http::Response::builder()
            .header("CONTENT-TYPE", "9f86d081_äöüß_ñçøå_15a3bf4f")
            .body(reqwest::Body::default())
            .expect("Valid object should build the response object.")
            .into();

        match validate_json_response(&response) {
            Err(ClientError::Opaque(_)) => {}
            other => panic!(
                "Expected opaque response header value error, but got: {:?}",
                other
            ),
        }
    }

    #[test]
    fn missing_header_on_content_type_not_set() {
        let response: reqwest::Response = http::Response::builder()
            .body(reqwest::Body::default())
            .expect("Valid object should build the response.")
            .into();

        match validate_json_response(&response) {
            Err(ClientError::MissingHeader(header)) => {
                assert_eq!(header, CONTENT_TYPE.as_str());
            }
            other => panic!("Expected missing header, but got: {:?}", other),
        }
    }

    #[test]
    fn unsupported_content_type_on_non_json_type() {
        let response: reqwest::Response = http::Response::builder()
            .header("CONTENT-TYPE", "text/plain")
            .body(reqwest::Body::default())
            .expect("Valid object should build the response.")
            .into();

        let result = validate_json_response(&response);
        assert!(matches!(
            result,
            Err(ClientError::UnsupportedContentTypeHeader)
        ));
    }

    #[test]
    fn error_name_to_string() {
        let error_names = [
            (ErrorName::Unauthorized, "UnauthorizedError"),
            (ErrorName::Validation, "ValidationError"),
            (ErrorName::NotFound, "NotFoundError"),
        ];

        for (error_name, str) in error_names {
            assert_eq!(error_name.as_str(), str);
        }
    }

    #[test]
    fn error_name_display_string() {
        let error_names = [
            (ErrorName::Unauthorized, "UnauthorizedError"),
            (ErrorName::Validation, "ValidationError"),
            (ErrorName::NotFound, "NotFoundError"),
        ];

        for (error_name, str) in error_names {
            assert_eq!(error_name.to_string(), str);
        }
    }

    #[test]
    fn failed_on_invalid_header_value() {
        assert!(is_json_content_type("application/json"));
        assert!(is_json_content_type("application/json; charset=utf-8"));
        assert!(!is_json_content_type(
            "446d3396-2bb2-4d68-aa0c-182cecd14a13"
        ));
    }

    #[tokio::test]
    async fn reqwest_error_to_client_error() {
        let reqwest_error = reqwest::get("://invalid_url").await.unwrap_err();
        let error: ClientError = reqwest_error.into();
        assert!(matches!(error, ClientError::Client(_)));
    }
}
