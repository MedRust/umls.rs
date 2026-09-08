//! Asynchronous helpers for discovering and downloading releases from the
//! [UMLS Terminology Services (UTS)][uts] Release and Download APIs.
//!
//! The `reqwest` feature enables concrete clients for release discovery and streaming downloads.
//! [`release_type::Client`] fetches categories, and [`ReleaseFetcher`] fetches release metadata.
//! Download operations return a stream or write an archive to a caller-selected path.
//!
//! [uts]: https://documentation.uts.nlm.nih.gov/automating-downloads.html
#![warn(missing_docs)]

use std::pin::Pin;
#[cfg(feature = "reqwest")]
use std::{fs::File, io::{BufWriter, Write}, path::Path};

use bytes::Bytes;
use chrono::NaiveDate;
use futures_core::Stream;
use futures_util::StreamExt;
#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "reqwest")]
use crate::client::ReqwestClient;

use crate::client::WithoutClient;
#[cfg(feature = "reqwest")]
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, Current);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, Current)
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, All);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, All)
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, Current);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, Current)
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, All);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, All)
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, Current);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, Current)
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, All);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, All)
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, Current);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, Current)
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
    /// let fetcher = ReleaseFetcher::new(Default::default(), category, All);
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
    /// let releases = ReleaseFetcher::new(Default::default(), category, All)
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

/// A thread-safe, dynamically dispatched error produced while reading a stream.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A pinned stream of release-file byte chunks.
pub type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, BoxError>> + Send + 'static>>;

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

/// Builds a release download operation with a client and destination value.
///
/// A downloader is created without a client and transitions to an executable state with
/// [`Self::with_client`]. The destination type selects the operation: an [`http::Uri`] produces a
/// [`ByteStream`], while [`DownloadResource`] writes the response to a path.
///
/// # Examples
///
/// ```
/// use umls_downloader::{ReleaseDownloader, client::WithoutClient};
/// let source: http::Uri = "https://example.com/release.zip".parse()?;
/// let downloader = ReleaseDownloader::new("api-key", source);
/// let _ready = downloader.with_client(WithoutClient);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ReleaseDownloader<C, T> {
    client: C,
    value: T,
    api_key: SecretString,
}

impl<T> ReleaseDownloader<WithoutClient, T> {
    /// Creates a downloader without attaching an HTTP client.
    ///
    /// This method performs no I/O. The API key is retained for the eventual UTS download request.
    /// Call [`Self::with_client`] before calling an operation-specific `download` method.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::ReleaseDownloader;
    /// let source: http::Uri = "https://example.com/release.zip".parse()?;
    /// let downloader = ReleaseDownloader::new("api-key", source);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(api_key: impl Into<SecretString>, value: T) -> Self {
        Self {
            client: WithoutClient,
            value,
            api_key: api_key.into(),
        }
    }

    /// Attaches an HTTP client and returns an executable downloader.
    ///
    /// This method consumes the current downloader and performs no network I/O. The client is
    /// owned by the returned value, so it can be reused by cloning a cloneable client first.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseDownloader, client::WithoutClient};
    /// let source: http::Uri = "https://example.com/release.zip".parse()?;
    /// let downloader = ReleaseDownloader::new("api-key", source).with_client(WithoutClient);
    /// # let _ = downloader;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_client<C>(self, client: C) -> ReleaseDownloader<C, T> {
        ReleaseDownloader {
            client,
            value: self.value,
            api_key: self.api_key,
        }
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseDownloader<ReqwestClient, http::Uri> {
    /// Starts a release download and returns its response body as a stream.
    ///
    /// The request is sent lazily when this method is awaited. The returned [`ByteStream`] owns
    /// the response body and yields chunks as they arrive. It does not create or modify a file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Url`] or [`Error::Uri`] when the UTS download URL cannot be constructed,
    /// [`Error::Http`] when the request or response cannot be built, or an error produced by the
    /// underlying HTTP client. Errors encountered while reading the body are yielded by the
    /// returned stream.
    pub async fn download(&self) -> Result<ByteStream, Error> {
        let uri = download_uri(&self.value, self.api_key.expose_secret())?;

        let request = http::Request::builder()
            .method(http::Method::GET)
            .uri(uri)
            .body(())?;

        Ok(self.client.execute_stream(request).await?.into_body())
    }
}

/// Associates a release URI with a destination value.
///
/// When used with [`ReleaseDownloader`], a destination implementing [`AsRef<Path>`] causes the
/// downloaded response to be written to that path. Existing files are replaced. The destination
/// may be a [`PathBuf`], `&Path`, or another path-like type.
///
/// # Examples
///
/// ```
/// use umls_downloader::DownloadResource;
/// let source = "https://example.com/release.zip".parse()?;
/// let target = DownloadResource::new(source, "release.zip");
/// # let _ = target;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct DownloadResource<D> {
    source: http::Uri,
    destination: D,
}

impl<D> DownloadResource<D> {
    /// Associates a source URI with a destination.
    ///
    /// This method performs no I/O and does not validate whether the URI or destination exists.
    /// Validation occurs when the associated downloader is executed.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::DownloadResource;
    /// let source = "https://example.com/release.zip".parse()?;
    /// let target = DownloadResource::new(source, "release.zip");
    /// # let _ = target;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(source: http::Uri, destination: D) -> Self {
        Self {
            source,
            destination,
        }
    }
}

#[cfg(feature = "reqwest")]
impl<D: AsRef<Path>> ReleaseDownloader<ReqwestClient, DownloadResource<D>> {
    /// Downloads the release and writes it to the configured destination path.
    ///
    /// Existing files are replaced. File creation and writes are synchronous, so callers should
    /// avoid invoking this operation on a latency-sensitive executor thread for large files.
    /// A failed request, stream error, or cancellation can leave a partial file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] when the destination cannot be created, written, or flushed. Returns
    /// [`Error::Stream`] when reading the response body fails, or another [`Error`] variant when
    /// request construction or the HTTP operation fails.
    pub async fn download(self) -> Result<(), Error> {
        let mut stream = ReleaseDownloader::new(self.api_key.clone(), self.value.source)
            .with_client(self.client.clone())
            .download()
            .await?;

        let file = File::create(self.value.destination)?;
        let mut writer = BufWriter::new(file);

        while let Some(chunk) = stream.next().await {
            writer.write_all(&chunk?)?;
        }

        writer.flush()?;

        Ok(())
    }
}

fn download_uri(release_download_uri: &http::Uri, api_key: &str) -> Result<http::Uri, Error> {
    let mut url = url::Url::parse("https://uts-ws.nlm.nih.gov/download")?;
    url.query_pairs_mut()
        .append_pair("url", &release_download_uri.to_string())
        .append_pair("apiKey", api_key);

    Ok(url.as_str().parse()?)
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
