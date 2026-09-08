//! Asynchronous helpers for discovering and downloading releases from the
//! [UMLS Terminology Services (UTS)][uts] Release and Download APIs.
//!
//! The `reqwest` feature enables concrete clients for release discovery and streaming downloads.
//! [`release_type::Client`] fetches categories, and [`ReleaseFetcher`] fetches release metadata.
//! Download operations return a stream or write an archive to a caller-selected path.
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

#[cfg(feature = "reqwest")]
use crate::client::ReqwestClient;

use crate::{kind::Current, release_type::ReleaseType};

/// Provides concrete HTTP clients and a marker for builders without a client.
pub mod client;
/// Provides release-category metadata and category discovery.
pub mod release_type;

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

/// Fetches metadata for a selected UTS release category.
///
/// `Type` identifies a release category, either by URI, name, or
/// [`ReleaseType`]. `Kind` is one of the marker types in [`kind`] and controls
/// whether fetching returns the first release or all releases. Execution is available with
/// the `reqwest` feature and consumes the fetcher. Cloning the HTTP client before construction
/// allows its connection pool to be reused across fetchers.
pub struct ReleaseFetcher<C, Type, Kind: Copy> {
    client: C,
    release_type: Type,
    kind: Kind,
}

impl<C, Type, Kind: Copy> ReleaseFetcher<C, Type, Kind> {
    /// Creates a fetcher using the supplied client, release category, and selection kind.
    ///
    /// Construction takes ownership of its arguments and does not make a network request. A
    /// category can be an identifier, an explicit [`http::Uri`], or a [`ReleaseType`].
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::WithoutClient, kind::Current};
    /// let fetcher = ReleaseFetcher::new(WithoutClient, "umls-full-release", Current);
    /// ```
    pub fn new(client: C, release_type: Type, kind: Kind) -> Self {
        Self {
            client,
            release_type,
            kind,
        }
    }

    /// Returns the selection-kind marker.
    ///
    /// The returned reference has the same selection type supplied during construction.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::WithoutClient, kind::Current};
    /// let fetcher = ReleaseFetcher::new(WithoutClient, "umls-full-release", Current);
    /// let selection: &Current = fetcher.kind();
    /// ```
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, http::Uri, Current> {
    /// Returns the exact Release API URI that will be requested.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "https://uts-ws.nlm.nih.gov/releases".parse::<http::Uri>()?;
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, Current);
    /// let selected = fetcher.uri();
    /// # Ok(())
    /// # }
    /// ```
    pub fn uri(&self) -> &http::Uri {
        &self.release_type
    }

    /// Fetches the first release for the selected category.
    ///
    /// Returns [`None`] for an empty response array and otherwise returns its first release.
    /// The endpoint is used as supplied; the selection marker does not add query parameters.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///     "releaseType=umls-full-release&current=true").parse::<http::Uri>()?;
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Option<Release>, http::Error> {
        let uri = self.uri();

        let request = http::Request::builder()
            .uri(uri.clone())
            .method(http::Method::GET)
            .body(())?;

        self.client
            .execute::<(), Vec<Release>>(request)
            .await
            .map(|x| x.into_body().first().cloned())
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, http::Uri, kind::All> {
    /// Returns the exact Release API URI that will be requested.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "https://uts-ws.nlm.nih.gov/releases".parse::<http::Uri>()?;
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, All);
    /// let selected = fetcher.uri();
    /// # Ok(())
    /// # }
    /// ```
    pub fn uri(&self) -> &http::Uri {
        &self.release_type
    }

    /// Fetches all releases for the selected category.
    ///
    /// Returns every release in response order, including an empty [`Vec`] for an empty array.
    /// The endpoint is used as supplied; the selection marker does not add query parameters.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///     "releaseType=umls-full-release&current=true").parse::<http::Uri>()?;
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Vec<Release>, http::Error> {
        let uri = self.uri();

        let request = http::Request::builder()
            .uri(uri.clone())
            .method(http::Method::GET)
            .body(())?;

        self.client
            .execute::<(), Vec<Release>>(request)
            .await
            .map(|x| x.into_body())
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, &str, Current> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, Current);
    /// let selected = fetcher.name();
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        self.release_type
    }

    /// Fetches the first release for the selected category.
    ///
    /// Returns [`None`] for an empty response array and otherwise returns its first release.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Option<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}&current=true",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, &str, kind::All> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, All);
    /// let selected = fetcher.name();
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        self.release_type
    }

    /// Fetches all releases for the selected category.
    ///
    /// Returns every release in response order, including an empty [`Vec`] for an empty array.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Vec<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, String, Current> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, Current);
    /// let selected = fetcher.name();
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        &self.release_type
    }

    /// Fetches the first release for the selected category.
    ///
    /// Returns [`None`] for an empty response array and otherwise returns its first release.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Option<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}&current=true",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, String, kind::All> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, All);
    /// let selected = fetcher.name();
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        &self.release_type
    }

    /// Fetches all releases for the selected category.
    ///
    /// Returns every release in response order, including an empty [`Vec`] for an empty array.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Vec<Release>, http::Error> {
        let release_type = self.name();

        let uri: http::Uri = format!(
            "https://uts-ws.nlm.nih.gov/releases?releaseType={}",
            release_type
        )
        .parse()?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, ReleaseType, Current> {
    /// Returns the release category used by this fetcher.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(), name: "umls-full-release".into(),
    ///     endpoint: "https://uts-ws.nlm.nih.gov/releases".into(),
    /// };
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, Current);
    /// let selected = fetcher.release_type();
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_type(&self) -> &ReleaseType {
        &self.release_type
    }

    /// Fetches the first release for the selected category.
    ///
    /// Returns [`None`] for an empty response array and otherwise returns its first release.
    /// The endpoint is used as supplied; the selection marker does not add query parameters.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::Current};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(),
    ///     name: "umls-full-release".into(),
    ///     endpoint: concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///         "releaseType=umls-full-release&current=true").into(),
    /// };
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Option<Release>, http::Error> {
        let uri: http::Uri = self.release_type().endpoint().parse()?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<client::ReqwestClient, ReleaseType, kind::All> {
    /// Returns the release category used by this fetcher.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(), name: "umls-full-release".into(),
    ///     endpoint: "https://uts-ws.nlm.nih.gov/releases".into(),
    /// };
    /// let fetcher = ReleaseFetcher::new(ReqwestClient::new(), category, All);
    /// let selected = fetcher.release_type();
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_type(&self) -> &ReleaseType {
        &self.release_type
    }

    /// Fetches all releases for the selected category.
    ///
    /// Returns every release in response order, including an empty [`Vec`] for an empty array.
    /// The endpoint is used as supplied; the selection marker does not add query parameters.
    /// This operation consumes the fetcher.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{ReleaseFetcher, client::ReqwestClient, kind::All};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(),
    ///     name: "umls-full-release".into(),
    ///     endpoint: concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///         "releaseType=umls-full-release&current=true").into(),
    /// };
    /// let releases = ReleaseFetcher::new(ReqwestClient::new(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures use [`http::Error`].
    pub async fn send(self) -> Result<Vec<Release>, http::Error> {
        let uri: http::Uri = self.release_type().endpoint().parse()?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[async_trait]
/// An HTTP client capable of returning deserialized response bodies.
///
/// Implementations may add authentication, headers, and transport-specific
/// behavior before sending `request`.
pub trait HttpClient {
    /// Sends a request and deserializes its response body as `U`.
    async fn execute<T: Serialize + Send, U: DeserializeOwned>(
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
    async fn execute<T: Serialize + Send>(
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
/// Takes ownership of the client; a clone can be passed to share connections with later requests.
/// The returned stream owns the response and can be consumed after this call returns.
///
/// # Examples
///
/// ```no_run
/// # #[allow(deprecated)]
/// use umls_downloader::{client::ReqwestClient, download_release};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ReqwestClient::new();
/// let source = "https://download.nlm.nih.gov/umls/kss/2026AA/umls-2026AA-full.zip".parse()?;
/// let key = std::env::var("UMLS_API_KEY")?;
/// # #[allow(deprecated)]
/// let stream = download_release(client, source, &key).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`Error::Url`] or [`Error::Uri`] if the Download API request URL
/// cannot be constructed, and [`Error::Http`] if the request cannot be built or
/// sent. Errors encountered while consuming the returned stream are yielded as
/// [`BoxError`] values.
pub async fn download_release(
    client: ReqwestClient,
    release_download_uri: http::Uri,
    api_key: &str,
) -> Result<ByteStream, Error> {
    let uri = download_uri(&release_download_uri, api_key)?;

    let request = http::Request::builder()
        .method(http::Method::GET)
        .uri(uri)
        .body(())?;

    Ok(client.execute_stream(request).await?.into_body())
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
/// Takes ownership of the client; a clone can be passed to share connections with later requests.
/// A failure or cancellation can leave a partial file. Local file operations are synchronous.
///
/// # Examples
///
/// ```no_run
/// # #[allow(deprecated)]
/// use umls_downloader::{client::ReqwestClient, download_release_to_path};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ReqwestClient::new();
/// let source = "https://download.nlm.nih.gov/umls/kss/2026AA/umls-2026AA-full.zip".parse()?;
/// let key = std::env::var("UMLS_API_KEY")?;
/// # #[allow(deprecated)]
/// download_release_to_path(client, source, &key, "release.zip").await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the download request cannot be constructed or sent, the
/// response stream fails, or the destination file cannot be created, written,
/// or flushed.
pub async fn download_release_to_path<P: AsRef<Path>>(
    client: ReqwestClient,
    release_download_uri: http::Uri,
    api_key: &str,
    destination: P,
) -> Result<(), Error> {
    let mut stream = download_release(client, release_download_uri, api_key).await?;

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
/// Takes ownership of the client; a clone can be passed to share connections with later requests.
/// A failure or cancellation can leave a partial file. Local file operations are synchronous.
///
/// # Examples
///
/// ```no_run
/// # #[allow(deprecated)]
/// use umls_downloader::{client::ReqwestClient, download_umls_terminology_service};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ReqwestClient::new();
/// let source = "https://download.nlm.nih.gov/umls/kss/2026AA/umls-2026AA-full.zip".parse()?;
/// let key = std::env::var("UMLS_API_KEY")?;
/// # #[allow(deprecated)]
/// download_umls_terminology_service(client, source, &key, "release.zip").await?;
/// # Ok(())
/// # }
/// ```
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
pub async fn download_umls_terminology_service<P: AsRef<Path>>(
    client: ReqwestClient,
    release_download_uri: http::Uri,
    api_key: &str,
    destination: P,
) -> Result<(), Error> {
    download_release_to_path(client, release_download_uri, api_key, destination).await
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
