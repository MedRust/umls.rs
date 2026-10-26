use std::marker::PhantomData;

#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;

#[cfg(feature = "reqwest")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "reqwest")]
use url::Url;

#[cfg(feature = "reqwest")]
use crate::core::aui::Aui;

#[cfg(feature = "reqwest")]
use crate::search::core::{DisplayName, EmptyString};
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

    /// Represents the source parent types.
    ///
    /// It used as state for [`super::Parent`] to fetch source-asserted parents.
    pub struct Source;
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

    /// Failed to create a domain object.
    #[error("Invalid Domain object error.")]
    Domain,

    /// Represents invalid base URL.
    #[error("Invalid base URL")]
    InvalidBaseUrl,
}

#[cfg(feature = "reqwest")]
impl Parent<types::Source, umls_http::reqwest::Client, Url> {
    /// Fetches one page of source-asserted parent concepts using the wrapped endpoint URL.
    ///
    /// The URL is validated at runtime against the base URL in `umls_config` to ensure requests
    /// are only dispatched to the authorized UMLS host and path prefix.
    ///
    /// # Errors
    /// Returns [`Error`] if:
    /// - [`Error::Validation`]: The target URL scheme, host, port or path prefix does not match
    ///   [`umls_config.base_url()`](crate::config::Config::base_url).
    /// - [`Error::Url`]: The URL with credential and query parameters could not be constructed.
    /// - [`Error::Http`]: The HTTP request could not be constructed.
    /// - [`Error::Client`]: The transport failed or the server returns the error status.
    /// - [`Error::Domain`]: One or more response fields failed validation when converting into a
    ///   [`SourceAtomCluster`].
    ///
    /// # Example
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use umls_client::{config::Config, parent::{Parent, types::Source}};
    /// use umls_http::reqwest::Client;
    /// use url::Url;
    ///
    /// let config = Config::builder("your-api-key").build()?;
    /// let url: Url = "https://uts-ws.nlm.nih.gov/rest/content/current/source/SNOMEDCT_US/82065001/parents".parse()?;
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

        let response: Page<Content<SerializableSourceAtomCluster>> = self
            .client()
            .execute(request)
            .await
            .map_err(Error::Client)?
            .into_body();

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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SerializableSourceAtomCluster {
    ui: String,
    suppressible: bool,
    obsolete: bool,
    root_source: String,
    atom_count: u64,
    /// Explicitly renamed because UTS capitalizes the 'V' ("cVMemberCount").
    #[serde(rename = "cVMemberCount")]
    cv_member_count: u64,
    name: String,
}

/// A source-asserted concept or atom cluster defined by a source vocabulary (such as SNOMED CT or MeSH).
#[cfg(feature = "reqwest")]
#[derive(Debug, Clone)]
pub struct SourceAtomCluster {
    ui: DisplayName,
    /// Human-readable name.
    name: DisplayName,
    root_source: DisplayName,
    suppressible: bool,
    obsolete: bool,
    atom_count: u64,
    cv_member_count: u64,
}

#[cfg(feature = "reqwest")]
impl SourceAtomCluster {
    /// Returns the unique identifier (e.g. SNOMED CT code) of this source concept.
    pub fn ui(&self) -> &DisplayName {
        &self.ui
    }

    /// Returns the human-readable preferred name of this source concept.
    pub fn name(&self) -> &DisplayName {
        &self.name
    }

    /// Returns a root source vocabulary abbreviation (e.g. SNOMEDCT_US).
    pub fn root_source(&self) -> &DisplayName {
        &self.root_source
    }

    /// Returns `true` if the source concept is marked as suppressible.
    pub fn suppressible(&self) -> bool {
        self.suppressible
    }

    /// Returns `true` if the source concept is obsolete.
    pub fn obsolete(&self) -> bool {
        self.obsolete
    }

    /// Returns number of atoms associated with this source concept.
    pub fn atom_count(&self) -> u64 {
        self.atom_count
    }

    /// Returns the number of Content View (CV) memberships for this source concept.
    pub fn cv_member_count(&self) -> u64 {
        self.cv_member_count
    }
}

#[cfg(feature = "reqwest")]
impl TryFrom<SerializableSourceAtomCluster> for SourceAtomCluster {
    type Error = crate::search::core::EmptyString;

