#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;
use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use umls_http::reqwest;
#[cfg(feature = "reqwest")]
use url::Url;

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
impl Definition<reqwest::Client, Url> {
    /// Fetch concepts definitions for given URL.
    ///
    /// The method doesn't validate the URL.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<dto::Definition>>, Box<dyn std::error::Error>> {
        let url = Url::parse_with_params(
            self.value().as_str(),
            &[("apiKey", umls_config.api_key().expose_secret().to_string())],
        )?;

        let request = http::Request::builder().uri(url.as_str()).body(())?;

        let response: http::Response<Page<Content<dto::RawDefinition>>> =
            self.client().execute(request).await?;

        let page = response
            .into_body()
            .try_map::<_, Box<dyn std::error::Error>>(|x| {
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
        Box<dyn std::error::Error>,
    > {
        let cui = self.value().value().cui();
        let path = format!("/content/{}/CUI/{}/definitions", umls_config.version(), cui);
        let url_string = umls_config.base_url().join(&path)?.as_str().to_owned();

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

        let url = Url::parse_with_params(&url_string, &vector)?;

        let page = Definition::new(url)
            .with_client(self.client().clone())
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
    ) -> Result<Content<dto::Definition>, Box<dyn std::error::Error>> {
        let cui = self.value().cui();
        let path = format!("/content/{}/CUI/{}/definitions", umls_config.version(), cui);
        let url_string = umls_config.base_url().join(&path)?.as_str().to_owned();

        let mut vector = [
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

        let page = Definition::new(url)
            .with_client(self.client().clone())
            .fetch(umls_config)
            .await?;

        Ok(page.content)
    }
}

/// Encapsulates the definition DTO symbols.
#[cfg(feature = "reqwest")]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use crate::search::core::DisplayName;

    /// Represents possible the de/serializable class type.
    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
    pub enum ClassType {
        /// A difinition class type.
        Definition,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub(crate) struct RawDefinition {
        #[serde(rename = "classType")]
        class_type: ClassType,

        #[serde(rename = "sourceOriginated")]
        source_originated: bool,

        #[serde(rename = "rootSource")]
        root_source: DisplayName,

        value: DisplayName,
    }

    /// The metadata of the respecitve definition.
    pub struct MetaData {
        source_originated: bool,
        root_source: DisplayName,
    }

    impl MetaData {
        fn new(source_originated: bool, root_source: DisplayName) -> Self {
            Self {
                source_originated,
                root_source,
            }
        }

        /// Returns the definition is source originated or not.
        pub fn is_source_originated(&self) -> bool {
            self.source_originated
        }

        /// Returns the root source vocabulary of the definition.
        pub fn root_source(&self) -> &DisplayName {
            &self.root_source
        }
    }

    /// The definition object.
    pub struct Definition {
        value: DisplayName,
        metadata: MetaData,
    }

    impl Definition {
        fn new(value: DisplayName, metadata: MetaData) -> Self {
            Self { value, metadata }
        }

        /// Returns human readable representation of definition.
        pub fn value(&self) -> &DisplayName {
            &self.value
        }

        /// Returns the reference to metadata.
        pub fn metadata(&self) -> &MetaData {
            &self.metadata
        }
    }

    impl TryFrom<RawDefinition> for Definition {
        type Error = Box<dyn std::error::Error>;

        fn try_from(value: RawDefinition) -> Result<Self, Self::Error> {
            if value.class_type != ClassType::Definition {
                panic!();
            }

            let metadata = MetaData::new(value.source_originated, value.root_source);
            let definition = Definition::new(value.value, metadata);

            Ok(definition)
        }
    }
}
