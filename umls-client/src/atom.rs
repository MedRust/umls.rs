//! Retrieve the atoms associated with a UMLS concept.
//!
//! An atom is a source-specific name or identifier connected to a UMLS
//! concept. Use [`crate::atom::Config::builder`] to select a concept, add
//! optional filters, and create an [`crate::atom::Atom`] request with
//! [`crate::atom::Builder::atoms`].
//!
//! CUI-based all-atoms and preferred-atom requests use the release selected by
//! [`crate::config::Builder::version`], which defaults to `current`. Requests
//! built from an explicit atom-list [`Url`] use that URL unchanged; the client
//! configuration does not replace the release embedded in it.
//!
//! # Example
//!
//! ```no_run
//! use umls_client::{atom, concept::cui::Cui, config::Config};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::builder(std::env::var("UMLS_API_KEY")?)
//!     .version("2026AA".into())
//!     .build()?;
//! let cui = Cui::new("C0010200")?;
//! let atoms = atom::Config::builder(cui)
//!     .try_add_source_vocabulary("MSH")?
//!     .language("ENG")?
//!     .atoms(&config)
//!     .fetch()
//!     .await?
//!     .into_content();
//!
//! for atom in atoms.list() {
//!     println!("{}: {}", atom.ui, atom.name);
//! }
//! # Ok(())
//! # }
//! ```

use std::{marker::PhantomData, num::NonZeroU64};

use secrecy::ExposeSecret;
use serde::{self, Deserialize, Deserializer};
use url::Url;

use crate::{
    concept::cui::Cui,
    config,
    search::{
        Content, Response,
        core::{DisplayName, EmptyString},
    },
};

/// An error produced while constructing, sending, or decoding an atom request.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The atom endpoint or its query parameters could not be represented as a
    /// valid URL.
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// The HTTP request could not be sent or a response could not be received.
    #[error("atom request failed: {0}")]
    Transport(#[source] reqwest::Error),

    /// The server returned an unsuccessful HTTP status.
    #[error("atom endpoint returned an unsuccessful status: {0}")]
    HttpStatus(#[source] reqwest::Error),

    /// The response body could not be decoded as an atom page.
    #[error("failed to decode atom response: {0}")]
    Decode(#[source] reqwest::Error),
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
    /// The marker `S` selects the atom operation. [`All`] is the marker used
    /// for retrieving all atoms associated with a concept.
    pub fn atoms<'a, S>(self, umls_config: &'a config::Config) -> Atom<'a, S, Batch<Config<T>>> {
        Atom::new(umls_config, self.build())
    }
}

/// Filters and target information for retrieving concept atoms.
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

/// Marker selecting retrieval of all atoms associated with a concept.
pub struct All;

/// Marker selecting retrieval of preferred atoms associated with a concept.
pub struct Preferred;

impl Preferred {
    /// Creates a request for the preferred atom associated with a concept.
    ///
    /// Pass a validated [`Cui`] as `value`, then call [`Atom::fetch`] to
    /// execute the request using the configured UMLS release.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use umls_client::{
    ///     atom::Preferred,
    ///     concept::cui::Cui,
    ///     config::Config,
    /// };
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::builder(std::env::var("UMLS_API_KEY")?)
    ///     .version("2026AA".into())
    ///     .build()?;
    /// let cui = Cui::new("C0010200")?;
    /// let atom = Preferred::for_concept(&config, cui).fetch().await?;
    ///
    /// println!("{}: {}", atom.ui, atom.name);
    /// # Ok(())
    /// # }
    /// ```
    pub fn for_concept<'a, T>(umls_config: &'a config::Config, value: T) -> Atom<'a, Self, T> {
        Atom::new(umls_config, value)
    }
}

/// An executable atom operation associated with a UMLS client configuration.
///
/// `S` identifies the operation and `T` stores its request state. Most callers
/// create this type through [`Builder::atoms`].
pub struct Atom<'a, S, T> {
    umls_config: &'a config::Config,
    value: T,
    _data: PhantomData<S>,
}

impl<'a, S, T> Atom<'a, S, T> {
    /// Creates an atom operation from client configuration and request state.
    pub fn new(umls_config: &'a config::Config, value: T) -> Self {
        Self {
            umls_config,
            value,
            _data: PhantomData,
        }
    }

    /// Returns the operation's request state.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the client configuration used to execute the operation.
    pub fn umls_config(&self) -> &'a config::Config {
        self.umls_config
    }
}

impl<'a, S, T> Atom<'a, S, Batch<Config<T>>> {
    fn inject_query_params(&self, url_string: &str) -> Result<Url, Error> {
        let source_vocabularies_params = self
            .value()
            .value()
            .source_vocabularies()
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let tty_params = self
            .value()
            .value()
            .term_types()
            .iter()
            .map(|x| x.as_ref())
            .collect::<Vec<&str>>()
            .join(",");

        let include_obsolete = self.value().value().is_include_obsolete().to_string();
        let include_suppressible = self.value().value().is_include_suppressible().to_string();

        let page_number = self.value().number().to_string();
        let page_size = self.value().size().to_string();

        let mut params = [
            ("apiKey", self.umls_config().api_key().expose_secret()),
            ("sabs", &source_vocabularies_params),
            ("tty", &tty_params),
            ("includeObsolete", &include_obsolete),
            ("includeSuppressible", &include_suppressible),
            ("pageNumber", &page_number),
            ("pageSize", &page_size),
        ]
        .to_vec();

        if let Some(language) = self.value().value().language().map(|x| x.as_ref()) {
            params.push(("language", language));
        }

        Url::parse_with_params(url_string, &params).map_err(Error::Url)
    }
}

