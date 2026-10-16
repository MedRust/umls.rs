use serde::{Deserialize, Serialize};
use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use umls_http::reqwest;

use crate::{
    concept::cui::Cui,
    search::core::{DisplayName, EmptyString},
};

#[cfg(feature = "reqwest")]
use crate::{atom::Batch, config};

#[cfg(feature = "reqwest")]
use crate::{
    atom::Page,
    search::{self, Content, Response},
};

/// Filters and target information for retrieving concept definitions.
///
/// The `Config` specifies the target concept by its [`Cui`] and an optional list of UMLS
/// source vocabularies. When `source_vocabularies` is non-empty, only definitions from those
/// vocabularies will be returned for the given [`Cui`].
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
impl Definition<reqwest::Client, Batch<Config>> {
    /// Fetch concept definitions for respective value.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<
        Response<Page<Content<DefinitionDto>>, Definition<reqwest::Client, Batch<Config>>>,
        Box<dyn std::error::Error>,
    > {
        use secrecy::ExposeSecret;
        use url::Url;

        let cui = self.value().value().cui();
        let path = format!("/content/{}/CUI/{}/definitions", umls_config.version(), cui);
        let url_string = umls_config.base_url().join(&path)?.as_str().to_owned();

        let mut vector = [
            ("apiKey", umls_config.api_key().expose_secret().into()),
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

        let url = Url::parse_with_params(&url_string, &vector)?;

        let request = http::Request::builder().uri(url.as_str()).body(())?;

        let response: http::Response<Page<Content<RawDefinition>>> =
            self.client().execute(request).await?;

        let page = response
            .into_body()
            .try_map::<_, Box<dyn std::error::Error>>(|x| {
                let list = x
                    .list
                    .into_iter()
                    .map(|x| x.try_into_definition_dto(cui.clone()))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Content::new(list))
            })?;

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
    ) -> Result<Content<DefinitionDto>, Box<dyn std::error::Error>> {
        use secrecy::ExposeSecret;
        use url::Url;

        let cui = self.value().cui();
        let path = format!("/content/{}/CUI/{}/definitions", umls_config.version(), cui);
        let url_string = umls_config.base_url().join(&path)?.as_str().to_owned();

        let mut vector = [
            ("apiKey", umls_config.api_key().expose_secret().into()),
            ("pageSize", search::core::DEFAULT_PAGE_SIZE.to_string()),
            ("pageNumber", search::core::DEFAULT_PAGE_NUMBER.to_string()),
        ]
        .to_vec();

        if !self.value().source_vocabularies().is_empty() {
            let values: Vec<&str> = self
                .value()
                .source_vocabularies()
                .iter()
                .map(|x| x.as_str())
                .collect();
            let values = values.join(",");
            vector.push(("sabs", values));
        }

        let url = Url::parse_with_params(&url_string, &vector)?;

        let request = http::Request::builder()
            .uri(url.as_str())
            .header("apiKey", umls_config.api_key().expose_secret())
            .body(())?;

        let response: http::Response<Page<Content<RawDefinition>>> =
            self.client().execute(request).await?;

        let page = response
            .into_body()
            .try_map::<_, Box<dyn std::error::Error>>(|x| {
                let list = x
                    .list
                    .into_iter()
                    .map(|x| x.try_into_definition_dto(cui.clone()))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Content::new(list))
            })?;

        Ok(page.content)
    }
}

/// Represents possible the de/serializable class type.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum ClassType {
    /// A difinition class type.
    Definition,
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Deserialize, Clone)]
struct RawDefinition {
    #[serde(rename = "classType")]
    class_type: ClassType,

    #[serde(rename = "sourceOriginated")]
    source_originated: bool,

    #[serde(rename = "rootSource")]
    root_source: DisplayName,

    value: DisplayName,
}

/// A object that encapsulates the definition and metadata.
#[derive(Debug)]
pub struct DefinitionDto {
    cui: Cui,
    is_source_originated: bool,
    root_source: DisplayName,
    value: DisplayName,
}

impl DefinitionDto {
    /// Return read-only reference to [`Cui`]
    pub fn cui(&self) -> &Cui {
        &self.cui
    }

    /// Return if the definition is source originated.
    pub fn is_source_originated(&self) -> bool {
        self.is_source_originated
    }

    /// Returns a source vocabularie where the definition is oriented from.
    pub fn root_source(&self) -> &DisplayName {
        &self.root_source
    }

    /// Returns the definition.
    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
}

#[cfg(feature = "reqwest")]
impl RawDefinition {
    fn try_into_definition_dto(
        self,
        cui: Cui,
    ) -> Result<DefinitionDto, Box<dyn std::error::Error>> {
        if self.class_type != ClassType::Definition {
            panic!();
        }

        Ok(DefinitionDto {
            cui,
            is_source_originated: self.source_originated,
            value: self.value,
            root_source: self.root_source,
        })
    }
}
