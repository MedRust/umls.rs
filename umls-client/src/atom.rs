//! Retrieve the atoms associated with a UMLS concept.
//!
//! An atom is a source-specific name or identifier connected to a UMLS
//! concept. Use [`crate::atom::Config::builder`] to select a concept, add
//! optional filters, and create an [`crate::atom::Atom`] request with
//! [`crate::atom::Builder::atoms`].
//!
//! CUI-based all-atoms and preferred-atom requests use the release selected by
//! [`crate::config::Builder::version`], which defaults to `current`. Requests
//! built from an explicit atom-list [`http::Uri`] retain its endpoint path and
//! release; the request adds authentication, filter, and pagination parameters.
//!
//! CUI-based requests resolve relative paths against the configured base URL.
//! Include a trailing `/` to preserve its full path prefix (see
//! [`crate::config::Builder::base_url`]). The default base URL is
//! `https://uts-ws.nlm.nih.gov/rest/`; configuration validation rejects base
//! URL paths without a trailing slash.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "reqwest")]
//! use umls_http::reqwest;
//! use umls_client::{atom, core::cui::Cui, config::Config};
//!
//! # #[cfg(feature = "reqwest")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::builder(std::env::var("UMLS_API_KEY")?)
//!     .base_url("https://uts-ws.nlm.nih.gov/rest/".parse()?)
//!     .try_version("2026AA")?
//!     .build()?;
//! let cui = Cui::new("C0010200")?;
//! let atoms = atom::Config::builder(cui)
//!     .try_add_source_vocabulary("MSH")?
//!     .language("ENG")?
//!     .atoms()
//!     .with_client(reqwest::Client::default())
//!     .fetch(&config)
//!     .await?
//!     .into_content();
//!
//! for atom in atoms.list() {
//!     println!("{}: {}", atom.ui, atom.name);
//! }
//! # Ok(())
//! # }
//! ```

use crate::core::url::validator;
use std::{marker::PhantomData, num::NonZeroU64};

use serde::{self, Deserialize};
use umls_http::WithoutClient;

#[cfg(feature = "reqwest")]
use crate::types;

#[cfg(feature = "reqwest")]
use serde::Serialize;

#[cfg(feature = "reqwest")]
use crate::core::cui::Cui;

#[cfg(feature = "reqwest")]
use umls_http::reqwest;

use crate::search::{
    Content, Response,
    core::{DisplayName, EmptyString},
};

#[cfg(feature = "reqwest")]
use crate::config;

