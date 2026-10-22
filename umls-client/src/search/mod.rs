//! High-level search results, conversion, and pagination.

use std::{marker::PhantomData, num::NonZeroU64};

use serde::Deserialize;
use umls_http::WithoutClient;
use url::Url;

use crate::{
    concept,
    config::Config,
    search::core::{
        DEFAULT_PAGE_NUMBER, DEFAULT_PAGE_SIZE, DisplayName, EmptyString, Inclusion, InputType,
        ReturnIdType, SearchType, Term, TermMatch,
    },
};

#[cfg(feature = "reqwest")]
use crate::search::raw::{RawSearchDto, SearchResponse};

/// Low-level request and response types for the UMLS search endpoint.
pub mod raw;

/// Shared validation primitives used by public value types.
pub mod core;

/// Builder for a validated UMLS search [`Request`].
///
/// New builders exclude obsolete and suppressible terms, return concept
/// identifiers, use word matching, require all terms, and request the first
/// page with 25 results.
pub struct Builder {
    term: Term,
    input_type: InputType,
    return_id_type: ReturnIdType,
    obsolete: Inclusion,
    suppressible: Inclusion,
    source_vocabularies: Vec<DisplayName>,
    search_type: SearchType,
    semantic_types: Vec<DisplayName>,
    semantic_groups: Vec<DisplayName>,
    term_match: TermMatch,
    page_size: NonZeroU64,
    page_number: NonZeroU64,
}

impl Builder {
    /// Creates a builder with the default UMLS search parameters.
    pub fn new(term: Term) -> Self {
        Self {
            term,
            input_type: InputType::Atom,
            return_id_type: ReturnIdType::Concept,
            obsolete: Inclusion::Exclude,
            suppressible: Inclusion::Exclude,
            source_vocabularies: Vec::new(),
            search_type: SearchType::Words,
            semantic_types: Vec::new(),
            semantic_groups: Vec::new(),
            term_match: TermMatch::AllTerms,
            page_size: DEFAULT_PAGE_SIZE,
            page_number: DEFAULT_PAGE_NUMBER,
        }
    }

    /// Replaces the search term.
    pub fn term(mut self, term: Term) -> Self {
        self.term = term;
        self
    }

    /// Sets how the search term should be interpreted.
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// Sets what type of information to return.
    pub fn return_id_type(mut self, return_id_type: ReturnIdType) -> Self {
        self.return_id_type = return_id_type;
        self
    }

    /// Controls whether matches on obsolete terms are included.
    pub fn obsolete(mut self, obsolete: Inclusion) -> Self {
        self.obsolete = obsolete;
        self
    }

    /// Controls whether matches on suppressible terms are included.
    pub fn suppressible(mut self, suppressible: Inclusion) -> Self {
        self.suppressible = suppressible;
        self
    }

    /// Replaces the source-vocabulary filters.
    pub fn source_vocabularies(mut self, source_vocabularies: Vec<DisplayName>) -> Self {
        self.source_vocabularies = source_vocabularies;
        self
    }

    /// Adds a source-vocabulary filter.
    pub fn try_add_source_vocabulary(
        mut self,
        source_vocabulary: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.source_vocabularies.push(source_vocabulary.try_into()?);
        Ok(self)
    }

    /// Sets the matching strategy.
    pub fn search_type(mut self, search_type: SearchType) -> Self {
        self.search_type = search_type;
        self
    }

    /// Replaces the semantic-type filters.
    pub fn semantic_types(mut self, semantic_types: Vec<DisplayName>) -> Self {
        self.semantic_types = semantic_types;
        self
    }

    /// Adds a semantic-type filter.
    pub fn try_add_semantic_type(
        mut self,
        semantic_type: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.semantic_types.push(semantic_type.try_into()?);
        Ok(self)
    }

    /// Replaces the semantic-group filters.
    pub fn semantic_groups(mut self, semantic_groups: Vec<DisplayName>) -> Self {
        self.semantic_groups = semantic_groups;
        self
    }

    /// Adds a semantic-group filter.
    pub fn try_add_semantic_group(
        mut self,
        semantic_group: impl TryInto<DisplayName, Error = EmptyString>,
    ) -> Result<Self, EmptyString> {
        self.semantic_groups.push(semantic_group.try_into()?);
        Ok(self)
    }

