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
use crate::search::core::get_trimmed_non_empty_string;
#[cfg(feature = "reqwest")]
use crate::{definition, relation};

#[cfg(feature = "reqwest")]
use crate::{concept::cui::Cui, config::Config, search::ConceptSnapshot};

/// Validated UMLS Concept Unique Identifiers (CUIs).
pub mod cui {
    use core::str;
    use std::str::FromStr;

    use crate::search::core::{EmptyString, get_trimmed_non_empty_string};

    #[derive(Debug, thiserror::Error)]
    /// An error produced while validating a [`crate::concept::cui::Cui`].
    pub enum Error {
        /// The supplied identifier was empty after trimming.
        #[error("Empty String Error")]
        EmptyString,

        /// The supplied identifier did not match `C` followed by seven digits.
        #[error("invalid CUI format: {0}")]
        InvalidFormat(String),
    }

    impl From<EmptyString> for Error {
        fn from(_value: EmptyString) -> Self {
            Self::EmptyString
        }
    }

    fn is_valid_cui(value: &str) -> bool {
        value.len() == 8
            && value.starts_with('C')
            && value[1..].bytes().all(|byte| byte.is_ascii_digit())
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    /// A validated UMLS Concept Unique Identifier.
    ///
    /// CUIs consist of an uppercase `C` followed by seven ASCII digits.
    pub struct Cui(String);

    impl Cui {
        /// Creates a CUI after trimming surrounding whitespace.
        ///
        /// # Errors
        ///
        /// Returns [`Error::EmptyString`] for an empty value or
        /// [`Error::InvalidFormat`] when the identifier has an invalid shape.
        ///
        /// # Example
        ///
        /// ```
        /// use umls_client::concept::cui::Cui;
        ///
        /// let cui = Cui::new(" C0010200 ")?;
        /// assert_eq!(cui.as_str(), "C0010200");
        /// # Ok::<(), umls_client::concept::cui::Error>(())
        /// ```
        pub fn new(value: &str) -> Result<Self, Error> {
            let trimmed = get_trimmed_non_empty_string(value)?;

            if !is_valid_cui(trimmed) {
                return Err(Error::InvalidFormat(trimmed.into()));
            }

            Ok(Self(trimmed.to_owned()))
        }

        /// Returns the validated identifier as a string slice.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl AsRef<str> for Cui {
        fn as_ref(&self) -> &str {
            self.as_str()
        }
    }

    impl FromStr for Cui {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Self::new(s)
        }
    }

    impl TryFrom<&str> for Cui {
        type Error = Error;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            Self::new(value)
        }
    }

    impl TryFrom<String> for Cui {
        type Error = Error;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Self::new(&value)
        }
    }

    impl std::fmt::Display for Cui {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn creates_valid_cui() {
            let cui = Cui::new("C0010200").unwrap();

            assert_eq!(cui.as_str(), "C0010200");
        }

        #[test]
        fn trims_surrounding_whitespace() {
            for value in [" C0010200", "C0010200 ", "  C0010200  ", "\tC0010200\n"] {
                let cui = Cui::new(value).unwrap();

                assert_eq!(cui.as_str(), "C0010200");
            }
        }

        #[test]
        fn rejects_empty_cui() {
            assert!(matches!(Cui::new(""), Err(Error::EmptyString)));
        }

        #[test]
        fn rejects_whitespace_only_cui() {
            for value in [" ", "   ", "\t", "\n", " \t\n "] {
                assert!(
                    matches!(Cui::new(value), Err(Error::EmptyString)),
                    "{value:?} should be rejected as empty"
                );
            }
        }

        #[test]
        fn rejects_wrong_prefix() {
            for value in ["A0010200", "S0010200", "c0010200", "X0010200", "00010200"] {
                assert!(
                    matches!(Cui::new(value), Err(Error::InvalidFormat(_))),
                    "{value} should have been rejected"
                );
            }
        }

        #[test]
        fn rejects_incorrect_length() {
            for value in ["C", "C0", "C001020", "C00102000", "C000000000"] {
                assert!(
                    matches!(Cui::new(value), Err(Error::InvalidFormat(_))),
                    "{value} should have been rejected"
                );
            }
        }

        #[test]
        fn rejects_non_digit_characters() {
            for value in [
                "C001O200", // letter O instead of zero
                "C00102A0", "C001-200", "C001 200", "C001.200", "C001_200",
            ] {
                assert!(
                    matches!(Cui::new(value), Err(Error::InvalidFormat(_))),
                    "{value} should have been rejected"
                );
            }
        }

        #[test]
        fn rejects_non_ascii_digits() {
            let result = Cui::new("C１２３４５６７");

            assert!(matches!(result, Err(Error::InvalidFormat(_))));
        }

        #[test]
        fn parses_cui_from_string_slice() {
            let cui: Cui = "  C0010200  ".parse().unwrap();

            assert_eq!(cui.as_str(), "C0010200");
        }

        #[test]
        fn converts_from_borrowed_string() {
            let cui = Cui::try_from("  C0010200  ").unwrap();

            assert_eq!(cui.as_str(), "C0010200");
        }

        #[test]
        fn converts_from_owned_string() {
            let cui = Cui::try_from("  C0010200  ".to_owned()).unwrap();

            assert_eq!(cui.as_str(), "C0010200");
        }

        #[test]
        fn exposes_cui_through_as_ref() {
            let cui = Cui::new("C0010200").unwrap();

            assert_eq!(cui.as_ref(), "C0010200");
        }

        #[test]
        fn formats_cui_for_display() {
            let cui = Cui::new("C0010200").unwrap();

            assert_eq!(cui.to_string(), "C0010200");
        }

        #[test]
        fn validates_cui_format() {
            assert!(is_valid_cui("C0010200"));
            assert!(is_valid_cui("C0000000"));

            assert!(!is_valid_cui("C001020"));
            assert!(!is_valid_cui("c0010200"));
            assert!(!is_valid_cui("C00102A0"));
        }
    }
}

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
/// The available `fetch` method depends on whether the value is a [`crate::concept::cui::Cui`], a
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
    /// use umls_client::{concept::{Info, cui::Cui}, config::Config};
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
    /// Endpoint for the concept's definitions, when available.
    pub definitions: Option<definition::Definition<umls_http::WithoutClient, Url>>,
    /// Endpoint for the concept's relations, when available.
    pub relations: Option<relation::Relation<umls_http::WithoutClient, Url>>,
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
    atoms: OptionalResource,

    /// Endpoint for the concept's definitions, when available.
    definitions: OptionalResource,

    /// Endpoint for the concept's relations, when available.
    relations: OptionalResource,
    /// Endpoint for the default preferred atom, when available.
    default_preferred_atom: OptionalResource,
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
