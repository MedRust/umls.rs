//! Asynchronous helpers for discovering and downloading releases from the
//! [UMLS Terminology Services (UTS)][uts] Release and Download APIs.
//!
//! The crate is HTTP-client agnostic. Implement [`HttpClient`] to query release
//! metadata and [`StreamHttpClient`] to stream release archives to any
//! caller-selected destination.
//!
//! [uts]: https://documentation.uts.nlm.nih.gov/automating-downloads.html
#![warn(missing_docs)]

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    pin::Pin,
};

use async_trait::async_trait;
use bytes::Bytes;
use chrono::NaiveDate;
use futures_core::Stream;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::kind::Current;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A release category advertised by the UTS Release API.
pub struct ReleaseType {
    /// The product to which this release category belongs, such as UMLS or RxNorm.
    pub product: String,
    /// The value accepted by the Release API's `releaseType` query parameter.
    #[serde(rename = "releaseType")]
    pub name: String,
    /// The Release API endpoint for releases in this category.
    pub endpoint: String,
}

impl std::fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.product, self.name)
    }
}

impl ReleaseType {
    /// Returns the product name.
    pub fn product(&self) -> &str {
        &self.product
    }

    /// Returns the Release API's release-type identifier.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the endpoint used to fetch releases in this category.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReleaseTypes {
    #[serde(rename = "releaseTypes")]
    types: Vec<ReleaseType>,
}

/// Fetches the release categories advertised by the UTS Release API.
///
/// The supplied client is responsible for sending the request and
/// deserializing its JSON response.
pub async fn get_release_types<C: HttpClient>(
    http_client: &C,
) -> Result<Vec<ReleaseType>, http::Error> {
    let request = http::Request::builder()
        .uri("https://uts-ws.nlm.nih.gov/releases")
        .method(http::Method::GET)
        .body(())?;

    http_client
        .execute::<(), ReleaseTypes>(request)
        .await
        .map(|x| x.into_body())
        .map(|x| x.types)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Metadata for a downloadable UTS release file.
pub struct Release {
    /// The name of the release archive.
    pub file_name: String,
    /// The product-specific release version.
    pub release_version: String,
    /// The release date.
    pub release_date: NaiveDate,
    /// The source URL to pass to the UTS Download API.
    pub download_url: String,
    /// The release-type identifier associated with this file.
    pub release_type: String,
    /// The product to which the release belongs.
    pub product: String,

    /// Whether this file belongs to the current release.
    #[serde(default)]
    pub current: bool,
}

impl std::fmt::Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} ({}) - {} [{}]{}",
            self.product,
            self.release_version,
            self.release_type,
            self.release_date,
            self.file_name,
            if self.current { " [current]" } else { "" },
        )
    }
}

/// Marker types that select which releases a [`ReleaseFetcher`] requests.
pub mod kind {
    /// Selects only the current release.
    #[derive(Debug, Clone, Copy)]
    pub struct Current;

    /// Selects all available releases, including past releases.
    #[derive(Debug, Clone, Copy)]
    pub struct All;
}

/// A request builder for querying UTS release metadata.
///
/// `Type` identifies a release category, either by URI, name, or
/// [`ReleaseType`]. `Kind` is one of the marker types in [`kind`] and controls
/// whether [`send`](ReleaseFetcher::send) returns the current release or all
/// releases.
pub struct ReleaseFetcher<Type, Kind: Copy> {
    release_type: Type,
    kind: Kind,
}

impl<Type, Kind: Copy> ReleaseFetcher<Type, Kind> {
    /// Creates a fetcher for a release category and selection kind.
    pub fn new(release_type: Type, kind: Kind) -> Self {
        Self { release_type, kind }
    }

    /// Returns the selection-kind marker.
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

impl ReleaseFetcher<http::Uri, Current> {
    /// Returns the exact Release API URI that will be requested.
    pub fn uri(&self) -> &http::Uri {
        &self.release_type
    }

    /// Fetches the first release returned by the configured URI.
    ///
    /// Returns `None` when the API returns an empty array.
    pub async fn send<C: HttpClient>(
        &self,
        http_client: &C,
    ) -> Result<Option<Release>, http::Error> {
        let uri = self.uri();

        let request = http::Request::builder()
            .uri(uri.clone())
            .method(http::Method::GET)
            .body(())?;

        http_client
            .execute::<(), Vec<Release>>(request)
            .await
            .map(|x| x.into_body().first().cloned())
    }
}

impl ReleaseFetcher<http::Uri, kind::All> {
    /// Returns the exact Release API URI that will be requested.
    pub fn uri(&self) -> &http::Uri {
        &self.release_type
    }

