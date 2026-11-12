use std::marker::PhantomData;

use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use umls_http::reqwest;

#[cfg(feature = "reqwest")]
use crate::dto::atom::AtomResponse;

#[cfg(feature = "reqwest")]
use crate::core::aui::Aui;
#[cfg(feature = "reqwest")]
use crate::core::url::validator;
#[cfg(feature = "reqwest")]
use crate::dto;

#[cfg(feature = "reqwest")]
use crate::dto::concept::Payload;
#[cfg(feature = "reqwest")]
use crate::dto::page::Page;
#[cfg(feature = "reqwest")]
use crate::types;

#[cfg(feature = "reqwest")]
use crate::{config::Config, core::cui::Cui, search::ConceptSnapshot};

#[cfg(feature = "reqwest")]
#[derive(Debug, thiserror::Error)]
/// An error produced while locating or retrieving a concept.
pub enum Error<T> {
    /// A client error.
    #[error("Client error: {0}")]
    Client(#[source] T),

    /// Represents uri construction error.
    #[error("Uri construction Error: {0}")]
    Uri(#[source] http::uri::InvalidUri),

    /// Represents URL validation error.
    #[error("Url validation Error: {0}")]
    Validator(#[source] validator::Error),
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for Error<reqwest::Error> {
    fn from(value: reqwest::Error) -> Self {
        Error::Client(value)
    }
}

#[cfg(feature = "reqwest")]
impl<T> From<http::uri::InvalidUri> for Error<T> {
    fn from(value: http::uri::InvalidUri) -> Self {
        Error::Uri(value)
    }
}

/// A typed reference to information that can identify a UMLS concept.
///
/// The available `fetch` method depends on whether the value is a [`crate::core::cui::Cui`], a
/// [`http::Uri`], or a [`crate::search::ConceptSnapshot`]. Attach a `reqwest::Client` using
/// [`Self::with_client`] before fetching; fetching consumes the request.
pub struct Info<C, M, T> {
    client: C,
    value: T,
    _marker: PhantomData<M>,
}

impl<C, M, T> Info<C, M, T> {
    /// Returns the wrapped reference value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the wraper reference client.
    pub fn client(&self) -> &C {
        &self.client
    }
}

impl<M, T> Info<WithoutClient, M, T> {
    /// Wraps a concept reference value.
    pub fn new(value: T) -> Self {
        Self {
            client: WithoutClient,
            value,
            _marker: PhantomData,
        }
    }

    /// Attaches an owned HTTP client without sending a request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// use umls_http::reqwest;
    /// use umls_client::{info::Info, core::cui::Cui, config::Config};
    /// # #[cfg(feature = "reqwest")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
    /// let client = reqwest::Client::default();
    /// let concept = Info::new(Cui::new("C0010200")?)
    ///     .with_client(client).fetch(&config).await?;
    /// println!("{}", concept.name);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Execution requires the `reqwest` feature and a `reqwest::Client`.
    /// Clone an existing client to share its connection pool across requests.
    pub fn with_client<C>(self, client: C) -> Info<C, M, T> {
        Info {
            client,
            value: self.value,
            _marker: PhantomData,
        }
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, types::Concept, Cui> {
    /// Retrieves the concept identified by this CUI.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if endpoint construction, transport, HTTP status
    /// validation, or response decoding fails.
    pub async fn fetch(
        self,
        config: &Config,
    ) -> Result<dto::concept::Payload, Error<reqwest::Error>> {
        let uri = config.info_url::<types::Concept, _>(self.value).url();

        Info::<_, types::Concept, _>::new(uri)
            .with_client(self.client)
            .fetch(config)
            .await
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, types::Concept, url::Url> {
    /// Retrieves a concept from the wrapped UMLS resource URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if URL construction, transport, HTTP status
    /// validation, or response decoding fails.
    pub async fn fetch(
        self,
        config: &Config,
    ) -> Result<dto::concept::Payload, Error<reqwest::Error>> {
        config
            .info_url_validator::<types::Concept>(self.value())
            .validate()
            .map_err(Error::Validator)?;

        let url = config.authenticated_url(self.value);

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<dto::concept::ConceptResponse>()
            .await
            .map(|x| x.result)
            .map(Into::<Payload>::into)
            .map_err(Error::Client)
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, types::Concept, ConceptSnapshot> {
    /// Retrieves the full concept represented by a search-result snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the concept request cannot be completed or decoded.
    pub async fn fetch(
        self,
        config: &Config,
    ) -> Result<dto::concept::Payload, Error<reqwest::Error>> {
        self.value.info.with_client(self.client).fetch(config).await
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, types::Atom, Aui> {
    /// Fetch atom information for given atom.
    pub async fn fetch(self, config: &Config) -> Result<AtomResponse, Error<reqwest::Error>> {
        let url = config.info_url::<types::Atom, _>(&self.value).url();

        self.client()
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Page<AtomResponse>>()
            .await
            .map(|x| x.content)
            .map_err(Error::Client)
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, types::Source, url::Url> {
    /// Fetches source-asserted concept information for given source concept.
    ///
    /// The method validates, add authorization, send request and handle response.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// use umls_http::reqwest;
    /// use umls_client::{info::Info, core::cui::Cui, config::Config, types};
    /// # #[cfg(feature = "reqwest")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
    /// let client = reqwest::Client::default();
    /// let url =
    /// "https://uts-ws.nlm.nih.gov/rest/content/current/source/SNOMEDCT_US/9468002".parse::<url::Url>()?;
    /// let concept = Info::<_, types::Source, _>::new(url)
    ///     .with_client(client).fetch(&config).await?;
    /// println!("{}", concept.name());
    /// # Ok(())
    /// # }
    /// ```
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// use umls_http::reqwest;
    ///
    /// ```
    ///
    /// # Errors
    /// - [`Error::Client`]: When HTTP failure and JSON deserialization error.
    /// - [`Error::Validator`]: When URL validation failure.
    pub async fn fetch(
        self,
        config: &Config,
    ) -> Result<dto::source::SourceAtomCluster, Error<reqwest::Error>> {
        config
            .info_url_validator::<types::Source>(self.value())
            .validate()
            .map_err(Error::Validator)?;

        let client = self.client;
        let url = config.authenticated_url(self.value);

        Ok(client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Page<dto::source::SerializableSourceAtomCluster>>()
            .await?
            .content
            .try_into()
            .unwrap())
    }
}