/// An error produced while constructing, sending, or decoding an atom request.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error<T> {
    /// A client error.
    #[error("Client error: {0}")]
    Client(#[source] T),

    /// Represents uri construction error.
    #[error("Uri construction failed: {0}")]
    Uri(#[source] http::uri::InvalidUri),

    /// Represents URL validation error.
    #[error("Url validation Error: {0}")]
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

#[derive(Debug, thiserror::Error)]
/// An error produced while changing the requested atom page.
pub enum BatchError {
    /// No next page exists, or incrementing the page number would exceed
    /// [`u64::MAX`].
    #[error("Page overflow")]
    Overflow,

    /// The first page has no preceding page.
    #[error("No previous page")]
    NoPreviousPage,
}

/// A value paired with non-zero pagination parameters.
#[derive(Debug, Clone)]
pub struct Batch<T> {
    size: NonZeroU64,
    number: NonZeroU64,
    value: T,
}

impl<T> Batch<T> {
    /// Creates a paginated value with the supplied page size and page number.
    pub fn new(size: NonZeroU64, number: NonZeroU64, value: T) -> Self {
        Self {
            size,
            number,
            value,
        }
    }

    /// Returns the maximum number of atoms requested per page.
    pub fn size(&self) -> std::num::NonZero<u64> {
        self.size
    }

    /// Returns the one-based page number.
    pub fn number(&self) -> std::num::NonZero<u64> {
        self.number
    }

    /// Returns the value associated with these pagination parameters.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Increments the page number by one.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Overflow`] if incrementing would exceed
    /// [`u64::MAX`].
    pub fn checked_next(&mut self) -> Result<(), BatchError> {
        let number = self
            .number
            .get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(BatchError::Overflow)?;

        self.number = number;
        Ok(())
    }

    /// Decrements the page number by one.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::NoPreviousPage`] when already on the first page.
    pub fn checked_previous(&mut self) -> Result<(), BatchError> {
        let number = self
            .number
            .get()
            .checked_sub(1)
            .and_then(NonZeroU64::new)
            .ok_or(BatchError::NoPreviousPage)?;

        self.number = number;
        Ok(())
    }
}

#[derive(Debug)]
/// Builder for a filtered, paginated concept-atoms request.
///
/// Requests default to the first page, use the library's default page size,
/// and exclude obsolete and suppressible atoms.
pub struct Builder<T> {
    value: T,
    source_vocabularies: Vec<DisplayName>,
    term_types: Vec<DisplayName>,
    language: Option<DisplayName>,
    include_obsolete: bool,
    include_suppressible: bool,
    page_size: NonZeroU64,
    page_number: NonZeroU64,
}

impl<T> Builder<T> {
    /// Creates a builder for a concept identifier or atom-list URL.
    pub fn new(value: T) -> Self {
        Builder {
            value,
            source_vocabularies: Vec::new(),
            term_types: Vec::new(),
            language: None,
            include_obsolete: false,
            include_suppressible: false,
            page_size: super::search::core::DEFAULT_PAGE_SIZE,
            page_number: super::search::core::DEFAULT_PAGE_NUMBER,
        }
    }

    /// Replaces the concept identifier or URL targeted by the request.
    pub fn value(mut self, value: T) -> Self {
        self.value = value;
        self
    }

    /// Replaces the source-vocabulary filters.
    pub fn source_vocabularies(mut self, source_vocabularies: Vec<DisplayName>) -> Self {
        self.source_vocabularies = source_vocabularies;
        self
    }

    /// Adds a validated source-vocabulary filter.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] if the value is empty after trimming.
    pub fn try_add_source_vocabulary(
        mut self,
        source_vocabulary: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.source_vocabularies.push(source_vocabulary.try_into()?);
        Ok(self)
    }

    /// Replaces the term-type filters.
    pub fn term_types(mut self, term_types: Vec<DisplayName>) -> Self {
        self.term_types = term_types;
        self
    }

    /// Adds a validated term-type filter.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] if the value is empty after trimming.
    pub fn try_add_term_type(
        mut self,
        term_type: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.term_types.push(term_type.try_into()?);
        Ok(self)
    }

    /// Restricts results to a validated language code.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] if the value is empty after trimming.
    pub fn language(
        mut self,
        language: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.language = Some(language.try_into()?);
        Ok(self)
    }

    /// Sets whether obsolete atoms should be included.
    pub fn should_include_obsolete(mut self, value: bool) -> Self {
        self.include_obsolete = value;
        self
    }

    /// Sets whether suppressible atoms should be included.
    pub fn should_include_suppressible(mut self, value: bool) -> Self {
        self.include_suppressible = value;
        self
    }

    /// Sets the one-based page number to request.
    pub fn page_number(mut self, page_number: NonZeroU64) -> Self {
        self.page_number = page_number;
        self
    }

    /// Sets the maximum number of atoms requested per page.
    pub fn page_size(mut self, page_size: NonZeroU64) -> Self {
        self.page_size = page_size;
        self
    }

    /// Builds the atom filters together with their pagination parameters.
    pub fn build(self) -> Batch<Config<T>> {
        let config = Config {
            value: self.value,
            source_vocabularies: self.source_vocabularies,
            term_types: self.term_types,
            language: self.language,
            include_obsolete: self.include_obsolete,
            include_suppressible: self.include_suppressible,
        };

        Batch::new(self.page_size, self.page_number, config)
    }

    /// Builds an executable atom request using the supplied client configuration.
    ///
    /// The marker `S` selects the atom operation.
    pub fn atoms<S>(self) -> Atom<WithoutClient, S, Batch<Config<T>>> {
        Atom::<WithoutClient, S, Batch<Config<T>>>::new(self.build())
    }
}

/// Filters and target information for retrieving concept atoms.
#[derive(Debug, Clone)]
pub struct Config<T> {
    value: T,
    source_vocabularies: Vec<DisplayName>,
    term_types: Vec<DisplayName>,
    language: Option<DisplayName>,
    include_obsolete: bool,
    include_suppressible: bool,
}

impl<T> Config<T> {
    /// Returns a builder for the supplied concept identifier or atom-list URL.
    pub fn builder(value: T) -> Builder<T> {
        Builder::new(value)
    }

    /// Returns the UMLS source-vocabulary filters.
    ///
    /// Each value identifies a vocabulary to which returned atoms must belong.
    pub fn source_vocabularies(&self) -> &[DisplayName] {
        &self.source_vocabularies
    }

    /// Returns the UMLS term-type filters.
    ///
    /// Each value identifies a term type to which returned atoms must belong.
    pub fn term_types(&self) -> &[DisplayName] {
        &self.term_types
    }

    /// Returns the optional language filter.
    pub fn language(&self) -> Option<&DisplayName> {
        self.language.as_ref()
    }

    /// Returns whether obsolete concept atoms are included.
    pub fn is_include_obsolete(&self) -> bool {
        self.include_obsolete
    }

    /// Returns whether suppressible concept atoms are included.
    pub fn is_include_suppressible(&self) -> bool {
        self.include_suppressible
    }

    /// Returns the concept identifier or URL targeted by this configuration.
    pub fn value(&self) -> &T {
        &self.value
    }
}

// #[cfg(feature = "reqwest")]
// impl Preferred {
//     /// Creates a request for the preferred atom associated with a concept.
//     ///
//     /// Pass a validated [`crate::core::cui::Cui`] as `value`, attach a `reqwest::Client` with [`Atom::with_client`], then call `fetch()` to
//     /// execute the request using the configured UMLS release.
//     ///
//     /// # Example
//     ///
//     /// ```no_run
//     /// # #[cfg(feature = "reqwest")]
//     /// use umls_http::reqwest;
//     /// use umls_client::{
//     ///     atom::Preferred,
//     ///     core::cui::Cui,
//     ///     config::Config,
//     /// };
//     ///
//     /// # #[cfg(feature = "reqwest")]
//     /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//     /// let config = Config::builder(std::env::var("UMLS_API_KEY")?)
//     ///     .base_url("https://uts-ws.nlm.nih.gov/rest/".parse()?)
//     ///     .try_version("2026AB")?
//     ///     .build()?;
//     /// let cui = Cui::new("C0010200")?;
//     /// let atom = Preferred::for_concept(cui)
//     ///     .with_client(reqwest::Client::default())
//     ///     .fetch(&config).await?;
//     ///
//     /// println!("{}: {}", atom.ui, atom.name);
//     /// # Ok(())
//     /// # }
//     /// ```
//     pub fn for_concept<T>(value: T) -> Atom<WithoutClient, Self, T> {
//         Atom::<WithoutClient, Self, T>::new(value)
//     }
// }

/// An executable atom operation associated with a UMLS client configuration.
///
/// `S` identifies the operation and `T` stores its request state. Most callers
/// create this type through [`Builder::atoms`].
pub struct Atom<C, S, T> {
    client: C,
    value: T,
    _data: PhantomData<S>,
}

impl<S, T> Atom<WithoutClient, S, T> {
    /// Creates an atom operation from client configuration and request state.
    pub fn new(value: T) -> Atom<WithoutClient, S, T> {
        Atom {
            client: WithoutClient,
            value,
            _data: PhantomData,
        }
    }
}

impl<C, S, T> Atom<C, S, T> {
    /// Returns a reference to client.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Returns a reference to value.
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<S, T> Atom<WithoutClient, S, T> {
    /// Attaches an owned HTTP client without sending a request.
    ///
    /// Execution requires the `reqwest` feature and a `reqwest::Client`.
    /// Clone an existing client to share its connection pool across requests.
    pub fn with_client<C>(self, client: C) -> Atom<C, S, T> {
        Atom {
            client,
            value: self.value,
            _data: self._data,
        }
    }
}

#[cfg(feature = "reqwest")]
impl Atom<reqwest::Client, types::atom::All, Batch<Config<Cui>>> {
    /// Fetches one filtered page of atoms for a validated CUI.
    ///
    /// Uses the release returned by [`crate::config::Config::version`].
    ///
    /// # Errors
    ///
    /// Returns an error if URL construction, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<
        Response<
            Page<Content<AtomResponse>>,
            Atom<reqwest::Client, types::atom::All, Batch<Config<Cui>>>,
        >,
        Error<reqwest::Error>,
    > {
        let url = umls_config
            .atom_url::<types::Concept, types::atom::All, _>(&self.value)
            .url();

        let body = Atom::<_, types::atom::All, _>::new(url)
            .with_client(self.client().clone())
            .fetch(umls_config)
            .await?;

        Ok(Response::new(body, self))
    }
}

#[cfg(feature = "reqwest")]
impl Atom<reqwest::Client, types::atom::All, Cui> {
    /// Fetches the first, unfiltered page of atoms for a validated CUI.
    ///
    /// Uses the release returned by [`crate::config::Config::version`].
    ///
    /// # Errors
    ///
    /// Returns an error if URL construction, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Content<AtomResponse>, Error<reqwest::Error>> {
        let url = umls_config
            .atom_url::<types::Concept, types::atom::All, _>(self.value)
            .url();

        Atom::<_, types::atom::All, _>::new(url)
            .with_client(self.client)
            .fetch(umls_config)
            .await
            .map(|x| x.content)
    }
}

#[cfg(feature = "reqwest")]
impl Atom<reqwest::Client, types::atom::All, url::Url> {
    /// Fetches one page of concept atoms from an atom-list URL endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if URL parsing, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<AtomResponse>>, Error<reqwest::Error>> {
        umls_config
            .atom_url_validator::<types::Concept, types::atom::All>(self.value())
            .validate()
            .map_err(Error::Validator)?;

        let url = umls_config.authenticated_url(self.value);

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(Error::Client)
    }
}

#[cfg(feature = "reqwest")]
impl Atom<reqwest::Client, types::atom::Preferred, Cui> {
    /// Fetches the preferred concept atom for the given [`crate::core::cui::Cui`] using the
    /// configured UMLS release.
    ///
    /// # Errors
    ///
    /// Returns an error if URL construction, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<AtomResponse, Error<reqwest::Error>> {
        let url = umls_config
            .atom_url::<types::Concept, types::atom::Preferred, _>(self.value)
            .url();

        Atom::<_, types::atom::Preferred, _>::new(url)
            .with_client(self.client)
            .fetch(umls_config)
            .await
    }
}