    fn try_from(wire: SerializableSourceAtomCluster) -> Result<Self, Self::Error> {
        Ok(Self {
            ui: wire.ui.parse()?,
            name: wire.name.parse()?,
            root_source: wire.root_source.parse()?,
            suppressible: wire.suppressible,
            obsolete: wire.obsolete,
            atom_count: wire.atom_count,
            cv_member_count: wire.cv_member_count,
        })
    }
}

/// A source-asserted concept identified by its source vocabulary and code.
#[cfg(feature = "reqwest")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConcept {
    source: DisplayName,
    code: DisplayName,
}

#[cfg(feature = "reqwest")]
impl SourceConcept {
    /// Creates a new [`SourceConcept`] with the given source vocabulary and code.
    pub fn new(source: DisplayName, code: DisplayName) -> Self {
        Self { source, code }
    }

    /// Returns the root source vocabulary abbreviation (e.g. SNOMEDCT_US).
    pub fn source(&self) -> &DisplayName {
        &self.source
    }

    /// Returns the unique code/identifier of associated source vocabulary.
    pub fn code(&self) -> &DisplayName {
        &self.code
    }
}

#[cfg(feature = "reqwest")]
impl Parent<types::Source, umls_http::reqwest::Client, SourceConcept> {
    /// Fetches one page of source-asserted parent concepts using [`SourceConcept`].
    ///
    /// # Errors
    /// Returns [`Error`] if:
    /// - [`Error::Validation`]: The target URL scheme, host, port or path prefix does not match
    ///   [`umls_config.base_url()`](crate::config::Config::base_url).
    /// - [`Error::Url`]: The URL with credentials and query parameters could not be constructed.
    /// - [`Error::Http`]: The HTTP request could not be constructed.
    /// - [`Error::Client`]: The transport failed or the server returned an error status.
    /// - [`Error::Domain`]: One or more response fields failed validation when converting into a
    ///   [`SourceAtomCluster`].
    ///
    /// # Example
    /// ```no_run
    /// # #[cfg(feature = "reqwest")]
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use umls_client::{config::Config, parent::{Parent, types::Source, SourceConcept}};
    /// use umls_http::reqwest::Client;
    /// use url::Url;
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
        let version = umls_config.version().to_string();

        let mut url = umls_config.base_url().clone();
        url.path_segments_mut()
            .map_err(|_| Error::InvalidBaseUrl)?
            .extend(&[
                "content",
                &version,
                "source",
                self.value().source().as_ref(),
                self.value().code().as_ref(),
                "parents",
            ]);

        Parent::<types::Source, _, _>::new(url)
            .with_client(self.client)
            .fetch(umls_config)
            .await
    }
}

#[cfg(test)]
#[cfg(feature = "reqwest")]
mod tests {

    use super::*;

    #[test]
    fn success_on_serializable_source_atom_cluster_to_source_atom_cluster() {
        let object = SerializableSourceAtomCluster {
            ui: "82065001".into(),
            suppressible: false,
            obsolete: false,
            root_source: "SNOMEDCT_US".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "Fracture of carpal bone".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_ok());

        let object = result.unwrap();
        assert_eq!(object.ui().as_str(), "82065001");
        assert_eq!(object.name().as_str(), "Fracture of carpal bone");
        assert!(!object.suppressible());
        assert!(!object.obsolete());
        assert_eq!(object.root_source().as_str(), "SNOMEDCT_US");
    }

    #[test]
    fn failed_on_empty_ui() {
        let object = SerializableSourceAtomCluster {
            ui: "".into(),
            suppressible: false,
            obsolete: false,
            root_source: "SNOMEDCT_US".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "Fracture of carpal bone".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn failed_on_empty_name() {
        let object = SerializableSourceAtomCluster {
            ui: "82065001".into(),
            suppressible: false,
            obsolete: false,
            root_source: "SNOMEDCT_US".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn failed_on_empty_root_source() {
        let object = SerializableSourceAtomCluster {
            ui: "82065001".into(),
            suppressible: false,
            obsolete: false,
            root_source: "".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "Fracture of carpal bone".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_err());
    }
}
