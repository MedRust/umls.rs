//! High-level search results, conversion, and pagination.

use std::{marker::PhantomData, num::NonZeroU64};

use umls_http::WithoutClient;

use crate::dto::batch::BatchError;

/// Shared validation primitives used by public value types.
pub mod core;

#[cfg(feature = "reqwest")]
use crate::{
    dto::{self, content::Content, search::partial_search::NoPartial},
    types,
};

/// An executable UMLS search associated with a client configuration.
///
/// `T` is a zero-sized marker describing the requested result type.
pub struct Search<C, T, V> {
    client: C,
    value: V,
    _type: PhantomData<T>,
}

impl<C, T, V> Search<C, T, V> {
    /// Return the reference of client.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Returns value.
    pub fn value(&self) -> &V {
        &self.value
    }

    #[cfg(feature = "reqwest")]
    fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

impl<T, V> Search<WithoutClient, T, V> {
    /// Create new search object without client.
    pub fn new(value: V) -> Search<WithoutClient, T, V> {
        Search {
            client: WithoutClient,
            value,
            _type: PhantomData,
        }
    }

    /// Attaches an owned HTTP client without sending a request.
    ///
    /// Execution requires the `reqwest` feature and a `reqwest::Client`.
    /// Clone an existing client to share its connection pool across requests.
    pub fn with_client<C>(self, client: C) -> Search<C, T, V> {
        Search {
            client,
            value: self.value,
            _type: PhantomData,
        }
    }
}

#[cfg(feature = "reqwest")]
impl
    Search<
        umls_http::reqwest::Client,
        types::Concept,
        dto::batch::Batch<dto::search::concept::Config<NoPartial>>,
    >
{
    /// Executes a search that returns concept identifiers.
    ///
    /// The returned response can be converted into content with
    /// [`Response::into_content`] or into a navigable page with
    /// [`Response::into_page`].
    pub async fn fetch(
        self,
        config: &crate::config::Config,
    ) -> Result<
        Response<dto::page::Page<Content<dto::search::ConceptResponse>>, Self>,
        Box<dyn std::error::Error>,
    > {
        let url = config.search_url(self.value()).url();

        let body = self
            .client
            .clone()
            .with_type::<umls_http::reqwest::Json>()
            .get::<_, dto::search::SearchPage>(url)
            .await
            .map(Into::<dto::page::Page<Content<dto::search::RawSearch>>>::into)?
            .try_iter_map(TryInto::<dto::search::ConceptResponse>::try_into)?;

        Ok(Response::new(body, self))
    }
}

/// One page of content together with pagination state.
///
/// `C` retains the client needed to request adjacent pages.
pub struct Page<T, C> {
    content: T,
    size: NonZeroU64,
    number: NonZeroU64,
    count: NonZeroU64,
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
        count: NonZeroU64,
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

    /// Returns the total number pages.
    ///
    /// `None` means the response contained no matching records. The UMLS API
    /// omits `recCount` when the count is zero.
    pub fn count(&self) -> NonZeroU64 {
        self.count
    }

    /// Returns `true` when a previous page exists.
    pub fn has_previous(&self) -> bool {
        self.number().get() > 1
    }

    /// Returns `true` when a next page exists.
    ///
    /// The function will return false if the count is [`Option::None`].
    pub fn has_next(&self) -> bool {
        self.number < self.count
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
    pub fn new(body: T, client: C) -> Self {
        Self { body, client }
    }
}

#[cfg(feature = "reqwest")]
impl
    Page<
        Content<dto::search::ConceptResponse>,
        Search<
            umls_http::reqwest::Client,
            types::Concept,
            dto::batch::Batch<dto::search::concept::Config<dto::search::partial_search::NoPartial>>,
        >,
    >
{
    /**
    Consumes this page and returns a search request for the next page.

    Preserves the attached HTTP client and search filters without making a network
    request.

    # Example

    ```no_run
    use umls_http::{reqwest, WithoutClient};
    use umls_client::{config::Config, search::Search, dto::{search::{concept, Input}, batch::Batch}, types};
    # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
    let concept = concept::Config::builder()
        .input(Input::Atom("cough".parse()?))
        .build()?;

    let batch = Batch::new(25.try_into()?, 1.try_into()?, concept);

    let page = Search::<WithoutClient, types::Concept, _>::new(batch)
        .with_client(reqwest::Client::default())
        .fetch(&config).await?.into_page();

    if page.has_next() {
        let next_request = page.next()?;
        let next_page = next_request.fetch(&config).await?.into_page();
    }
    # Ok(())
    # }
    ```

    # Errors

    Returns [`PageError::NoNextPage`] at the final page.
    */
    pub fn next(
        mut self,
    ) -> Result<
        Search<
            umls_http::reqwest::Client,
            types::Concept,
            dto::batch::Batch<dto::search::concept::Config<dto::search::partial_search::NoPartial>>,
        >,
        PageError,
    > {
        if !self.has_next() {
            return Err(PageError::NoNextPage);
        }

        self.client_mut().value_mut().checked_next()?;

        Ok(self.client)
    }

    /**
     * Consumes this page and returns a search request for the previous page.
     *
     * Preserves the attached HTTP client and search filters without making a network
     * request.
     *
     * # Errors
     * Returns [`PageError::NoPreviousPage`] at the first page.
     */
    pub fn previous(
        mut self,
    ) -> PageResult<
        Search<
            umls_http::reqwest::Client,
            types::Concept,
            dto::batch::Batch<dto::search::concept::Config<dto::search::partial_search::NoPartial>>,
        >,
    > {
        if !self.has_previous() {
            return Err(PageError::NoPreviousPage);
        }

        self.client_mut().value_mut().checked_previous()?;

        Ok(self.client)
    }
}

/// A page result object.
pub type PageResult<T> = Result<T, PageError>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// An error produced while converting or navigating a search page.
pub enum PageError {
    /// The current page is already the first page.
    #[error("Already at the first page")]
    NoPreviousPage,

