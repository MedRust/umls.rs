//! Asynchronous helpers for discovering and downloading releases from the
//! [UMLS Terminology Services (UTS)][uts] Release and Download APIs.
//!
//! Enable the optional `reqwest` feature to execute requests with
//! `umls_http::reqwest::Client`. Cloning the client shares its connection pool.
//! [`release_type::Client`] fetches categories, and [`ReleaseFetcher`] fetches release metadata.
//! [`ReleaseFetcher::new`] takes the HTTP client, category, and selection kind;
//! its `send()` method consumes the fetcher. [`ReleaseDownloader::new`] takes an API
//! key and either an [`http::Uri`] for streaming or a [`DownloadResource`] for file
//! output. Attach the HTTP client with [`ReleaseDownloader::with_client`] before
//! calling `download()`.
//!
//! Without `reqwest`, metadata and builder types remain available, but HTTP
//! execution methods are disabled.
//!
//! [uts]: https://documentation.uts.nlm.nih.gov/automating-downloads.html
#![warn(missing_docs)]

#[cfg(feature = "reqwest")]
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

#[cfg(feature = "reqwest")]
use bytes::Bytes;
use chrono::NaiveDate;
#[cfg(feature = "reqwest")]
use futures_core::Stream;
#[cfg(feature = "reqwest")]
use futures_util::StreamExt;
#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(feature = "reqwest")]
use umls_http::reqwest;

