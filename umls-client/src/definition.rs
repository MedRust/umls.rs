use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use crate::dto::definition as dto;

use crate::core::url::validator;

#[cfg(feature = "reqwest")]
use crate::types;

#[cfg(feature = "reqwest")]
use umls_http::reqwest;

use crate::{
    core::cui::Cui,
    search::core::{DisplayName, EmptyString},
};

#[cfg(feature = "reqwest")]
use crate::config;

#[cfg(feature = "reqwest")]
use crate::{
    dto::{batch::Batch, content::Content, page::Page},
    search::Response,
};

/// Represents definition error.
#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    /// Represents client error.
    #[error("Client error: {0}")]
    Client(#[source] T),

    /// Represents url validation failure.
    #[error("URL validator error: {0}")]
    Validator(#[source] validator::Error),

    /// Represnets unsupporetd response class type.
    #[error("Unsupported class type")]
    UnsupportedClassType,
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

#[cfg(feature = "reqwest")]
impl<T> From<dto::UnsupportedClassType> for Error<T> {
    fn from(_value: dto::UnsupportedClassType) -> Self {
        Error::UnsupportedClassType
    }
}

/// Filters and target information for retrieving concept definitions.
///
/// The `Config` specifies the target concept by its [`Cui`] and an optional list of UMLS
/// source vocabularies. When `source_vocabularies` is non-empty, only definitions from those
/// vocabularies will be returned for the given [`Cui`].
#[derive(Debug, Clone)]
pub struct Config {
    cui: Cui,
    source_vocabularies: Vec<DisplayName>,
}

impl Config {
    /// Creates a new [`Builder`] for the given [`Cui`].
    pub fn builder(cui: Cui) -> Builder {
        Builder::new(cui)
    }

    /// Returns a reference to the [`Cui`] identifying the target concept.
    pub fn cui(&self) -> &Cui {
        &self.cui
    }

    /// Returns a slice of UMLS source vocabularies used to filter which definitions are returned.
    ///
    /// When non-empty, only definitions from the listed source vocabularies will be considered
    /// for the given [`Cui`]. The returned slice is read-only.
    pub fn source_vocabularies(&self) -> &[DisplayName] {
        &self.source_vocabularies
    }
}

/// A builder to build the [`Config`] object.
pub struct Builder {
    cui: Cui,
    source_vocabularies: Vec<DisplayName>,
}

impl Builder {
    /// Create and return a new [`Builder`] for given cui.
    pub fn new(cui: Cui) -> Self {
        Builder {
            cui,
            source_vocabularies: Vec::new(),
        }
    }

    /// Sets [`Cui`] for given object.
    pub fn cui(mut self, cui: Cui) -> Self {
        self.cui = cui;
        self
    }

    /// Sets list of source vocabularies.
    pub fn source_vocabularies(mut self, source_vocabularies: Vec<DisplayName>) -> Self {
        self.source_vocabularies = source_vocabularies;
        self
    }

    /// Adds a source vocabularies.
    pub fn try_add_source_vocabulary(
        mut self,
        source_vocabulary: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.source_vocabularies.push(source_vocabulary.try_into()?);
        Ok(self)
    }

    /// Create and return a [`Config`] object.
    pub fn build(self) -> Config {
        Config {
            cui: self.cui,
            source_vocabularies: self.source_vocabularies,
        }
    }
}

/// A UMLS concept definition client.
pub struct Definition<C, V> {
    client: C,
    value: V,
}

impl<C, V> Definition<C, V> {
    /// Return a read-only reference to client.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Returns a reference to the value.
    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<V> Definition<WithoutClient, V> {
    /// Create a new [`Definition`] client for give value.
    ///
    /// The [`Definition`] client is created using [`WithoutClient`].
    pub fn new(value: V) -> Definition<WithoutClient, V> {
        Definition {
            client: WithoutClient,
            value,
        }
    }

    /// Create a [`Definition`] object with different client.
    pub fn with_client<C>(self, client: C) -> Definition<C, V> {
        Definition {
            client,
            value: self.value,
        }
    }
}

#[cfg(feature = "reqwest")]
impl Definition<reqwest::Client, url::Url> {
    /// Fetch concepts definitions for given URL.
    ///
    /// The method is responsible to validate, authenticate url, send request and processing of
    /// response.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<dto::Definition>>, Error<umls_http::reqwest::Error>> {
        umls_config
            .definition_url_validator::<types::Concept>(self.value())
            .validate()?;

        let url = umls_config.authenticated_url(self.value);

        let body = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Page<Content<dto::RawDefinition>>>()
            .await?;

        let page = body.try_map::<_, Error<umls_http::reqwest::Error>>(|x| {
            let pages = x
                .list
                .into_iter()
                .map(|x| x.try_into())
                .collect::<Result<Vec<dto::Definition>, _>>()?;
            Ok(Content::new(pages))
        })?;

        Ok(page)
    }
}

#[cfg(feature = "reqwest")]
impl Definition<reqwest::Client, Batch<Config>> {
    /// Fetch concept definitions for respective value.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<
        Response<Page<Content<dto::Definition>>, Definition<reqwest::Client, Batch<Config>>>,
        Error<umls_http::reqwest::Error>,
    > {
        let mut vector = [
            ("pageSize", self.value().size().get().to_string()),
            ("pageNumber", self.value().number().get().to_string()),
        ]
        .to_vec();

        if !self.value().value().source_vocabularies().is_empty() {
            let values: Vec<&str> = self
                .value()
                .value()
                .source_vocabularies()
                .iter()
                .map(|x| x.as_str())
                .collect();
            let values = values.join(",");
            vector.push(("sabs", values));
        }

        let url = umls_config
            .definition_url::<types::Concept, _>(&self.value)
            .url();

        let page = Definition::new(url)
            .with_client(self.client.clone())
            .fetch(umls_config)
            .await?;

        Ok(Response::new(page, self))
    }
}

#[cfg(feature = "reqwest")]
impl Definition<reqwest::Client, Config> {
    /// Fetch concept definitions for respective value.
    ///
    /// The function uses DEFAULT_PAGE_SIZE for page size and DEFAULT_PAGE_NUMBER for page number.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Content<dto::Definition>, Error<umls_http::reqwest::Error>> {
        let url = umls_config
            .definition_url::<types::Concept, _>(&self.value)
            .url();

        let page = Definition::new(url)
            .with_client(self.client)
            .fetch(umls_config)
            .await?;

        Ok(page.content)
    }
}