    /// Sets whether all query terms are required or partial matches are
    /// allowed.
    pub fn term_match(mut self, term_match: TermMatch) -> Self {
        self.term_match = term_match;
        self
    }

    /// Sets the maximum number of results requested for a page.
    pub fn page_size(mut self, page_size: NonZeroU64) -> Self {
        self.page_size = page_size;
        self
    }

    /// Sets the one-based page number to request.
    pub fn page_number(mut self, page_number: NonZeroU64) -> Self {
        self.page_number = page_number;
        self
    }

    /// Builds the search request.
    ///
    /// Page size and number are already valid because they are represented by
    /// [`NonZeroU64`].
    pub fn build(self) -> Request {
        let page = core::PageRequest::new(self.page_size, self.page_number);

        Request {
            term: self.term,
            input_type: self.input_type,
            return_id_type: self.return_id_type,
            obsolete: self.obsolete,
            suppressible: self.suppressible,
            source_vocabularies: self.source_vocabularies,
            search_type: self.search_type,
            semantic_types: self.semantic_types,
            semantic_groups: self.semantic_groups,
            term_match: self.term_match,
            page,
        }
    }

    /// Return a concept search.
    ///
    /// The method override return id type to [ReturnIdType::Concept] before creation of [Search]
    /// object.
    pub fn concept<'a>(
        self,
        config: &'a Config,
    ) -> Search<'a, WithoutClient, return_type::Concept> {
        let request = self.return_id_type(ReturnIdType::Concept).build();
        Search::<WithoutClient, _>::new(config, request)
    }
}

/// A validated set of parameters for a UMLS Metathesaurus search.
///
/// Construct requests with [`Request::builder`] and [`Builder::build`].
pub struct Request {
    term: Term,
    input_type: InputType,
    return_id_type: ReturnIdType,
    obsolete: Inclusion,
    suppressible: Inclusion,
    source_vocabularies: Vec<DisplayName>,
    search_type: SearchType,
    semantic_types: Vec<DisplayName>,
    semantic_groups: Vec<DisplayName>,
    term_match: TermMatch,
    page: core::PageRequest,
}

impl Request {
    /// Creates a request builder for the supplied search term.
    pub fn builder(term: Term) -> Builder {
        Builder::new(term)
    }

    /// Returns the search term.
    pub fn term(&self) -> &Term {
        &self.term
    }

    /// Returns how the search term is interpreted.
    pub fn input_type(&self) -> InputType {
        self.input_type
    }

    /// Returns whether matches on obsolete terms are included.
    pub fn obsolete(&self) -> Inclusion {
        self.obsolete
    }

    /// Returns whether matches on suppressible terms are included.
    pub fn suppressible(&self) -> Inclusion {
        self.suppressible
    }

    /// Returns the source vocabularies used to restrict the search.
    pub fn source_vocabularies(&self) -> &[DisplayName] {
        &self.source_vocabularies
    }

    /// Returns the matching strategy used by the search.
    pub fn search_type(&self) -> SearchType {
        self.search_type
    }

    /// Returns the semantic types used to restrict the search.
    pub fn semantic_types(&self) -> &[DisplayName] {
        &self.semantic_types
    }

    /// Returns the semantic groups used to restrict the search.
    pub fn semantic_groups(&self) -> &[DisplayName] {
        &self.semantic_groups
    }

    /// Returns whether all query terms or partial terms may match.
    pub fn term_match(&self) -> TermMatch {
        self.term_match
    }

    /// Returns the request's pagination parameters.
    pub fn page(&self) -> &core::PageRequest {
        &self.page
    }

    #[cfg(feature = "reqwest")]
    pub(crate) fn page_mut(&mut self) -> &mut core::PageRequest {
        &mut self.page
    }

    /// Returns the identifier type requested for search results.
    pub fn return_id_type(&self) -> ReturnIdType {
        self.return_id_type
    }
}

/// The search return type.
pub mod return_type {
    /// A Zero cost object for Concept search return type.
    pub struct Concept;
}

/// An executable UMLS search associated with a client configuration.
///
/// `T` is a zero-sized marker describing the requested result type.
pub struct Search<'a, C, T> {
    client: C,
    config: &'a Config,
    request: Request,
    _value: PhantomData<T>,
}