#[cfg(feature = "reqwest")]
impl Atom<reqwest::Client, types::atom::Preferred, url::Url> {
    /// Fetches the preferred concept atom from a preferred-atom URL endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if URL parsing, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<AtomResponse, Error<reqwest::Error>> {
        umls_config
            .atom_url_validator::<types::Concept, types::atom::Preferred>(self.value())
            .validate()
            .map_err(Error::Validator)?;

        let url = umls_config.authenticated_url(self.value);

        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Page<AtomResponse>>()
            .await
            .map(|x| x.content)
            .map_err(Error::Client)
    }
}

impl<T, C> Response<Page<Content<T>>, C> {
    /// Consumes the response and returns its atom content.
    ///
    /// Pagination metadata and retained request state are discarded.
    pub fn into_content(self) -> Content<T> {
        self.body.content
    }

    /// Converts the response into a navigable page retaining request state.
    pub fn into_page(self) -> super::search::Page<Content<T>, C> {
        let count = self.body.count().saturating_mul(self.body.size());
        let size = self.body.size();
        let number = self.body.number();

        let content = self.body.content;

        super::search::Page::new(content, size, number, Some(count), self.client)
    }
}

#[cfg(feature = "reqwest")]
impl<T, U>
    super::search::Page<Content<T>, Atom<reqwest::Client, types::atom::All, Batch<Config<U>>>>
{
    /// Consumes this page and returns a request for the next page.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Overflow`] if the response indicates that no next
    /// page exists or if the page number cannot be incremented.
    pub fn next(
        mut self,
    ) -> Result<Atom<reqwest::Client, types::atom::All, Batch<Config<U>>>, BatchError> {
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
    ) -> Result<Atom<reqwest::Client, types::atom::All, Batch<Config<U>>>, BatchError> {
        if !self.has_previous() {
            return Err(BatchError::NoPreviousPage);
        }

        self.client_mut().value.checked_previous()?;

        Ok(self.client)
    }
}

