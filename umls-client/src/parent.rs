use std::marker::PhantomData;

#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;
#[cfg(feature = "reqwest")]
use url::Url;

#[cfg(feature = "reqwest")]
use crate::core::aui::Aui;

#[cfg(feature = "reqwest")]
use crate::{
    atom::{Batch, BatchError},
    search::{self, Response},
};

#[cfg(feature = "reqwest")]
use crate::{
    atom::{AtomResponse, Page},
    config,
    search::Content,
};

/// The parent types.
pub mod types {
    /// Represents the Atom parent types.
    ///
    /// It used as state for [`super::Parent`] to fetch immediate parents of UMLS atom.
    pub struct Atom;
}

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
impl Parent<types::Atom, umls_http::reqwest::Client, Url> {
    /// Fetch parents of UMLS atom.
    ///
    /// The method only fullfill the requirements and call specified http client.
    /// The function will raise [`Error::Validation`] if sepcified URL validation failed against the
    /// base URL of specified UMLS configuration.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<AtomResponse>>, Error<umls_http::reqwest::Error>> {
        if self.value().scheme() != umls_config.base_url().scheme()
            || self.value().host_str() != umls_config.base_url().host_str()
            || self.value().port_or_known_default()
                != umls_config.base_url().port_or_known_default()
            || !self.value().username().is_empty()
            || self.value().password().is_some()
            || !(self.value().path() == umls_config.base_url().path().trim_end_matches('/')
                || self.value().path().starts_with(&format!(
                    "{}/",
                    umls_config.base_url().path().trim_end_matches('/')
                )))
        {
            return Err(Error::Validation(self.value().clone()));
        }

        let url = Url::parse_with_params(
            self.value().as_str(),
            &[("apiKey", umls_config.api_key().expose_secret().to_owned())],
        )
        .map_err(Error::Url)?;

        let request = http::Request::builder()
            .method("GET")
            .uri(url.as_str())
            .body(())
            .map_err(Error::Http)?;

        let response: http::Response<Page<Content<AtomResponse>>> = self
            .client()
            .execute(request)
            .await
            .map_err(Error::Client)?;

        Ok(response.into_body())
    }
}

#[cfg(feature = "reqwest")]
impl Parent<types::Atom, umls_http::reqwest::Client, Aui> {
    /// Fetch a first page of parents of specified UMLS [`Aui`].
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<AtomResponse>>, Error<umls_http::reqwest::Error>> {
        let path = format!(
            "content/{}/AUI/{}/parents",
            umls_config.version(),
            self.value()
        );
        let url = umls_config.base_url().join(&path).map_err(Error::Url)?;

        Parent::new(url)
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
        let path = format!(
            "content/{}/AUI/{}/parents",
            umls_config.version(),
            self.value().value()
        );
        let url = umls_config.base_url().join(&path).map_err(Error::Url)?;
        let url = Url::parse_with_params(
            url.as_str(),
            &[
                ("pageSize", self.value().size().get().to_string()),
                ("pageNumber", self.value().number().get().to_string()),
            ],
        )
        .map_err(Error::Url)?;

        let page = Parent::new(url)
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

#[cfg(feature = "reqwest")]
/// An error object to fetch data.
#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    /// A Client error.
    #[error("Client Error: {0}")]
    Client(T),

    /// Url Error.
    #[error("Url Error: {0}")]
    Url(url::ParseError),

    /// Http error.
    #[error("Http Error: {0}")]
    Http(http::Error),

    /// Represents URL validation error.
    #[error("URL Validation Error")]
    Validation(Url),
}
