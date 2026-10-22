#[cfg(feature = "reqwest")]
use http::Response;
#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;

use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use umls_http::reqwest;

#[cfg(feature = "reqwest")]
use url::Url;

#[cfg(feature = "reqwest")]
use serde::Deserialize;

#[cfg(feature = "reqwest")]
use crate::atom::{AtomResponse, Page};
#[cfg(feature = "reqwest")]
use crate::core::aui::Aui;
#[cfg(feature = "reqwest")]
use crate::search::core::get_trimmed_non_empty_string;
#[cfg(feature = "reqwest")]
use crate::{atom, definition, relation};

#[cfg(feature = "reqwest")]
use crate::{config::Config, core::cui::Cui, search::ConceptSnapshot};

#[cfg(feature = "reqwest")]
#[derive(Debug, thiserror::Error)]
/// An error produced while locating or retrieving a concept.
pub enum Error<T> {
    /// A client error.
    #[error("Client error: {0}")]
    Client(#[source] T),

    /// A concept endpoint URL could not be constructed.
    #[error("Url error: {0}")]
    Url(url::ParseError),
}

/// A typed reference to information that can identify a UMLS concept.
///
/// The available `fetch` method depends on whether the value is a [`crate::core::cui::Cui`], a
/// [`url::Url`], or a [`crate::search::ConceptSnapshot`]. Attach a `reqwest::Client` using
/// [`Self::with_client`] before fetching; fetching consumes the request.
pub struct Info<C, T> {
    client: C,
    value: T,
}

impl<C, T> Info<C, T> {
    /// Returns the wrapped reference value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the wraper reference client.
    pub fn client(&self) -> &C {
        &self.client
    }
}

impl<T> Info<WithoutClient, T> {
    /// Wraps a concept reference value.
    pub fn new(value: T) -> Self {
        Self {
            client: WithoutClient,
            value,
        }
    }

