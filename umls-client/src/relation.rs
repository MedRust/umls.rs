use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use serde::{Deserialize, Deserializer};

#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;

#[cfg(feature = "reqwest")]
use umls_http::reqwest::Client;
#[cfg(feature = "reqwest")]
use url::Url;

#[cfg(feature = "reqwest")]
use crate::{
    atom::{Batch, BatchError, Page},
    search::{Content, Response},
};

/// A UMLS concept relationship configuration module.
#[cfg(feature = "reqwest")]
pub mod config {
    use std::collections::HashSet;

    use crate::{
        core::cui::Cui,
        search::core::{DisplayName, EmptyString},
    };

    /// A relationship configuration.
    ///
    /// The object is created using [`Builder`].
    ///
    /// # Example
    /// ```
    /// use umls_client::relation::config::Config;
    /// use umls_client::core::cui::Cui;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cui: Cui = "C0010200".parse()?;
    /// let config = Config::builder(cui.clone()).build();
    /// assert_eq!(config.cui(), &cui);
    /// assert!(config.labels().is_empty());
    /// assert!(config.additional_labels().is_empty());
    /// assert!(!config.include_obsolete());
    /// assert!(!config.include_suppressible());
    /// assert!(config.source_vocabularies().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub struct Config {
        cui: Cui,
        labels: Vec<DisplayName>,
        additional_labels: Vec<DisplayName>,
        include_obsolete: bool,
        include_suppressible: bool,
        source_vocabularies: Vec<DisplayName>,
    }

    impl Config {
        /// Create and return a [`Builder`] object for given [`Cui`].
        ///
        /// # Example
        /// ```
        /// use umls_client::relation::config::Config;
        /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
        /// let cui = "C0010200".parse()?;
        /// let config = Config::builder(cui).build();
        /// # Ok(())
        /// # }
        /// ```
        pub fn builder(cui: Cui) -> Builder {
            Builder::new(cui)
        }

        /// Returns a reference of [`Cui`].
        pub fn cui(&self) -> &Cui {
            &self.cui
        }

        /// Returns a reference of relationship labels.
        pub fn labels(&self) -> &[DisplayName] {
            &self.labels
        }

        /// Returns reference of additional relationship labels.
        pub fn additional_labels(&self) -> &[DisplayName] {
            &self.additional_labels
        }

        /// return if the response contains obsolete data.
        pub fn include_obsolete(&self) -> bool {
            self.include_obsolete
        }

        /// Return if the response contains suppressible result.
        pub fn include_suppressible(&self) -> bool {
            self.include_suppressible
        }

        /// Returns reference of source vocabularies.
        pub fn source_vocabularies(&self) -> &[DisplayName] {
            &self.source_vocabularies
        }
    }

    /// A object builder to create [`Config`] object.
    ///
    /// The structure uses builder pattern with mathod chaining to give each of access.
    ///
    /// # Example
    /// ```
    /// use umls_client::relation::config::Builder;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let cui = "C0010200".parse()?;
    /// let builder = Builder::new(cui);
    /// # Ok(())
    /// # }
    /// ```
    pub struct Builder {
        cui: Cui,
        labels: Vec<DisplayName>,
        additional_labels: Vec<DisplayName>,
        include_obsolete: bool,
        include_suppressible: bool,
        source_vocabularies: Vec<DisplayName>,
    }

    impl Builder {
        /// Create a new instance of [`Builder`] for given CUI.
        ///
        /// The builder is created with empty relationship labels, additional relationship labels,
        /// source vocabularies and excluding the obsolete and is_suppressible concepts.
        pub fn new(cui: Cui) -> Self {
            Self {
                cui,
                labels: Vec::new(),
                additional_labels: Vec::new(),
                include_obsolete: false,
                include_suppressible: false,
                source_vocabularies: Vec::new(),
            }
        }

        /// Sets the relationship labels where response relationship belongs to.
        pub fn labels(mut self, labels: Vec<DisplayName>) -> Self {
            self.labels = labels;
            self
        }

        /// Add the relationship label where relationship response belongs to.
        pub fn try_add_label(
            mut self,
            label: impl TryInto<DisplayName, Error = EmptyString>,
        ) -> Result<Self, EmptyString> {
            self.labels.push(label.try_into()?);
            Ok(self)
        }

        /// sets the additional relationship labels where relationship response belongs to.
        pub fn additional_labels(mut self, additional_labels: Vec<DisplayName>) -> Self {
            self.additional_labels = additional_labels;
            self
        }

        /// Add the additional relationship label where relationship response belongs to.
        pub fn try_add_additional_label(
            mut self,
            additional_label: impl TryInto<DisplayName, Error = EmptyString>,
        ) -> Result<Self, EmptyString> {
            self.additional_labels.push(additional_label.try_into()?);
            Ok(self)
        }

        /// set to include obsolete relationships for respective CUI.
        pub fn include_obsolete(mut self) -> Self {
            self.include_obsolete = true;
            self
        }

        /// Include suppressible concept relationships.
        pub fn include_suppressible(mut self) -> Self {
            self.include_suppressible = true;
            self
        }

        /// Sets source vocabularies where relationships are belongs to.
        pub fn source_vocabularies(mut self, source_vocabularies: Vec<DisplayName>) -> Self {
            self.source_vocabularies = source_vocabularies;
            self
        }

        /// Add source vocabulary where relationships are belongs to.
        pub fn try_add_source_vocabulary(
            mut self,
            source_vocabulary: impl TryInto<DisplayName, Error = EmptyString>,
        ) -> Result<Self, EmptyString> {
            self.source_vocabularies.push(source_vocabulary.try_into()?);
            Ok(self)
        }

        /// Build [`Config`] object.
        pub fn build(self) -> Config {
            let labels = self
                .labels
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let additional_labels = self
                .additional_labels
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            let source_vocabularies = self
                .source_vocabularies
                .into_iter()
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            Config {
                cui: self.cui,
                labels,
                additional_labels,
                include_obsolete: self.include_obsolete,
                include_suppressible: self.include_suppressible,
                source_vocabularies,
            }
        }
    }
}

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
    ) -> Result<Content<RelationDto>, Box<dyn std::error::Error>> {
        let path = format!(
            "/content/{}/CUI/{}/relations",
            umls_config.version(),
            self.value().cui()
        );
        let url = umls_config.base_url().join(&path)?;

        let labels = self
            .value()
            .labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let additional_lables = self
            .value()
            .additional_labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let source_vocabularies = self
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let url = Url::parse_with_params(
            url.as_str(),
            &[
                ("includeRelationLabels", labels),
                ("includeAdditionalRelationLabels", additional_lables),
                ("sabs", source_vocabularies),
                (
                    "includeObsolete",
                    self.value().include_obsolete().to_string(),
                ),
                (
                    "includeSuppressible",
                    self.value().include_suppressible().to_string(),
                ),
            ],
        )?;

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
    ) -> Result<Response<Page<Content<RelationDto>>, Self>, Box<dyn std::error::Error>> {
        let path = format!(
            "/content/{}/CUI/{}/relations",
            umls_config.version(),
            self.value().value().cui()
        );
        let url = umls_config.base_url().join(&path)?;

        let labels = self
            .value()
            .value()
            .labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let additional_lables = self
            .value()
            .value()
            .additional_labels()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let source_vocabularies = self
            .value()
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        let url = Url::parse_with_params(
            url.as_str(),
            &[
                ("includeRelationLabels", labels),
                ("includeAdditionalRelationLabels", additional_lables),
                ("sabs", source_vocabularies),
                (
                    "includeObsolete",
                    self.value().value().include_obsolete().to_string(),
                ),
                (
                    "includeSuppressible",
                    self.value().value().include_suppressible().to_string(),
                ),
                ("pageNumber", self.value().number().get().to_string()),
                ("pageSize", self.value().size().get().to_string()),
            ],
        )?;

        let page = Relation::new(url)
            .with_client(self.client().clone())
            .fetch(umls_config)
            .await?;

        Ok(Response::new(page, self))
    }
}

