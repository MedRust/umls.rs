/// A UMLS concept relationship configuration module.
#[cfg(feature = "reqwest")]
pub mod config;

use umls_http::WithoutClient;

use crate::core::url::validator;

#[cfg(feature = "reqwest")]
use crate::types;

#[cfg(feature = "reqwest")]
use crate::dto::relation::RelationDto;

#[cfg(feature = "reqwest")]
use umls_http::reqwest::Client;

#[cfg(feature = "reqwest")]
use crate::{
    dto::{
        batch::{Batch, BatchError},
        content::Content,
        page::Page,
    },
    search::Response,
};

/// Represents relation error.
#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    /// Represents client error.
    #[error("Client error: {0}")]
    Client(#[source] T),

    /// Represents url validation failure.
    #[error("URL validation error: {0}")]
    Validator(validator::Error),
}

#[cfg(feature = "reqwest")]
impl From<umls_http::reqwest::Error> for Error<umls_http::reqwest::Error> {
    fn from(value: umls_http::reqwest::Error) -> Self {
        Error::Client(value)
    }
}

impl<T> From<validator::Error> for Error<T> {
    fn from(value: validator::Error) -> Self {
        Error::Validator(value)
    }
}

/// Represents Result of relation.
#[cfg(feature = "reqwest")]
pub type RelationResult<T> = std::result::Result<T, Error<umls_http::reqwest::Error>>;

/// A UMLS concept relation client.
pub struct Relation<C, V> {
    client: C,
    value: V,
}

impl<C, V> Relation<C, V> {
    /// Return a reference of client.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Return a reference of value.
    pub fn value(&self) -> &V {
        &self.value
    }

    #[cfg(feature = "reqwest")]
    fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

impl<V> Relation<WithoutClient, V> {
    /// Create a new [`Relation`] client for given value.
    ///
    /// The [`Relation`] client is created suing [`WithoutClient`].
    pub fn new(value: V) -> Self {
        Self {
            client: WithoutClient,
            value,
        }
    }

    /// Create a [`Relation`] object with different client.
    pub fn with_client<C>(self, client: C) -> Relation<C, V> {
        Relation {
            client,
            value: self.value,
        }
    }
}

#[cfg(feature = "reqwest")]
impl Relation<Client, config::Config> {
    /// Fetch concept relationships.
    pub async fn fetch(
        self,
        umls_config: &crate::config::Config,
    ) -> RelationResult<Content<RelationDto>> {
        let url = umls_config
            .relation_url::<types::Concept, _>(self.value())
            .url();

        let response = Relation::new(url)
            .with_client(self.client)
            .fetch(umls_config)
            .await?;

        Ok(response.content)
    }
}

#[cfg(feature = "reqwest")]
impl Relation<Client, Batch<config::Config>> {
    /// Fetch concept relationships for configuration.
    ///
    /// The function fetches the concept realtionship for specified page size and page number. The function
    pub async fn fetch(
        self,
        umls_config: &crate::config::Config,
    ) -> RelationResult<Response<Page<Content<RelationDto>>, Self>> {
        let url = umls_config
            .relation_url::<types::Concept, _>(&self.value)
            .url();

        let page = Relation::new(url)
            .with_client(self.client().clone())
            .fetch(umls_config)
            .await?;

        Ok(Response::new(page, self))
    }
}

#[cfg(feature = "reqwest")]
impl Relation<Client, url::Url> {
    /// Fetch concept relations for given url.
    ///
    /// The method is responsible for url validation, authentication and sending request and
    /// processing the response.
    pub async fn fetch(
        self,
        umls_config: &crate::config::Config,
    ) -> RelationResult<Page<Content<RelationDto>>> {
        umls_config
            .relation_url_validator(self.value())
            .validate()?;

        let client = self.client;
        let url = umls_config.authenticated_url(self.value);

        let response = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Page<Content<RelationDto>>>()
            .await?;

        Ok(response)
    }
}

#[cfg(feature = "reqwest")]
impl crate::search::Page<Content<RelationDto>, Relation<Client, Batch<config::Config>>> {
    /// Validate and return a prepared [`Relation`] client to fetch next page.
    ///
    /// The function may return [`BatchError::Overflow`] if there is not next page or the page
    /// number value is more then the [`u64::MAX`].
    pub fn next(mut self) -> Result<Relation<Client, Batch<config::Config>>, BatchError> {
        if !self.has_next() {
            return Err(BatchError::Overflow);
        }

        self.client_mut().value_mut().checked_next()?;

        Ok(self.client)
    }

    /// Validate the return a prepared [`Relation`] client to fetch previous page.
    ///
    /// The function may return [`BatchError::NoPreviousPage`] if there is not previous page or try
    /// to set page number less then 1.
    pub fn previous(mut self) -> Result<Relation<Client, Batch<config::Config>>, BatchError> {
        if !self.has_previous() {
            return Err(BatchError::NoPreviousPage);
        }

        self.client_mut().value_mut().checked_previous()?;

        Ok(self.client)
    }
}
