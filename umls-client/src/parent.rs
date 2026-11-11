#[cfg(feature = "reqwest")]
use crate::core::url::validator;

#[cfg(feature = "reqwest")]
use crate::types;

use std::marker::PhantomData;

#[cfg(feature = "reqwest")]
use crate::dto::source::{SerializableSourceAtomCluster, SourceAtomCluster, SourceConcept};

#[cfg(feature = "reqwest")]
use umls_http::reqwest;

#[cfg(feature = "reqwest")]
use crate::core::aui::Aui;

#[cfg(feature = "reqwest")]
use crate::search::core::EmptyString;
#[cfg(feature = "reqwest")]
use crate::search::{self, Response};

#[cfg(feature = "reqwest")]
use crate::dto::{
    atom::AtomResponse,
    batch::{Batch, BatchError},
};

#[cfg(feature = "reqwest")]
use crate::{
    config,
    dto::{content::Content, page::Page},
};

/// UMLS parent client.
pub struct Parent<U, C, T> {
    client: C,
    value: T,
    _type: PhantomData<U>,
}

impl<U, T> Parent<U, umls_http::WithoutClient, T> {
    /// Create and return a new instance of [`Parent`] for given value.
    ///
    /// The client is created with http client [`umls_http::WithoutClient`].
    pub fn new(value: T) -> Self {
        Self {
            client: umls_http::WithoutClient,
            value,
            _type: PhantomData,
        }
    }

    /// Create a new [`Parent`] object with specified client.
    pub fn with_client<C>(self, client: C) -> Parent<U, C, T> {
        Parent {
            client,
            value: self.value,
            _type: PhantomData,
        }
    }
}

impl<U, C, T> Parent<U, C, T> {
    /// Returns a reference of client object.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Returns a reference of value object.
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[cfg(feature = "reqwest")]
impl Parent<types::Atom, umls_http::reqwest::Client, url::Url> {
    /// Fetch parents of UMLS atom.
    ///
    /// The method only fullfill the requirements and call specified http client.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<AtomResponse>>, Error<umls_http::reqwest::Error>> {
        umls_config
            .parent_url_validator::<types::Atom>(self.value())
            .validate()
            .map_err(Error::Validator)?;

        let url = umls_config.authenticated_url(self.value);

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Page<Content<AtomResponse>>>()
            .await
            .map_err(Error::Client)
    }
}

#[cfg(feature = "reqwest")]
impl Parent<types::Atom, umls_http::reqwest::Client, Aui> {
    /// Fetch a first page of parents of specified UMLS [`Aui`].
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<AtomResponse>>, Error<umls_http::reqwest::Error>> {
        let url = umls_config.parent_url::<types::Atom, _>(self.value).url();

        Parent::<types::Atom, _, _>::new(url)
            .with_client(self.client)
            .fetch(umls_config)
            .await
    }
}

#[cfg(feature = "reqwest")]
impl Parent<types::Atom, umls_http::reqwest::Client, Batch<Aui>> {
    /// Fetch multiple pages (one at a time) with given batch.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Response<Page<Content<AtomResponse>>, Self>, Error<umls_http::reqwest::Error>> {
        let url = umls_config.parent_url::<types::Atom, _>(&self.value).url();

        let page = Parent::<types::Atom, _, _>::new(url)
            .with_client(self.client().clone())
            .fetch(umls_config)
            .await?;

        Ok(Response::new(page, self))
    }
}

#[cfg(feature = "reqwest")]
impl
    search::Page<Content<AtomResponse>, Parent<types::Atom, umls_http::reqwest::Client, Batch<Aui>>>
{
    /// Consumes this page and returns a request for the next page.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Overflow`] if the response indicates that no next
    /// page exists or if the page number cannot be incremented.
    pub fn next(
        mut self,
    ) -> Result<Parent<types::Atom, umls_http::reqwest::Client, Batch<Aui>>, BatchError> {
        if !self.has_next() {
            return Err(BatchError::Overflow);
        }

        self.client_mut().value.checked_next()?;

        Ok(self.client)
    }

    /// Consumes this page and returns a request for the previous page.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::NoPreviousPage`] when this is the first page or
    /// the page number otherwise cannot be decremented.
    pub fn previous(
        mut self,
    ) -> Result<Parent<types::Atom, umls_http::reqwest::Client, Batch<Aui>>, BatchError> {
        if !self.has_previous() {
            return Err(BatchError::NoPreviousPage);
        }

        self.client_mut().value.checked_previous()?;

        Ok(self.client)
    }
}