impl<'a, C, T> Search<'a, C, T> {
    fn new(config: &'a Config, request: Request) -> Search<'a, WithoutClient, T> {
        Search {
            client: WithoutClient,
            config,
            request,
            _value: PhantomData,
        }
    }

    #[cfg(feature = "reqwest")]
    pub(crate) fn request_mut(&mut self) -> &mut Request {
        &mut self.request
    }

    /// Return the reference of client.
    pub fn client(&self) -> &C {
        &self.client
    }
}

impl<'a, T> Search<'a, WithoutClient, T> {
    /// Attaches an owned HTTP client without sending a request.
    ///
    /// Execution requires the `reqwest` feature and a `reqwest::Client`.
    /// Clone an existing client to share its connection pool across requests.
    pub fn with_client<C>(self, client: C) -> Search<'a, C, T> {
        Search {
            client,
            config: self.config,
            request: self.request,
            _value: PhantomData,
        }
    }
}

#[cfg(feature = "reqwest")]
impl<'a> Search<'a, umls_http::reqwest::Client, return_type::Concept> {
    /// Executes a search that returns concept identifiers.
    ///
    /// The returned response can be converted into content with
    /// [`Response::into_content`] or into a navigable page with
    /// [`Response::into_page`].
    ///
    /// # Errors
    ///
    /// Returns [`SearchError`] when URL construction, transport, HTTP status
    /// validation, or response decoding fails.
    pub async fn get(
        self,
    ) -> Result<
        Response<SearchResponse, Search<'a, umls_http::reqwest::Client, return_type::Concept>>,
        SearchError<umls_http::reqwest::Error>,
    > {
        use umls_http::WithoutClient;

        let body = raw::Search::<WithoutClient>::default()
            .with_client(self.client.clone())
            .search(self.config, &self.request)
            .await
            .map_err(SearchError::Raw)?;

        Ok(Response::new(body, self))
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// An error produced while executing a high-level search.
pub enum SearchError<T> {
    /// The underlying raw search operation failed.
    #[error(transparent)]
    Raw(#[from] raw::Error<T>),

    /// The underlying raw pagination failer.
    #[error(transparent)]
    Page(#[from] core::PageError),
}

/// An owned collection of converted results.
#[derive(Debug)]
pub struct Content<T> {
    pub(crate) list: Vec<T>,
}

impl<T> Content<T> {
    /// Creates content from an owned result list.
    pub(crate) fn new(list: Vec<T>) -> Self {
        Self { list }
    }

    /// Returns the results as a slice in API-provided order.
    pub fn list(&self) -> &[T] {
        &self.list
    }
}

impl<T> AsRef<[T]> for Content<T> {
    fn as_ref(&self) -> &[T] {
        &self.list
    }
}

impl<U> From<Vec<U>> for Content<U> {
    fn from(value: Vec<U>) -> Self {
        Content::new(value)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Content<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let list = Vec::<T>::deserialize(deserializer)?;

        Ok(Content::new(list))
    }
}

/// One page of content together with pagination state.
///
/// `C` retains the client needed to request adjacent pages.
pub struct Page<T, C> {
    content: T,
    size: NonZeroU64,
    number: NonZeroU64,
    count: Option<NonZeroU64>,
    pub(crate) client: C,
}

impl<T, C> Page<T, C> {
    /// Creates a page from content, pagination metadata, and its client.
    ///
    /// A `None` count represents the zero-result response used by the UMLS API.
    pub(crate) fn new(
        content: T,
        size: NonZeroU64,
        number: NonZeroU64,
        count: Option<NonZeroU64>,
        client: C,
    ) -> Self {
        Self {
            content,
            size,
            number,
            count,
            client,
        }
    }

    /// Returns the maximum number of results requested for this page.
    pub fn size(&self) -> NonZeroU64 {
        self.size
    }

    /// Returns this page's content.
    pub fn content(&self) -> &T {
        &self.content
    }

    /// Returns the one-based page number.
    pub fn number(&self) -> NonZeroU64 {
        self.number
    }

    /// Returns the total number of matching records.
    ///
    /// `None` means the response contained no matching records. The UMLS API
    /// omits `recCount` when the count is zero.
    pub fn count(&self) -> Option<NonZeroU64> {
        self.count
    }

    /// Returns the number of non-empty result pages.
    ///
    /// `None` means there are no matching records.
    pub fn total_pages(&self) -> Option<NonZeroU64> {
        self.count.map(|x| x.div_ceil(self.size))
    }

    /// Returns `true` when a previous page exists.
    pub fn has_previous(&self) -> bool {
        self.number().get() > 1
    }

    /// Returns `true` when a next page exists.
    ///
    /// The function will return false if the count is [`Option::None`].
    pub fn has_next(&self) -> bool {
        self.total_pages()
            .is_some_and(|total_pages| self.number() < total_pages)
    }

    /// Returns the retained client or search state.
    pub fn client(&self) -> &C {
        &self.client
    }

    #[cfg(feature = "reqwest")]
    pub(crate) fn client_mut(&mut self) -> &mut C {
        &mut self.client
    }
}

/// A raw response paired with the client state that produced it.
pub struct Response<T, C> {
    pub(crate) body: T,
    pub(crate) client: C,
}

impl<T, C> Response<T, C> {
    #[cfg(feature = "reqwest")]
    /// Creates a response wrapper from its body and retained client state.
    pub(crate) fn new(body: T, client: C) -> Self {
        Self { body, client }
    }
}

#[cfg(feature = "reqwest")]
fn convert_raw_search_to_concept_snapshot(
    list: Vec<RawSearchDto>,
) -> Result<Vec<ConceptSnapshot>, ConceptSnapshotError> {
    list.into_iter().map(TryFrom::try_from).collect()
}

#[cfg(feature = "reqwest")]
impl<'a> Response<SearchResponse, Search<'a, umls_http::reqwest::Client, return_type::Concept>> {
    /// Converts raw search records into concept snapshots.
    ///
    /// # Errors
    ///
    /// Returns [`ConceptSnapshotError`] if any raw result lacks required
    /// concept fields or contains an invalid CUI.
    pub fn into_content(self) -> Result<Content<ConceptSnapshot>, ConceptSnapshotError> {
        let objects = convert_raw_search_to_concept_snapshot(self.body.result.results)?;
        Ok(Content::new(objects))
    }

    /// Converts the response into a navigable page of concept snapshots.
    ///
    /// # Errors
    ///
    /// Returns [`PageError::Custom`] if any result cannot be converted into a
    /// [`ConceptSnapshot`].
    pub fn into_page(
        self,
    ) -> PageResult<
        Page<
            Content<ConceptSnapshot>,
            Search<'a, umls_http::reqwest::Client, return_type::Concept>,
        >,
        ConceptSnapshotError,
    > {
        let count = self.body.result.rec_count.and_then(NonZeroU64::new);
        let size = self.body.page_size;
        let number = self.body.page_number;

        let objects = convert_raw_search_to_concept_snapshot(self.body.result.results)
            .map_err(PageError::Custom)?;
        let content = Content::new(objects);

        Ok(Page::new(content, size, number, count, self.client))
    }
}

#[cfg(feature = "reqwest")]
impl<'a>
    Page<Content<ConceptSnapshot>, Search<'a, umls_http::reqwest::Client, return_type::Concept>>
{
    /// Consumes this page and returns a search request for the next page.
    ///
    /// Preserves the attached HTTP client and search filters without making a network
    /// request. Call `get().await` on the returned request to fetch its results.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use umls_http::reqwest;
    /// use umls_client::{config::Config, search::Request};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
    /// let page = Request::builder("cough".parse()?)
    ///     .concept(&config)
    ///     .with_client(reqwest::Client::default())
    ///     .get().await?.into_page()?;
    /// if page.has_next() {
    ///     let next_request = page.next()?;
    ///     let next_page = next_request.get().await?.into_page()?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`PageError::NoNextPage`] at the final page. A page-number
    /// adjustment failure is wrapped in [`PageError::Custom`] as [`SearchError::Page`].
    /// HTTP and response-decoding errors are returned by the subsequent `get()` call.
    pub fn next(
        mut self,
    ) -> Result<
        Search<'a, umls_http::reqwest::Client, return_type::Concept>,
        PageError<SearchError<umls_http::reqwest::Error>>,
    > {
        if !self.has_next() {
            return Err(PageError::NoNextPage);
        }

        self.client_mut()
            .request_mut()
            .page_mut()
            .checked_next()
            .map_err(SearchError::Page)
            .map_err(PageError::Custom)?;

        Ok(self.client)
    }

    /// Consumes this page and returns a search request for the previous page.
    ///
    /// Preserves the attached HTTP client and search filters without making a network
    /// request. Call `get().await` on the returned request to fetch its results.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use umls_http::reqwest;
    /// use umls_client::{config::Config, search::Request};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
    /// let page = Request::builder("cough".parse()?)
    ///     .concept(&config)
    ///     .with_client(reqwest::Client::default())
    ///     .get().await?.into_page()?;
    /// if page.has_next() {
    ///     let page = page.next()?.get().await?.into_page()?;
    ///     let previous_request = page.previous()?;
    ///     let previous_page = previous_request.get().await?.into_page()?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`PageError::NoPreviousPage`] at the first page. A page-number
    /// adjustment failure is wrapped in [`PageError::Custom`] as [`SearchError::Page`].
    /// HTTP and response-decoding errors are returned by the subsequent `get()` call.
    pub fn previous(
        mut self,
    ) -> PageResult<
        Search<'a, umls_http::reqwest::Client, return_type::Concept>,
        SearchError<umls_http::reqwest::Error>,
    > {
        if !self.has_previous() {
            return Err(PageError::NoPreviousPage);
        }

        self.client_mut()
            .request_mut()
            .page_mut()
            .checked_previous()
            .map_err(SearchError::Page)
            .map_err(PageError::Custom)?;

        Ok(self.client)
    }
}

/// A page result object.
pub type PageResult<T, E> = Result<T, PageError<E>>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// An error produced while converting or navigating a search page.
pub enum PageError<E: std::error::Error> {
    /// The current page is already the first page.
    #[error("Already at the first page")]
    NoPreviousPage,

    /// The current page is already the final page.
    #[error("Already at the last page")]
    NoNextPage,

    /// Conversion or client operation `E` failed.
    #[error("Custom error: {0}")]
    Custom(E),
}

/// A compact concept representation returned by a search.
///
/// With the `reqwest` feature, pass an owned snapshot to [`concept::Info::new`],
/// attach a client with `with_client()`, and call `fetch(&config)` to retrieve
/// the full concept. [`Self::info`] provides a borrowed resource reference.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "reqwest")]
/// use umls_http::reqwest;
/// use umls_client::{concept::Info, config::Config, search::ConceptSnapshot};
/// # #[cfg(feature = "reqwest")]
/// async fn fetch_concept(
///     snapshot: ConceptSnapshot,
///     config: &Config,
///     client: reqwest::Client,
/// ) -> Result<(), Box<dyn std::error::Error>> {
///     let concept = Info::new(snapshot).with_client(client).fetch(config).await?;
///     println!("{}", concept.name);
///     Ok(())
/// }
/// ```
pub struct ConceptSnapshot {
    cui: crate::core::cui::Cui,
    root_source: DisplayName,
    pub(crate) info: concept::Info<umls_http::WithoutClient, Url>,
    name: DisplayName,
    semantic_types: Vec<DisplayName>,
}

impl ConceptSnapshot {
    #[cfg(feature = "reqwest")]
    /// Creates a concept snapshot from validated search-result fields.
    pub(crate) fn new(
        cui: crate::core::cui::Cui,
        root_source: DisplayName,
        info: concept::Info<umls_http::WithoutClient, Url>,
        name: DisplayName,
        semantic_types: Vec<DisplayName>,
    ) -> Self {
        Self {
            cui,
            root_source,
            info,
            name,
            semantic_types,
        }
    }

    /// Returns the concept's validated CUI as a string.
    pub fn cui(&self) -> &str {
        self.cui.as_ref()
    }

    /// Returns the source vocabulary that supplied the result.
    pub fn root_source(&self) -> &str {
        self.root_source.as_ref()
    }

    /// Returns the resource reference used to retrieve the full concept.
    pub fn info(&self) -> &concept::Info<umls_http::WithoutClient, Url> {
        &self.info
    }

    /// Returns the semantic types assigned to this search result.
    pub fn semantic_types(&self) -> &[DisplayName] {
        &self.semantic_types
    }

    /// Returns the human-readable concept name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl std::fmt::Display for ConceptSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.cui())
    }
}

#[cfg(feature = "reqwest")]
impl TryFrom<RawSearchDto> for ConceptSnapshot {
    type Error = ConceptSnapshotError;

    fn try_from(value: RawSearchDto) -> Result<Self, Self::Error> {
        let cui: crate::core::cui::Cui = value
            .ui
            .as_str()
            .parse()
            .map_err(ConceptSnapshotError::Cui)?;
        let root_source = value
            .root_source
            .ok_or(ConceptSnapshotError::MissingRootSource)?;
        let semantic_types = value
            .semantic_types
            .ok_or(ConceptSnapshotError::MissingSemanticTypes)?;
        let info = concept::Info::new(value.uri);
        let name = value.name;

        Ok(ConceptSnapshot::new(
            cui,
            root_source,
            info,
            name,
            semantic_types,
        ))
    }
}

#[derive(Debug, thiserror::Error)]
/// An error produced while converting a raw search record into a concept snapshot.
pub enum ConceptSnapshotError {
    /// The result contained an invalid concept identifier.
    #[error("CUI format error: {0}")]
    Cui(crate::core::cui::Error),

    /// The result omitted its root source.
    #[error("Missing root source")]
    MissingRootSource,

    /// The result omitted its semantic types.
    #[error("Missing semantic types")]
    MissingSemanticTypes,
}

#[cfg(test)]
mod tests {

    use crate::search::core::ReturnIdType;

    use super::*;

    #[test]
    fn successful_to_str_input_type_conversion() {
        fn assert_input_type(input_type: InputType, value: &str) {
            assert_eq!(input_type.as_str(), value);
        }

        assert_input_type(InputType::Atom, "atom");
        assert_input_type(InputType::Code, "code");
        assert_input_type(InputType::SourceConcept, "sourceConcept");
        assert_input_type(InputType::SourceDescriptor, "sourceDescriptor");
        assert_input_type(InputType::SourceUi, "sourceUi");
        assert_input_type(InputType::Tty, "tty");
    }

    #[test]
    fn successful_as_ref_input_type_conversion() {
        fn assert_input_type(input_type: InputType, value: &str) {
            assert_eq!(input_type.as_ref(), value);
        }

        assert_input_type(InputType::Atom, "atom");
        assert_input_type(InputType::Code, "code");
        assert_input_type(InputType::SourceConcept, "sourceConcept");
        assert_input_type(InputType::SourceDescriptor, "sourceDescriptor");
        assert_input_type(InputType::SourceUi, "sourceUi");
        assert_input_type(InputType::Tty, "tty");
    }

    #[test]
    fn successful_to_str_return_id_type_conversion() {
        fn assert_return_id_type(return_id_type: ReturnIdType, value: &str) {
            assert_eq!(return_id_type.as_str(), value);
        }

        assert_return_id_type(ReturnIdType::Concept, "concept");
        assert_return_id_type(ReturnIdType::Aui, "aui");
        assert_return_id_type(ReturnIdType::Code, "code");
        assert_return_id_type(ReturnIdType::SourceConcept, "sourceConcept");
        assert_return_id_type(ReturnIdType::SourceDescriptor, "sourceDescriptor");
        assert_return_id_type(ReturnIdType::SourceUi, "sourceUi");
    }

    #[test]
    fn successful_as_ref_return_id_type_conversion() {
        fn assert_return_id_type(return_id_type: ReturnIdType, value: &str) {
            assert_eq!(return_id_type.as_ref(), value);
        }

        assert_return_id_type(ReturnIdType::Concept, "concept");
        assert_return_id_type(ReturnIdType::Aui, "aui");
        assert_return_id_type(ReturnIdType::Code, "code");
        assert_return_id_type(ReturnIdType::SourceConcept, "sourceConcept");
        assert_return_id_type(ReturnIdType::SourceDescriptor, "sourceDescriptor");
        assert_return_id_type(ReturnIdType::SourceUi, "sourceUi");
    }

    #[test]
    fn successful_to_str_search_type_conversion() {
        fn assert_search_type(search_type: SearchType, value: &str) {
            assert_eq!(search_type.as_str(), value);
        }

        assert_search_type(SearchType::Exact, "exact");
        assert_search_type(SearchType::Words, "words");
        assert_search_type(SearchType::LeftTruncation, "leftTruncation");
        assert_search_type(SearchType::RightTruncation, "rightTruncation");
        assert_search_type(SearchType::NormalizedString, "normalizedString");
        assert_search_type(SearchType::NormalizedWords, "normalizedWords");
    }

    #[test]
    fn successful_as_ref_search_type_conversion() {
        fn assert_search_type(search_type: SearchType, value: &str) {
            assert_eq!(search_type.as_ref(), value);
        }

        assert_search_type(SearchType::Exact, "exact");
        assert_search_type(SearchType::Words, "words");
        assert_search_type(SearchType::LeftTruncation, "leftTruncation");
        assert_search_type(SearchType::RightTruncation, "rightTruncation");
        assert_search_type(SearchType::NormalizedString, "normalizedString");
        assert_search_type(SearchType::NormalizedWords, "normalizedWords");
    }

    #[test]
    fn create_trimmed_term() -> Result<(), Box<dyn std::error::Error>> {
        let values = vec![" term", "term   ", "term", "   term   "];

        for value in values {
            let term = Term::try_new(value)?;
            assert_eq!(term.as_str(), "term");
        }

        Ok(())
    }

    #[test]
    fn reject_empty_term() {
        assert!(Term::try_new("").is_err());
    }

    #[test]
    fn rejects_whitespace_only_term() {
        assert!(Term::try_new(" \t\n ").is_err());
    }

    #[test]
    fn parses_term_from_string_slice() {
        let term: Term = "  coughing  ".parse().unwrap();

        assert_eq!(term.as_str(), "coughing");
    }

    #[test]
    fn formats_term_for_display() {
        let term = Term::try_new("coughing").unwrap();

        assert_eq!(term.to_string(), "coughing");
    }

    #[test]
    fn creates_display_name() {
        let name = DisplayName::try_new("Diabetes mellitus").unwrap();

        assert_eq!(name.as_str(), "Diabetes mellitus");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        for value in [
            "Diabetes mellitus ",
            " Diabetes mellitus",
            "  Diabetes mellitus  ",
            "\tDiabetes mellitus\n",
        ] {
            let name = DisplayName::try_new(value).unwrap();

            assert_eq!(name.as_str(), "Diabetes mellitus");
        }
    }

    #[test]
    fn rejects_empty_string() {
        assert!(DisplayName::try_new("").is_err());
    }

    #[test]
    fn rejects_whitespace_only_string() {
        for value in [" ", "   ", "\t", "\n", " \t\n "] {
            assert!(DisplayName::try_new(value).is_err());
        }
    }

    #[test]
    fn parses_from_string_slice() {
        let name: DisplayName = "  Sign or Symptom  ".parse().unwrap();

        assert_eq!(name.as_str(), "Sign or Symptom");
    }

    #[test]
    fn tries_from_owned_string() {
        let name = DisplayName::try_from("  Disorders  ".to_owned()).unwrap();

        assert_eq!(name.as_str(), "Disorders");
    }

    #[test]
    fn tries_from_borrowed_string() {
        let name = DisplayName::try_from("  SNOMEDCT_US  ").unwrap();

        assert_eq!(name.as_str(), "SNOMEDCT_US");
    }

    #[test]
    fn exposes_value_through_as_ref() {
        let name = DisplayName::try_new("Finding").unwrap();

        assert_eq!(name.as_ref(), "Finding");
    }

    #[test]
    fn formats_value_for_display() {
        let name = DisplayName::try_new("Finding").unwrap();

        assert_eq!(name.to_string(), "Finding");
    }

    #[test]
    fn returns_underlying_string() {
        let name = DisplayName::try_new("Finding").unwrap();

        assert_eq!(name.into_inner(), "Finding");
    }

    #[test]
    fn creates_page_with_size_and_number() {
        let page =
            core::PageRequest::new(NonZeroU64::new(25).unwrap(), NonZeroU64::new(1).unwrap());

        assert_eq!(page.size().get(), 25);
        assert_eq!(page.number().get(), 1);
    }

    #[test]
    fn builds_request_with_valid_page() -> Result<(), Box<dyn std::error::Error>> {
        let term = Term::try_new("cough")?;

        let request = Request::builder(term).build();

        assert_eq!(request.page().size(), DEFAULT_PAGE_SIZE);
        assert_eq!(request.page().number(), DEFAULT_PAGE_NUMBER);

        Ok(())
    }
}