#[cfg(feature = "reqwest")]
use crate::{kind::Current, release_type::ReleaseType};

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
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::WithoutClient;
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
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::WithoutClient;
    /// let fetcher = ReleaseFetcher::new(WithoutClient, "umls-full-release", Current);
    /// let selection: &Current = fetcher.kind();
    /// ```
    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, http::Uri, Current> {
    /// Returns the exact Release API URI that will be requested.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "https://uts-ws.nlm.nih.gov/releases".parse::<http::Uri>()?;
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, Current);
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
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///     "releaseType=umls-full-release&current=true").parse::<http::Uri>()?;
    /// let releases = ReleaseFetcher::new(Client::default(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Option<Release>, Error<reqwest::Error>> {
        let uri = self.uri();

        let request = http::Request::builder()
            .uri(uri.clone())
            .method(http::Method::GET)
            .body(())
            .map_err(reqwest::Error::Http)
            .map_err(Error::Client)?;

        self.client
            .execute::<(), Vec<Release>>(request)
            .await
            .map(|x| x.into_body().first().cloned())
            .map_err(Error::Client)
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, http::Uri, kind::All> {
    /// Returns the exact Release API URI that will be requested.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "https://uts-ws.nlm.nih.gov/releases".parse::<http::Uri>()?;
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, All);
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
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///     "releaseType=umls-full-release").parse::<http::Uri>()?;
    /// let releases = ReleaseFetcher::new(Client::default(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Vec<Release>, Error<reqwest::Error>> {
        let uri = self.uri();

        let request = http::Request::builder()
            .uri(uri.clone())
            .method(http::Method::GET)
            .body(())
            .map_err(reqwest::Error::Http)
            .map_err(Error::Client)?;

        self.client
            .execute::<(), Vec<Release>>(request)
            .await
            .map(|x| x.into_body())
            .map_err(Error::Client)
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, &str, Current> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, Current);
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
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let releases = ReleaseFetcher::new(Client::default(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Option<Release>, Error<reqwest::Error>> {
        let release_type = self.name();

        let uri = release_uri(release_type, true).map_err(Error::Uri)?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, &str, kind::All> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, All);
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
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = "umls-full-release";
    /// let releases = ReleaseFetcher::new(Client::default(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Vec<Release>, Error<reqwest::Error>> {
        let release_type = self.name();

        let uri = release_uri(release_type, false).map_err(Error::Uri)?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, String, Current> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, Current);
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
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let releases = ReleaseFetcher::new(Client::default(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Option<Release>, Error<reqwest::Error>> {
        let release_type = self.name();

        let uri = release_uri(release_type, true).map_err(Error::Uri)?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, String, kind::All> {
    /// Returns the configured release-type identifier.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, All);
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
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = String::from("umls-full-release");
    /// let releases = ReleaseFetcher::new(Client::default(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Vec<Release>, Error<reqwest::Error>> {
        let release_type = self.name();

        let uri = release_uri(release_type, false).map_err(Error::Uri)?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, ReleaseType, Current> {
    /// Returns the release category used by this fetcher.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(), name: "umls-full-release".into(),
    ///     endpoint: "https://uts-ws.nlm.nih.gov/releases".into(),
    /// };
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, Current);
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
    /// use umls_downloader::{ReleaseFetcher, kind::Current};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(),
    ///     name: "umls-full-release".into(),
    ///     endpoint: concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///         "releaseType=umls-full-release&current=true").into(),
    /// };
    /// let releases = ReleaseFetcher::new(Client::default(), category, Current)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Option<Release>, Error<reqwest::Error>> {
        let uri: http::Uri = self.release_type().endpoint().parse().map_err(Error::Uri)?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[cfg(feature = "reqwest")]
impl ReleaseFetcher<reqwest::Client, ReleaseType, kind::All> {
    /// Returns the release category used by this fetcher.
    ///
    /// The returned reference borrows the category supplied during construction without making
    /// a network request or modifying the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(), name: "umls-full-release".into(),
    ///     endpoint: "https://uts-ws.nlm.nih.gov/releases".into(),
    /// };
    /// let fetcher = ReleaseFetcher::new(Client::default(), category, All);
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
    /// use umls_downloader::{ReleaseFetcher, kind::All};
    /// use umls_http::reqwest::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let category = umls_downloader::release_type::ReleaseType {
    ///     product: "UMLS".into(),
    ///     name: "umls-full-release".into(),
    ///     endpoint: concat!("https://uts-ws.nlm.nih.gov/releases?",
    ///         "releaseType=umls-full-release").into(),
    /// };
    /// let releases = ReleaseFetcher::new(Client::default(), category, All)
    ///     .send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if URI or request construction, serialization, transport, HTTP 4xx or 5xx
    /// status validation, or release metadata decoding fails. Failures preserve typed client errors in [`Error::Client`]
    /// or report invalid URIs through [`Error::Uri`].
    pub async fn send(self) -> Result<Vec<Release>, Error<reqwest::Error>> {
        let uri: http::Uri = self.release_type().endpoint().parse().map_err(Error::Uri)?;

        ReleaseFetcher::new(self.client, uri, self.kind)
            .send()
            .await
    }
}

#[derive(Debug, Error)]
/// # Errors that can occur while downloading a release.
pub enum Error<T: std::error::Error> {
    /// An error constructing or executing the HTTP request, or reading its response body.
    #[error("Client error: {0}")]
    Client(#[source] T),

    /// The UTS Download API base URL was invalid.
    #[error("URL parse error: {0}")]
    Url(#[source] url::ParseError),

    /// A release endpoint or constructed Download API URI was invalid.
    #[error("Invalid URI error: {0}")]
    Uri(#[source] http::uri::InvalidUri),

    /// An error creating, writing, or flushing the destination file.
    #[error("I/O Error: {0}")]
    Io(#[source] std::io::Error),
}

impl<T: std::error::Error> From<url::ParseError> for Error<T> {
    fn from(value: url::ParseError) -> Self {
        Self::Url(value)
    }
}

impl<T: std::error::Error> From<std::io::Error> for Error<T> {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// Builds a release download operation with a client and destination value.
///
/// A downloader is created without a client and transitions to an executable state with
/// [`Self::with_client`]. The destination type selects the operation: an [`http::Uri`] produces a
/// stream of byte chunks, while [`DownloadResource`] writes the response to a path.
///
/// # Examples
///
/// ```
/// use umls_downloader::{ReleaseDownloader};
/// use umls_http::WithoutClient;
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

impl<T> ReleaseDownloader<umls_http::WithoutClient, T> {
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
            client: umls_http::WithoutClient,
            value,
            api_key: api_key.into(),
        }
    }

    /// Attaches an HTTP client and returns the resulting downloader.
    ///
    /// This method consumes the current downloader and performs no network I/O. The client is
    /// owned by the returned value, so it can be reused by cloning a cloneable client first.
    /// Download methods require `umls_http::reqwest::Client` and the `reqwest` feature.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::{ReleaseDownloader};
    /// use umls_http::WithoutClient;
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
impl ReleaseDownloader<reqwest::Client, http::Uri> {
    /// Starts a release download and returns its response body as a stream.
    ///
    /// The request is sent when this method is awaited. The returned stream owns the response
    /// body and yields byte chunks as they arrive, with failures reported as [`reqwest::Error`].
    /// Pin the stream before consuming it with [`StreamExt::next`]. It does not create or modify
    /// a file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{Release, ReleaseDownloader};
    /// use umls_http::reqwest::Client;
    /// # async fn example(release: &Release) -> Result<(), Box<dyn std::error::Error>> {
    /// let api_key = std::env::var("UMLS_API_KEY")?;
    /// use futures_util::StreamExt;
    /// let source = release.download_url.parse::<http::Uri>()?;
    /// let downloader = ReleaseDownloader::new(api_key, source).with_client(Client::default());
    /// let mut stream = std::pin::pin!(downloader.download().await?);
    /// while let Some(chunk) = stream.next().await {
    ///     println!("Received {} bytes", chunk?.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Url`] or [`Error::Uri`] when the UTS download URL cannot be constructed,
    /// [`Error::Client`] when the request or response cannot be built, or an error produced by the
    /// underlying HTTP client. Errors encountered while reading the body are yielded by the
    /// returned stream.
    pub async fn download(
        &self,
    ) -> Result<
        impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static + use<>,
        Error<reqwest::Error>,
    > {
        let uri = download_uri(&self.value, self.api_key.expose_secret())?;

        self.download_from_uri(uri).await
    }

    async fn download_from_uri(
        &self,
        uri: http::Uri,
    ) -> Result<
        impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static + use<>,
        Error<reqwest::Error>,
    > {
        let request = http::Request::builder()
            .method(http::Method::GET)
            .uri(uri)
            .body(())
            .map_err(reqwest::Error::Http)
            .map_err(Error::Client)?;

        self.client
            .execute_stream(request)
            .await
            .map(|x| x.into_body())
            .map_err(Error::Client)
    }
}

/// Associates a release URI with a destination value.
///
/// When used with [`ReleaseDownloader`], a destination implementing [`AsRef<std::path::Path>`] causes the
/// downloaded response to be written to that path. Existing files are replaced. The destination
/// may be a [`PathBuf`](std::path::PathBuf), `&Path`, or another path-like type.
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
impl<D: AsRef<Path>> ReleaseDownloader<reqwest::Client, DownloadResource<D>> {
    /// Downloads the release and writes it to the configured destination path.
    ///
    /// Existing files are replaced. File creation and writes are synchronous, so callers should
    /// avoid invoking this operation on a latency-sensitive executor thread for large files.
    /// A failed request, stream error, or cancellation can leave a partial file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{Release, ReleaseDownloader, DownloadResource};
    /// use umls_http::reqwest::Client;
    /// # async fn example(release: &Release) -> Result<(), Box<dyn std::error::Error>> {
    /// let api_key = std::env::var("UMLS_API_KEY")?;
    /// let resource = DownloadResource::new(release.download_url.parse()?, "release.zip");
    /// ReleaseDownloader::new(api_key, resource)
    ///     .with_client(Client::default())
    ///     .download().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] when the destination cannot be created, written, or flushed. Returns
    /// [`Error::Client`] when reading the response body fails, or another [`enum@Error`] variant when
    /// request construction or the HTTP operation fails.
    pub async fn download(self) -> Result<(), Error<reqwest::Error>> {
        let stream = ReleaseDownloader::new(self.api_key.clone(), self.value.source)
            .with_client(self.client.clone())
            .download()
            .await?;

        write_download(stream, self.value.destination).await
    }
}

#[cfg(feature = "reqwest")]
async fn write_download(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>>,
    destination: impl AsRef<Path>,
) -> Result<(), Error<reqwest::Error>> {
    let mut stream = std::pin::pin!(stream);
    let file = File::create(destination)?;
    let mut writer = BufWriter::new(file);
    while let Some(chunk) = stream.next().await {
        writer.write_all(&chunk.map_err(Error::Client)?)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(feature = "reqwest")]
fn release_uri(name: &str, current: bool) -> Result<http::Uri, http::uri::InvalidUri> {
    let filter = if current { "&current=true" } else { "" };
    format!("https://uts-ws.nlm.nih.gov/releases?releaseType={name}{filter}").parse()
}

#[cfg(feature = "reqwest")]
fn download_uri(
    release_download_uri: &http::Uri,
    api_key: &str,
) -> Result<http::Uri, Error<reqwest::Error>> {
    let mut url = url::Url::parse("https://uts-ws.nlm.nih.gov/download")?;
    url.query_pairs_mut()
        .append_pair("url", &release_download_uri.to_string())
        .append_pair("apiKey", api_key);

    url.as_str().parse().map_err(Error::Uri)
}

#[cfg(test)]
mod tests {

    use super::*;
    use httpmock::prelude::*;
    use std::{error::Error as _, time::Duration};

    #[test]
    fn category_names_select_current_or_all() {
        for current in [false, true] {
            let uri = release_uri("umls-full-release", current).unwrap();
            let url = url::Url::parse(&uri.to_string()).unwrap();
            let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
            assert_eq!(pairs["releaseType"], "umls-full-release");
            assert_eq!(
                pairs.get("current").map(String::as_str),
                current.then_some("true")
            );
        }
        assert!(release_uri("invalid category", false).is_err());
    }

    #[test]
    fn outer_errors_preserve_original_sources() {
        let uri = "invalid uri".parse::<http::Uri>().unwrap_err();
        let error: Error<reqwest::Error> = Error::Uri(uri);
        assert!(error.source().unwrap().is::<http::uri::InvalidUri>());
        let error: Error<reqwest::Error> = url::Url::parse(":invalid").unwrap_err().into();
        assert!(error.source().unwrap().is::<url::ParseError>());
        let error: Error<reqwest::Error> =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied").into();
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        let original = http::Request::builder()
            .header("bad\nname", "value")
            .body(())
            .unwrap_err();
        let error = Error::Client(reqwest::Error::from(original));
        assert!(error.source().unwrap().is::<reqwest::Error>());
        assert!(
            error
                .source()
                .unwrap()
                .source()
                .unwrap()
                .is::<http::Error>()
        );
        let original = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error = reqwest::Error::from(original);
        assert!(error.source().unwrap().is::<serde_json::Error>());
    }

    #[tokio::test]
    async fn file_output_creates_and_replaces_with_all_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("archive.zip");
        for existing in [false, true] {
            if existing {
                std::fs::write(&path, b"old content longer than new archive").unwrap();
            }
            let stream = futures_util::stream::iter([
                Ok(Bytes::from_static(b"abc")),
                Ok(Bytes::from_static(b"def")),
            ]);
            write_download(stream, &path).await.unwrap();
            assert_eq!(std::fs::read(&path).unwrap(), b"abcdef");
        }
    }

    #[tokio::test]
    async fn invalid_destination_returns_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let error = write_download(futures_util::stream::empty(), dir.path())
            .await
            .unwrap_err();
        assert!(matches!(error, Error::Io(_)));
        assert!(error.source().unwrap().is::<std::io::Error>());
    }

    #[tokio::test]
    async fn stream_error_is_preserved_by_file_output() {
        let original = http::Request::builder()
            .header("bad\nname", "value")
            .body(())
            .unwrap_err();
        let dir = tempfile::tempdir().unwrap();
        let error = write_download(
            futures_util::stream::iter([Err(reqwest::Error::Http(original))]),
            dir.path().join("partial.zip"),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, Error::Client(reqwest::Error::Http(_))));
        assert!(
            error
                .source()
                .unwrap()
                .source()
                .unwrap()
                .is::<http::Error>()
        );
    }

    #[tokio::test]
    async fn download_response_stream_writes_local_archive() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/download")
                    .query_param("apiKey", "test+key")
                    .query_param("url", "https://example.test/archive.zip");
                then.body("archive bytes");
            })
            .await;
        let source = "https://example.test/archive.zip".parse().unwrap();
        let downloader = ReleaseDownloader::new("test+key", source)
            .with_client(umls_http::reqwest::Client::default());
        // Keep the production query construction; route only the origin to the local server.
        let uri = download_uri(&downloader.value, "test+key").unwrap();
        let local_uri = format!("{}{}", server.base_url(), uri.path_and_query().unwrap())
            .parse()
            .unwrap();
        let stream = downloader.download_from_uri(local_uri).await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("archive.zip");
        write_download(stream, &path).await.unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"archive bytes");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn cancelling_delayed_file_download_drops_stream_and_leaves_incomplete_file() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (started, ready) = tokio::sync::oneshot::channel();
        let (finish, wait) = tokio::sync::oneshot::channel();
        let server =
            tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    request.push(socket.read_u8().await.unwrap());
                }
                socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\nConnection: close\r\n\r\nabc")
            .await
            .unwrap();
                wait.await.unwrap();
            });
        let downloader = ReleaseDownloader::new(
            "key",
            "https://example.test/archive".parse::<http::Uri>().unwrap(),
        )
        .with_client(umls_http::reqwest::Client::default());
        let stream = downloader
            .download_from_uri(format!("http://{address}/download").parse().unwrap())
            .await
            .unwrap();
        let mut signal = Some(started);
        let stream = stream.inspect(move |_| {
            if let Some(signal) = signal.take() {
                let _ = signal.send(());
            }
        });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("partial.zip");
        let task = tokio::spawn(write_download(stream, path.clone()));
        tokio::time::timeout(Duration::from_secs(3), ready)
            .await
            .unwrap()
            .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert!(std::fs::metadata(path).unwrap().len() < 100000);
        finish.send(()).unwrap();
        server.await.unwrap();
    }

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