#[non_exhaustive]
#[cfg(feature = "reqwest")]
/// An error object to fetch data.
#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    /// A Client error.
    #[error("Client Error: {0}")]
    Client(T),

    /// Http error.
    #[error("Http Error: {0}")]
    Http(http::Error),

    /// Failed to create a domain object.
    #[error("Invalid Domain object error.")]
    Domain,

    /// Represents uri construction failure.
    #[error("Uri construction failed: {0}")]
    Uri(#[source] http::uri::InvalidUri),

    /// Represents URL validation failure.
    #[error("URL Validation error: {0}")]
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

#[cfg(feature = "reqwest")]
impl Parent<types::Source, umls_http::reqwest::Client, url::Url> {
    /// Fetches one page of source-asserted parent concepts using the wrapped endpoint URL.
    ///
    /// The URL is validated at runtime against the base URL in `umls_config` to ensure requests
    /// are only dispatched to the authorized UMLS host and path prefix.
    ///
    /// # Errors
    /// Returns [`Error`] if:
    /// - [`Error::Http`]: The HTTP request could not be constructed.
    /// - [`Error::Client`]: The transport failed or the server returns the error status.
    /// - [`Error::Domain`]: One or more response fields failed validation when converting into a
    ///   [`SourceAtomCluster`].
    ///
    /// # Example
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use umls_client::{config::Config, parent::Parent};
    /// use umls_http::reqwest::Client;
    /// use umls_client::types::Source;
    ///
    /// let config = Config::builder("your-api-key").build()?;
    /// let url: url:: Url = "https://uts-ws.nlm.nih.gov/rest/content/current/source/SNOMEDCT_US/82065001/parents".parse()?;
    ///
    /// let page = Parent::<Source, _, _>::new(url)
    ///     .with_client(Client::default())
    ///     .fetch(&config)
    ///     .await?;
    ///
    /// for parent in page.content().list() {
    ///     println!("{}: {}", parent.ui(), parent.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<SourceAtomCluster>>, Error<umls_http::reqwest::Error>> {
        umls_config
            .parent_url_validator::<types::Source>(self.value())
            .validate()
            .map_err(Error::Validator)?;

        // Move client locally becase the value is moved from the self. Alternative for clone.
        let client = self.client;

        let url = umls_config.authenticated_url(self.value);

        let response: Page<Content<SerializableSourceAtomCluster>> = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let value = response
            .try_map::<_, EmptyString>(|x| {
                let value = x
                    .list
                    .into_iter()
                    .map(TryInto::<SourceAtomCluster>::try_into)
                    .collect::<Result<Vec<SourceAtomCluster>, _>>()?;
                Ok(Content::new(value))
            })
            .map_err(|_| Error::Domain)?;

        Ok(value)
    }
}

#[cfg(feature = "reqwest")]
impl Parent<types::Source, umls_http::reqwest::Client, SourceConcept> {
    /// Fetches one page of source-asserted parent concepts using [`SourceConcept`].
    ///
    /// # Errors
    /// Returns [`Error`] if:
    /// - [`Error::Http`]: The HTTP request could not be constructed.
    /// - [`Error::Client`]: The transport failed or the server returned an error status.
    /// - [`Error::Domain`]: One or more response fields failed validation when converting into a
    ///   [`SourceAtomCluster`].
    ///
    /// # Example
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use umls_client::{config::Config, parent::{Parent}};
    /// use umls_http::reqwest::Client;
    /// use umls_client::types::Source;
    /// use umls_client::dto::source::SourceConcept;
    ///
    /// let config = Config::builder("your-api-key").build()?;
    /// let source_concept = SourceConcept::new("SNOMEDCT_US".try_into()?, "82065001".try_into()?);
    ///
    /// let page = Parent::<Source, _, _>::new(source_concept)
    ///     .with_client(Client::default())
    ///     .fetch(&config)
    ///     .await?;
    ///
    /// for parent in page.content().list() {
    ///     println!("{}: {}", parent.ui(), parent.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<SourceAtomCluster>>, Error<umls_http::reqwest::Error>> {
        let url = umls_config.parent_url::<types::Source, _>(self.value).url();

        Parent::<types::Source, _, _>::new(url)
            .with_client(self.client)
            .fetch(umls_config)
            .await
    }
}

#[cfg(feature = "reqwest")]
impl Parent<types::Source, umls_http::reqwest::Client, Batch<SourceConcept>> {
    /// Fetches one page of source-asserted parent concepts using a batch pagination configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if:
    /// - [`Error::Http`]: The HTTP request could not be constructed.
    /// - [`Error::Client`]: The transport failed or the server returned an error status.
    /// - [`Error::Domain`]: One or more response fields failed validation when converting into a
    ///   [`SourceAtomCluster`].
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Response<Page<Content<SourceAtomCluster>>, Self>, Error<umls_http::reqwest::Error>>
    {
        let url = umls_config
            .parent_url::<types::Source, _>(&self.value)
            .url();

        let page = Parent::<types::Source, _, _>::new(url)
            .with_client(self.client().clone())
            .fetch(umls_config)
            .await?;

        Ok(Response::new(page, self))
    }
}

#[cfg(feature = "reqwest")]
impl
    search::Page<
        Content<SourceAtomCluster>,
        Parent<types::Source, umls_http::reqwest::Client, Batch<SourceConcept>>,
    >
{
    /// Consumes this page and returns a request for the next page.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Overflow`] if the response indicates that no next page exists or if
    /// the page number cannot be incremented.
    pub fn next(
        mut self,
    ) -> Result<Parent<types::Source, umls_http::reqwest::Client, Batch<SourceConcept>>, BatchError>
    {
        if !self.has_next() {
            return Err(BatchError::Overflow);
        }

        self.client_mut().value.checked_next()?;

        Ok(self.client)
    }

    /// Consumes this page and returns a request for the previous page.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::NoPreviousPage`] when this is the first page or
    /// the page number otherwise cannot be decremented.
    pub fn previous(
        mut self,
    ) -> Result<Parent<types::Source, umls_http::reqwest::Client, Batch<SourceConcept>>, BatchError>
    {
        if !self.has_previous() {
            return Err(BatchError::NoPreviousPage);
        }

        self.client_mut().value.checked_previous()?;

        Ok(self.client)
    }
}