#[derive(Debug, Clone, Deserialize)]
/// A decoded page returned by the UMLS concept-atoms endpoint.
pub struct Page<T> {
    #[serde(rename = "pageSize")]
    size: NonZeroU64,
    #[serde(rename = "pageNumber")]
    number: NonZeroU64,
    #[serde(rename = "pageCount")]
    count: NonZeroU64,
    #[serde(rename = "result")]
    pub(crate) content: T,
}

impl<T> Page<T> {
    /// Returns the maximum number of atoms requested or returned per page.
    pub fn size(&self) -> std::num::NonZero<u64> {
        self.size
    }

    /// Returns the one-based number of this page.
    pub fn number(&self) -> std::num::NonZero<u64> {
        self.number
    }

    /// Returns the page count reported by the UMLS API.
    pub fn count(&self) -> std::num::NonZero<u64> {
        self.count
    }

    /// Returns this page's decoded content.
    pub fn content(&self) -> &T {
        &self.content
    }

    /// A conversion of page content type without affecting the size, number, count.
    pub fn try_map<U, E>(self, f: impl FnOnce(T) -> Result<U, E>) -> Result<Page<U>, E> {
        Ok(Page {
            size: self.size,
            number: self.number,
            count: self.count,
            content: f(self.content)?,
        })
    }
}

#[cfg(feature = "reqwest")]
mod bool_as_string {
    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(if *value { "true" } else { "false" })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(de::Error::custom("expected \"true\" or \"false\"")),
        }
    }
}

#[cfg(feature = "reqwest")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// A summary of one source-specific atom associated with a UMLS concept.
pub struct AtomResponse {
    /// The Atom Unique Identifier (AUI).
    pub ui: DisplayName,

    /// Whether the atom is suppressible.
    #[serde(with = "bool_as_string")]
    pub suppressible: bool,

    /// Whether the atom is obsolete.
    #[serde(with = "bool_as_string")]
    pub obsolete: bool,
    /// The abbreviation of the atom's root source vocabulary.
    pub root_source: DisplayName,
    /// The source vocabulary's term-type abbreviation.
    pub term_type: DisplayName,
    /// The atom's language code.
    pub language: DisplayName,
    /// The human-readable atom name.
    pub name: DisplayName,
}