    /// Attaches an owned HTTP client without sending a request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// use umls_http::reqwest;
    /// use umls_client::{concept::{Info}, core::cui::Cui, config::Config};
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
    pub fn with_client<C>(self, client: C) -> Info<C, T> {
        Info {
            client,
            value: self.value,
        }
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, Cui> {
    /// Retrieves the concept identified by this CUI.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if endpoint construction, transport, HTTP status
    /// validation, or response decoding fails.
    pub async fn fetch(self, config: &Config) -> Result<Concept, Error<reqwest::Error>> {
        let path = format!("content/{}/CUI/{}", config.version(), self.value.as_str());
        let url = config.base_url().join(&path).map_err(Error::Url)?;

        Info::new(url).with_client(self.client).fetch(config).await
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, Url> {
    /// Retrieves a concept from the wrapped UMLS resource URL.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if URL construction, transport, HTTP status
    /// validation, or response decoding fails.
    pub async fn fetch(self, config: &Config) -> Result<Concept, Error<reqwest::Error>> {
        let url = Url::parse_with_params(
            self.value.as_str(),
            &[("apiKey", config.api_key().expose_secret())],
        )
        .map_err(Error::Url)?;

        let request = http::Request::builder()
            .uri(url.as_str())
            .body(())
            .map_err(reqwest::Error::Http)
            .map_err(Error::Client)?;

        let response: Response<ConceptResponse> =
            self.client.execute(request).await.map_err(Error::Client)?;

        Ok(response.into_body().result.into())
    }
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, ConceptSnapshot> {
    /// Retrieves the full concept represented by a search-result snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the concept request cannot be completed or decoded.
    pub async fn fetch(self, config: &Config) -> Result<Concept, Error<reqwest::Error>> {
        self.value.info.with_client(self.client).fetch(config).await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "reqwest")]
struct ConceptResponse {
    result: SerializableConcept,
}

/// A full concept returned by the UMLS content endpoint.
#[cfg(feature = "reqwest")]
pub struct Concept {
    /// The response object's UMLS class discriminator.
    pub class_type: ConceptClass,
    /// The concept identifier returned by UMLS.
    pub ui: String,
    /// Whether the concept is suppressible.
    pub suppressible: bool,
    /// Date on which the concept was added, in the API's textual format.
    pub date_added: String,
    /// Date of the concept's latest major revision.
    pub major_revision_date: String,
    /// Current concept status reported by UMLS.
    pub status: String,
    /// Semantic types assigned to the concept.
    pub semantic_types: Vec<SemanticType>,
    /// Number of atoms associated with the concept.
    pub atom_count: u64,
    /// Number of attributes associated with the concept.
    pub attribute_count: u64,
    /// Number of content-view memberships associated with the concept.
    pub cv_member_count: u64,
    /// Endpoint for the concept's atoms, when available.
    pub atoms: atom::Atom<umls_http::WithoutClient, atom::All, Url>,
    /// Endpoint for the concept's definitions, when available.
    pub definitions: Option<definition::Definition<umls_http::WithoutClient, Url>>,
    /// Endpoint for the concept's relations, when available.
    pub relations: Option<relation::Relation<umls_http::WithoutClient, Url>>,
    /// Endpoint for the default preferred atom, when available.
    pub default_preferred_atom: atom::Atom<umls_http::WithoutClient, atom::Preferred, Url>,
    /// Number of relations associated with the concept.
    pub relation_count: u64,
    /// Human-readable preferred name of the concept.
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "reqwest")]
#[allow(unused)]
/// A full concept returned by the UMLS content endpoint.
struct SerializableConcept {
    /// The response object's UMLS class discriminator.
    class_type: ConceptClass,
    /// The concept identifier returned by UMLS.
    ui: String,
    /// Whether the concept is suppressible.
    suppressible: bool,
    /// Date on which the concept was added, in the API's textual format.
    date_added: String,
    /// Date of the concept's latest major revision.
    major_revision_date: String,
    /// Current concept status reported by UMLS.
    status: String,
    /// Semantic types assigned to the concept.
    semantic_types: Vec<SemanticType>,
    /// Number of atoms associated with the concept.
    atom_count: u64,
    /// Number of attributes associated with the concept.
    attribute_count: u64,
    /// Number of content-view memberships associated with the concept.
    cv_member_count: u64,
    /// Endpoint for the concept's atoms, when available.
    atoms: Url,

    /// Endpoint for the concept's definitions, when available.
    definitions: OptionalResource,

    /// Endpoint for the concept's relations, when available.
    relations: OptionalResource,
    /// Endpoint for the default preferred atom, when available.
    default_preferred_atom: Url,
    /// Number of relations associated with the concept.
    relation_count: u64,
    /// Human-readable preferred name of the concept.
    name: String,
}

#[cfg(feature = "reqwest")]
impl From<SerializableConcept> for Concept {
    fn from(value: SerializableConcept) -> Self {
        let definitions = match value.definitions {
            OptionalResource::Available(url) => Some(definition::Definition::new(url)),
            OptionalResource::Unavailable => None,
        };

        let relations = match value.relations {
            OptionalResource::Available(url) => Some(relation::Relation::new(url)),
            OptionalResource::Unavailable => None,
        };

        let atoms = atom::Atom::new(value.atoms);

        let default_preferred_atom = atom::Atom::new(value.default_preferred_atom);

        Concept {
            class_type: value.class_type,
            ui: value.ui,
            suppressible: value.suppressible,
            date_added: value.date_added,
            major_revision_date: value.major_revision_date,
            status: value.status,
            semantic_types: value.semantic_types,
            atom_count: value.atom_count,
            attribute_count: value.attribute_count,
            cv_member_count: value.cv_member_count,
            definitions,
            relations,
            relation_count: value.relation_count,
            name: value.name,
            atoms,
            default_preferred_atom,
        }
    }
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Clone, PartialEq, Eq)]
/// A resource link that the UMLS API may represent with the `NONE` sentinel.
pub enum OptionalResource {
    /// The resource is available at the contained URL.
    Available(Url),
    /// The API reported that the resource is unavailable.
    Unavailable,
}

#[cfg(feature = "reqwest")]
impl<'de> Deserialize<'de> for OptionalResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let trimmed = get_trimmed_non_empty_string(&string).map_err(serde::de::Error::custom)?;
        if trimmed == "NONE" {
            return Ok(OptionalResource::Unavailable);
        }

        let url: Url = trimmed.parse().map_err(serde::de::Error::custom)?;
        Ok(OptionalResource::Available(url))
    }
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Deserialize)]
/// The UMLS class discriminator for a concept response.
pub enum ConceptClass {
    /// A UMLS concept resource.
    Concept,
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Deserialize)]
/// A semantic type assigned to a concept.
pub struct SemanticType {
    /// Human-readable semantic-type name.
    pub name: String,
    /// UMLS resource URL for the semantic type.
    pub uri: OptionalResource,
}

#[cfg(feature = "reqwest")]
impl Info<reqwest::Client, Aui> {
    /// Fetch atom information for given atom.
    pub async fn fetch(self, config: &Config) -> Result<AtomResponse, Error<reqwest::Error>> {
        let path = format!("content/{}/AUI/{}", config.version(), self.value());
        let url = config.base_url().join(&path).map_err(Error::Url)?;
        let url = Url::parse_with_params(
            url.as_str(),
            &[("apiKey", config.api_key().expose_secret().to_owned())],
        )
        .map_err(Error::Url)?;

        let request = http::Request::builder()
            .method("GET")
            .uri(url.as_str())
            .body(())
            .map_err(reqwest::Error::Http)
            .map_err(Error::Client)?;

        let response: http::Response<Page<AtomResponse>> = self
            .client()
            .execute(request)
            .await
            .map_err(Error::Client)?;
        let response = response.into_body();

        Ok(response.content)
    }
}