    /// Fetches every release returned by the configured URI.
    pub async fn send<C: HttpClient>(&self, http_client: &C) -> Result<Vec<Release>, http::Error> {
        let uri = self.uri();

        let request = http::Request::builder()
            .uri(uri.clone())
            .method(http::Method::GET)
            .body(())?;

        http_client
            .execute::<(), Vec<Release>>(request)
            .await
            .map(|x| x.into_body())
    }
}

impl ReleaseFetcher<&str, Current> {
    /// Returns the configured release-type identifier.
    pub fn name(&self) -> &str {
        self.release_type
    }

    /// Fetches the current release for the configured release-type identifier.
    ///
    /// Returns `None` when the API returns an empty array.
    pub async fn send<C: HttpClient>(
        &self,
        http_client: &C,
    ) -> Result<Option<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}&current=true",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(uri, self.kind).send(http_client).await
    }
}

impl ReleaseFetcher<&str, kind::All> {
    /// Returns the configured release-type identifier.
    pub fn name(&self) -> &str {
        self.release_type
    }

    /// Fetches all releases for the configured release-type identifier.
    pub async fn send<C: HttpClient>(&self, http_client: &C) -> Result<Vec<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(uri, self.kind).send(http_client).await
    }
}

impl ReleaseFetcher<String, Current> {
    /// Returns the configured release-type identifier.
    pub fn name(&self) -> &str {
        &self.release_type
    }

    /// Fetches the current release for the configured release-type identifier.
    ///
    /// Returns `None` when the API returns an empty array.
    pub async fn send<C: HttpClient>(
        &self,
        http_client: &C,
    ) -> Result<Option<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}&current=true",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(uri, self.kind).send(http_client).await
    }
}

impl ReleaseFetcher<String, kind::All> {
    /// Returns the configured release-type identifier.
    pub fn name(&self) -> &str {
        &self.release_type
    }

    /// Fetches all releases for the configured release-type identifier.
    pub async fn send<C: HttpClient>(&self, http_client: &C) -> Result<Vec<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(uri, self.kind).send(http_client).await
    }
}

impl ReleaseFetcher<ReleaseType, Current> {
    /// Returns the release category used by this fetcher.
    pub fn release_type(&self) -> &ReleaseType {
        &self.release_type
    }

    /// Fetches the current release from the category's endpoint.
    ///
    /// Returns `None` when the API returns an empty array.
    pub async fn send<C: HttpClient>(
        &self,
        http_client: &C,
    ) -> Result<Option<Release>, http::Error> {
        let uri: http::Uri = self.release_type().endpoint().parse()?;

        ReleaseFetcher::new(uri, self.kind).send(http_client).await
    }
}

impl ReleaseFetcher<ReleaseType, kind::All> {
    /// Returns the release category used by this fetcher.
    pub fn release_type(&self) -> &ReleaseType {
        &self.release_type
    }

    /// Fetches all releases from the category's endpoint.
    pub async fn send<C: HttpClient>(&self, http_client: &C) -> Result<Vec<Release>, http::Error> {
        let uri: http::Uri = self.release_type().endpoint().parse()?;

        ReleaseFetcher::new(uri, self.kind).send(http_client).await
    }
}

#[async_trait]
/// An HTTP client capable of returning deserialized response bodies.
///
/// Implementations may add authentication, headers, and transport-specific
/// behavior before sending `request`.
pub trait HttpClient {
    /// Sends a request and deserializes its response body as `U`.
    async fn execute<T: Serialize, U: DeserializeOwned>(
        &self,
        request: http::Request<T>,
    ) -> Result<http::Response<U>, http::Error>;
}

/// A thread-safe, dynamically dispatched error produced while reading a stream.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A pinned stream of release-file byte chunks.
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send + 'static>>;

#[async_trait]
/// An HTTP client capable of streaming response bodies.
pub trait StreamHttpClient {
    /// Sends a request and returns its response body as a byte stream.
    async fn execute<T: Serialize>(
        &self,
        request: http::Request<T>,
    ) -> Result<http::Response<ByteStream>, http::Error>;
}