    /// The current page is already the final page.
    #[error("Already at the last page")]
    NoNextPage,

    /// Batch error.
    #[error("Batch Error: {0}")]
    Batch(#[source] BatchError),
}

impl From<BatchError> for PageError {
    fn from(value: BatchError) -> Self {
        PageError::Batch(value)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[cfg(feature = "reqwest")]
    use crate::{
        dto::{self, content::Content},
        types,
    };

    #[test]
    fn has_next_true_on_page_count_greater_than_page_number() {
        let page = Page::new(
            "",
            10.try_into().unwrap(),
            1.try_into().unwrap(),
            2.try_into().unwrap(),
            "",
        );

        assert!(page.has_next());
    }

    #[test]
    fn has_next_false_on_page_count_equal_to_page_number() {
        let page = Page::new(
            "",
            10.try_into().unwrap(),
            2.try_into().unwrap(),
            2.try_into().unwrap(),
            "",
        );

        assert!(!page.has_next());
    }

    #[test]
    fn has_next_false_on_page_count_less_than_page_number() {
        let page = Page::new(
            "",
            10.try_into().unwrap(),
            3.try_into().unwrap(),
            2.try_into().unwrap(),
            "",
        );

        assert!(!page.has_next());
    }

    #[test]
    fn has_previous_true_on_page_number_greater_then_one() {
        let page = Page::new(
            "",
            10.try_into().unwrap(),
            2.try_into().unwrap(),
            2.try_into().unwrap(),
            "",
        );

        assert!(page.has_previous());
    }

    #[test]
    fn has_previous_false_on_page_number_equal_to_one() {
        let page = Page::new(
            "",
            10.try_into().unwrap(),
            1.try_into().unwrap(),
            2.try_into().unwrap(),
            "",
        );

        assert!(!page.has_previous());
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn change_page_number_in_batch_on_next_call() {
        let value =
            dto::search::concept::Config::<dto::search::partial_search::NoPartial>::builder()
                .input(dto::search::Input::Atom("C0010200".try_into().unwrap()))
                .build()
                .unwrap();
        let value = dto::batch::Batch::new(10.try_into().unwrap(), 1.try_into().unwrap(), value);
        let client = Search::<_, types::Concept, _>::new(value)
            .with_client(umls_http::reqwest::Client::default());
        let page = Page::new(
            Content::<dto::search::ConceptResponse>::new(vec![]),
            10.try_into().unwrap(),
            1.try_into().unwrap(),
            10.try_into().unwrap(),
            client,
        );

        assert!(page.has_next());
        let client = page.next().unwrap();
        assert_eq!(client.value().number().get(), 2);
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn next_failed_with_no_next_page_error_on_has_next_false() {
        let value =
            dto::search::concept::Config::<dto::search::partial_search::NoPartial>::builder()
                .input(dto::search::Input::Atom("C0010200".try_into().unwrap()))
                .build()
                .unwrap();
        let value = dto::batch::Batch::new(10.try_into().unwrap(), 1.try_into().unwrap(), value);
        let client = Search::<_, types::Concept, _>::new(value)
            .with_client(umls_http::reqwest::Client::default());
        let page = Page::new(
            Content::<dto::search::ConceptResponse>::new(vec![]),
            10.try_into().unwrap(),
            1.try_into().unwrap(),
            1.try_into().unwrap(),
            client,
        );

        assert!(!page.has_next());
        let result = page.next();
        assert!(matches!(result, Err(PageError::NoNextPage)));
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn reduce_page_number_by_one_in_batch_on_previous_call() {
        let value =
            dto::search::concept::Config::<dto::search::partial_search::NoPartial>::builder()
                .input(dto::search::Input::Atom("C0010200".try_into().unwrap()))
                .build()
                .unwrap();
        let value = dto::batch::Batch::new(10.try_into().unwrap(), 2.try_into().unwrap(), value);
        let client = Search::<_, types::Concept, _>::new(value)
            .with_client(umls_http::reqwest::Client::default());
        let page = Page::new(
            Content::<dto::search::ConceptResponse>::new(vec![]),
            10.try_into().unwrap(),
            2.try_into().unwrap(),
            10.try_into().unwrap(),
            client,
        );

        assert!(page.has_previous());
        let client = page.previous().unwrap();
        assert_eq!(client.value().number().get(), 1);
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn previous_failed_with_no_previous_page_error_on_has_previous_false() {
        let value =
            dto::search::concept::Config::<dto::search::partial_search::NoPartial>::builder()
                .input(dto::search::Input::Atom("C0010200".try_into().unwrap()))
                .build()
                .unwrap();
        let value = dto::batch::Batch::new(10.try_into().unwrap(), 1.try_into().unwrap(), value);
        let client = Search::<_, types::Concept, _>::new(value)
            .with_client(umls_http::reqwest::Client::default());
        let page = Page::new(
            Content::<dto::search::ConceptResponse>::new(vec![]),
            10.try_into().unwrap(),
            1.try_into().unwrap(),
            10.try_into().unwrap(),
            client,
        );

        assert!(!page.has_previous());
        let result = page.previous();
        assert!(matches!(result, Err(PageError::NoPreviousPage)));
    }
}