impl<'a> Atom<'a, All, Batch<Config<Cui>>> {
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
    ) -> Result<Response<Page<Content<AtomResponse>>, Atom<'a, All, Batch<Config<Cui>>>>, Error>
    {
        let path = format!(
            "/content/{}/CUI/{}/atoms",
            self.umls_config().version(),
            self.value().value().value().as_ref()
        );

        let url_string = self.umls_config().base_url().join(&path)?.to_string();

        let url = self.inject_query_params(&url_string)?;

        let body = self
            .umls_config()
            .client()
            .get(url)
            .send()
            .await
            .map_err(Error::Transport)?
            .error_for_status()
            .map_err(Error::HttpStatus)?
            .json::<Page<Content<AtomResponse>>>()
            .await
            .map_err(Error::Decode)?;

        Ok(Response::new(body, self))
    }
}

impl<'a> Atom<'a, All, Batch<Config<Url>>> {
    /// Fetches one filtered page of atoms from an atom-list URL.
    ///
    /// Uses the supplied URL without replacing its release with the configured
    /// version.
    ///
    /// # Errors
    ///
    /// Returns an error if URL construction, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(
        self,
    ) -> Result<Response<Page<Content<AtomResponse>>, Atom<'a, All, Batch<Config<Url>>>>, Error>
    {
        let url_string = self.value().value().value().to_string();
        let url = self.inject_query_params(&url_string)?;

        let body = self
            .umls_config()
            .client()
            .get(url)
            .send()
            .await
            .map_err(Error::Transport)?
            .error_for_status()
            .map_err(Error::HttpStatus)?
            .json::<Page<Content<AtomResponse>>>()
            .await
            .map_err(Error::Decode)?;

        Ok(Response::new(body, self))
    }
}

impl<'a> Atom<'a, All, Cui> {
    /// Fetches the first, unfiltered page of atoms for a validated CUI.
    ///
    /// Uses the release returned by [`crate::config::Config::version`].
    ///
    /// # Errors
    ///
    /// Returns an error if URL construction, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(self) -> Result<Content<AtomResponse>, Error> {
        let path = format!(
            "/content/{}/CUI/{}/atoms",
            self.umls_config().version(),
            self.value()
        );
        let url_string = self.umls_config().base_url().join(&path)?.to_string();

        let url = Url::parse_with_params(
            &url_string,
            &[("apiKey", self.umls_config().api_key().expose_secret())],
        )?;

        let body = self
            .umls_config()
            .client()
            .get(url)
            .send()
            .await
            .map_err(Error::Transport)?
            .error_for_status()
            .map_err(Error::HttpStatus)?
            .json::<Page<Content<AtomResponse>>>()
            .await
            .map_err(Error::Decode)?;

        Ok(body.content)
    }
}

impl<'a> Atom<'a, Preferred, Cui> {
    /// Fetches the preferred concept atom for the given [`Cui`] using the
    /// configured UMLS release.
    ///
    /// # Errors
    ///
    /// Returns an error if URL construction, transport, HTTP status validation,
    /// or response decoding fails.
    pub async fn fetch(self) -> Result<AtomResponse, Error> {
        let path = format!(
            "content/{}/CUI/{}/atoms/preferred",
            self.umls_config().version(),
            self.value()
        );
        let url_string = self
            .umls_config()
            .base_url()
            .join(&path)
            .map_err(Error::Url)?
            .to_string();

        let url = Url::parse_with_params(
            &url_string,
            &[("apiKey", self.umls_config().api_key().expose_secret())],
        )?;

        let body = self
            .umls_config()
            .client()
            .get(url)
            .send()
            .await
            .map_err(Error::Transport)?
            .error_for_status()
            .map_err(Error::HttpStatus)?
            .json::<Page<AtomResponse>>()
            .await
            .map_err(Error::Decode)?;

        Ok(body.content)
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
        let count = self.body.count();
        let size = self.body.size();
        let number = self.body.number();

        let content = self.body.content;

        super::search::Page::new(content, size, number, Some(count), self.client)
    }
}

impl<'a, T, U> super::search::Page<Content<T>, Atom<'a, All, Batch<Config<U>>>> {
    /// Consumes this page and returns a request for the next page.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError::Overflow`] if the response indicates that no next
    /// page exists or if the page number cannot be incremented.
    pub fn next(mut self) -> Result<Atom<'a, All, Batch<Config<U>>>, BatchError> {
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
    pub fn previous(mut self) -> Result<Atom<'a, All, Batch<Config<U>>>, BatchError> {
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
    content: T,
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
}

fn deserialize_bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(serde::de::Error::custom("expected \"true\" or \"false\"")),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A summary of one source-specific atom associated with a UMLS concept.
pub struct AtomResponse {
    /// The Atom Unique Identifier (AUI).
    pub ui: DisplayName,

    /// Whether the atom is suppressible.
    #[serde(deserialize_with = "deserialize_bool_from_string")]
    pub suppressible: bool,

    /// Whether the atom is obsolete.
    #[serde(deserialize_with = "deserialize_bool_from_string")]
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