#[derive(Debug, Error)]
/// Errors that can occur while downloading a release.
pub enum Error {
    /// An error constructing or executing the HTTP request.
    #[error("HTTP error: {0}")]
    Http(#[source] http::Error),

    /// The UTS Download API base URL was invalid.
    #[error("URL parse error: {0}")]
    Url(#[source] url::ParseError),

    /// The constructed Download API URI was invalid.
    #[error("Invalid URI error: {0}")]
    Uri(#[source] http::uri::InvalidUri),

    /// An error creating, writing, or flushing the destination file.
    #[error("I/O Error: {0}")]
    Io(#[source] std::io::Error),

    /// An error yielded while reading the response body.
    #[error("response stream error: {0}")]
    Stream(#[source] BoxError),
}

impl From<http::Error> for Error {
    fn from(value: http::Error) -> Self {
        Self::Http(value)
    }
}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::Url(value)
    }
}

impl From<http::uri::InvalidUri> for Error {
    fn from(value: http::uri::InvalidUri) -> Self {
        Self::Uri(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<BoxError> for Error {
    fn from(value: BoxError) -> Self {
        Self::Stream(value)
    }
}

/// Opens a byte stream for a UTS release archive.
///
/// `release_download_uri` is the release's source URL (for example,
/// [`Release::download_url`] parsed as an [`http::Uri`]), not the UTS Download
/// API endpoint. The function supplies that URL and `api_key` to the Download
/// API and returns the response body without choosing where it is stored.
///
/// # Errors
///
/// Returns [`Error::Url`] or [`Error::Uri`] if the Download API request URL
/// cannot be constructed, and [`Error::Http`] if the request cannot be built or
/// sent. Errors encountered while consuming the returned stream are yielded as
/// [`BoxError`] values.
pub async fn download_release<C: StreamHttpClient>(
    http_client: &C,
    release_download_uri: http::Uri,
    api_key: &str,
) -> Result<ByteStream, Error> {
    let uri = download_uri(&release_download_uri, api_key)?;

    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(uri)
        .body(())?;

    Ok(http_client.execute(request).await?.into_body())
}

fn download_uri(release_download_uri: &http::Uri, api_key: &str) -> Result<http::Uri, Error> {
    let mut url = url::Url::parse("https://uts-ws.nlm.nih.gov/download")?;
    url.query_pairs_mut()
        .append_pair("url", &release_download_uri.to_string())
        .append_pair("apiKey", api_key);

    Ok(url.as_str().parse()?)
}

/// Downloads a UTS release archive to a local file.
///
/// This is a convenience wrapper around [`download_release`]. It replaces an
/// existing file at `destination`.
///
/// # Errors
///
/// Returns an error if the download request cannot be constructed or sent, the
/// response stream fails, or the destination file cannot be created, written,
/// or flushed.
pub async fn download_release_to_path<C: StreamHttpClient, P: AsRef<Path>>(
    http_client: &C,
    release_download_uri: http::Uri,
    api_key: &str,
    destination: P,
) -> Result<(), Error> {
    let mut stream = download_release(http_client, release_download_uri, api_key).await?;

    let file = File::create(destination)?;
    let mut writer = BufWriter::new(file);

    while let Some(chunk) = stream.next().await {
        writer.write_all(&chunk?)?;
    }

    writer.flush()?;

    Ok(())
}

/// Downloads a UTS release archive to a local file.
///
/// This compatibility alias delegates to [`download_release_to_path`]. New
/// code that needs destination-independent storage should use
/// [`download_release`] instead.
///
/// # Errors
///
/// Returns an error if the download request cannot be constructed or sent, the
/// response stream fails, or the destination file cannot be created, written,
/// or flushed.
#[deprecated(
    since = "0.1.0",
    note = "use download_release for a byte stream or download_release_to_path for a local file"
)]
pub async fn download_umls_terminology_service<C: StreamHttpClient, P: AsRef<Path>>(
    http_client: &C,
    release_download_uri: http::Uri,
    api_key: &str,
    destination: P,
) -> Result<(), Error> {
    download_release_to_path(http_client, release_download_uri, api_key, destination).await
}

#[cfg(test)]
mod tests {
    use super::download_uri;

    #[test]
    fn download_uri_encodes_query_parameter_values() {
        let release_uri = "https://example.com/a-release.zip?part=1&format=full"
            .parse()
            .unwrap();

        let uri = download_uri(&release_uri, "key+with/special=characters").unwrap();
        let url = url::Url::parse(&uri.to_string()).unwrap();
        let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

        assert_eq!(
            query.get("url").map(String::as_str),
            Some("https://example.com/a-release.zip?part=1&format=full")
        );
        assert_eq!(
            query.get("apiKey").map(String::as_str),
            Some("key+with/special=characters")
        );
    }
}