#[cfg(feature = "reqwest")]
impl Relation<Client, Url> {
    /// Fetch concept relations for given url.
    ///
    /// The method doesn't validate the URL.
    pub async fn fetch(
        self,
        umls_config: &crate::config::Config,
    ) -> Result<Page<Content<RelationDto>>, Box<dyn std::error::Error>> {
        let url = Url::parse_with_params(
            self.value().as_str(),
            &[("apiKey", umls_config.api_key().expose_secret().to_string())],
        )?;

        let request = http::Request::builder()
            .uri(url.as_str())
            .method("GET")
            .body(())?;

        let response: http::Response<Page<Content<RelationDto>>> =
            self.client().execute(request).await?;

        let content = response.into_body();

        Ok(content)
    }
}

#[cfg(feature = "reqwest")]
fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    Ok(if value == "NONE" { None } else { Some(value) })
}

/// A relationship DTO object.
#[cfg(feature = "reqwest")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationDto {
    /// UMLS Relationship Unique Identifier (RUI)
    pub ui: String,
    /// Represents respective object is suppressible.
    pub suppressible: bool,
    /// The source unique identifier if present.
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub source_ui: Option<String>,
    /// Represents respective object is obsolete.
    pub obsolete: bool,
    /// Represents respective object is source originated.
    pub source_originated: bool,
    /// The root source vocabularies.
    pub root_source: String,
    /// The grou id if present.
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub group_id: Option<String>,
    /// The attribute count.
    pub attribute_count: u32,
    /// The class type.
    pub class_type: String,
    /// From Id
    pub related_from_id: String,
    /// From name
    pub related_from_id_name: String,
    /// The relationship label.
    pub relation_label: String,
    /// The additional relationship label.
    #[serde(deserialize_with = "deserialize_optional_string")]
    pub additional_relation_label: Option<String>,
    /// The related id.
    pub related_id: String,
    /// The related id name
    pub related_id_name: String,
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
