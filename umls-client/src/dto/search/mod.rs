#![allow(missing_docs)]

pub mod concept;

use std::{marker::PhantomData, num::NonZeroU64, vec};

use serde::Deserialize;
use umls_http::WithoutClient;
use url::Url;

use crate::{
    dto::{self, content::Content, display_name::DisplayName},
    info::Info,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Atom(DisplayName),
    Code(DisplayName),
    SourceConept(DisplayName),
    SourceDescriptor(DisplayName),
    SourceUi(DisplayName),
    Tty(DisplayName),
}

impl Input {
    pub fn text(&self) -> &DisplayName {
        match self {
            Input::Atom(x) => x,
            Input::Code(x) => x,
            Input::SourceConept(x) => x,
            Input::SourceDescriptor(x) => x,
            Input::SourceUi(x) => x,
            Input::Tty(x) => x,
        }
    }

    pub fn input_type(&self) -> &str {
        match self {
            Input::Atom(_) => "atom",
            Input::Code(_) => "code",
            Input::SourceConept(_) => "sourceConcept",
            Input::SourceDescriptor(_) => "sourceDescriptor",
            Input::SourceUi(_) => "sourceUi",
            Input::Tty(_) => "tty",
        }
    }
}

pub mod return_type {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Aui;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Concept;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Code;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SourceConcept;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SourceDescriptor;
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SourceUi;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    Exact,
    Words,
    LeftTruncation,
    RightTruncation,
    NormalizedString,
    NormalizedWords,
}

impl SearchType {
    pub fn as_str(&self) -> &str {
        match self {
            SearchType::Exact => "exact",
            SearchType::Words => "words",
            SearchType::LeftTruncation => "leftTruncation",
            SearchType::RightTruncation => "rightTruncation",
            SearchType::NormalizedString => "normalizedString",
            SearchType::NormalizedWords => "normalizedWords",
        }
    }
}

pub mod partial_search {
    pub struct Partial;
    pub struct NoPartial;
}

pub struct Builder<T, U> {
    input: Option<Input>,
    include_obsolete: Option<bool>,
    include_suppressible: Option<bool>,
    source_vocabularies: Option<Vec<DisplayName>>,
    search_type: Option<SearchType>,
    semantic_types: Option<Vec<DisplayName>>,
    semantic_groups: Option<Vec<DisplayName>>,
    _return: PhantomData<T>,
    _partial_search: PhantomData<U>,
}

impl<T, U> Default for Builder<T, U> {
    fn default() -> Self {
        Self {
            input: None,
            include_obsolete: None,
            include_suppressible: None,
            source_vocabularies: None,
            search_type: None,
            semantic_types: None,
            semantic_groups: None,
            _return: PhantomData,
            _partial_search: PhantomData,
        }
    }
}

impl<T, U> Builder<T, U> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn input(mut self, input: Input) -> Self {
        self.input = Some(input);
        self
    }

    pub fn include_suppressible(mut self, include_suppressible: bool) -> Self {
        self.include_suppressible = Some(include_suppressible);
        self
    }

    pub fn include_obsolete(mut self, include_obsolete: bool) -> Self {
        self.include_obsolete = Some(include_obsolete);
        self
    }

    pub fn source_vocabularies(mut self, source_vocabularies: Vec<DisplayName>) -> Self {
        self.source_vocabularies = Some(source_vocabularies);
        self
    }

    pub fn search_type(mut self, search_type: SearchType) -> Self {
        self.search_type = Some(search_type);
        self
    }
}

impl<U> Builder<return_type::Concept, U> {
    pub fn semantic_types(mut self, semantic_types: Vec<DisplayName>) -> Self {
        self.semantic_types = Some(semantic_types);
        self
    }

    pub fn semantic_groups(mut self, semanetic_groups: Vec<DisplayName>) -> Self {
        self.semantic_groups = Some(semanetic_groups);
        self
    }

    pub fn build(self) -> Result<concept::Config<U>, Error> {
        Ok(concept::Config::new(
            self.input.ok_or(Error::MissingInput)?,
            self.include_obsolete.unwrap_or(false),
            self.include_suppressible.unwrap_or(false),
            self.source_vocabularies.unwrap_or(vec![]),
            self.search_type.unwrap_or(SearchType::Words),
            self.semantic_types.unwrap_or(vec![]),
            self.semantic_groups.unwrap_or(vec![]),
        ))
    }
}

impl From<SearchPage> for dto::page::Page<Content<RawSearch>> {
    fn from(value: SearchPage) -> Self {
        let total_count = value.content.rec_count.map_or(0, |x| x.get());
        let page_size = value.size.get();
        let mut page_count = total_count / page_size;
        page_count += if !total_count.is_multiple_of(page_size) {
            1
        } else {
            0
        };

        let content = Content::new(value.content.results);

        dto::page::Page::new(
            value.size,
            value.number,
            page_count.try_into().unwrap(),
            content,
        )
    }
}

#[derive(Debug)]
pub struct ConceptResponse {
    ui: DisplayName,
    root_source: DisplayName,
    uri: Info<WithoutClient, crate::types::Concept, Url>,
    name: DisplayName,
    semantic_types: Vec<DisplayName>,
}

impl ConceptResponse {
    pub fn ui(&self) -> &DisplayName {
        &self.ui
    }

    pub fn root_source(&self) -> &DisplayName {
        &self.root_source
    }

    pub fn uri(&self) -> &Info<WithoutClient, crate::types::Concept, Url> {
        &self.uri
    }

    pub fn name(&self) -> &DisplayName {
        &self.name
    }

    pub fn semantic_types(&self) -> &[DisplayName] {
        &self.semantic_types
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchPage {
    #[serde(rename = "pageSize")]
    size: NonZeroU64,
    #[serde(rename = "pageNumber")]
    number: NonZeroU64,
    #[serde(rename = "result")]
    content: SearchResult,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SearchResult {
    #[serde(rename = "results")]
    results: Vec<RawSearch>,
    #[serde(rename = "recCount")]
    rec_count: Option<NonZeroU64>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RawSearch {
    #[serde(rename = "ui")]
    id: DisplayName,
    #[serde(rename = "rootSource")]
    root_source: Option<DisplayName>,
    uri: Url,
    name: DisplayName,
    #[serde(rename = "semanticTypes")]
    semantic_types: Option<Vec<DisplayName>>,
}

impl TryFrom<RawSearch> for ConceptResponse {
    type Error = Error;

    fn try_from(value: RawSearch) -> Result<Self, Self::Error> {
        let info = Info::<WithoutClient, crate::types::Concept, _>::new(value.uri);
        let root_source = value.root_source.ok_or(Error::MissingRootSource)?;
        let semantic_types = value.semantic_types.ok_or(Error::MissingSemanticTypes)?;

        Ok(ConceptResponse {
            ui: value.id,
            root_source,
            uri: info,
            name: value.name,
            semantic_types,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing root source.")]
    MissingRootSource,

    #[error("Missing semantic types")]
    MissingSemanticTypes,

    #[error("Missing Input")]
    MissingInput,
}
