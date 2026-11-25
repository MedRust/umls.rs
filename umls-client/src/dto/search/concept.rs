use std::marker::PhantomData;

use crate::dto::{
    display_name::DisplayName,
    search::{Builder, Input, SearchType, return_type},
};

pub struct Config<U> {
    input: Input,
    include_obsolete: bool,
    include_suppressible: bool,
    source_vocabularies: Vec<DisplayName>,
    search_type: SearchType,
    semantic_types: Vec<DisplayName>,
    semantic_groups: Vec<DisplayName>,
    _partial_search: PhantomData<U>,
}

impl<U> Config<U> {
    pub(crate) fn new(
        input: Input,
        include_obsolete: bool,
        include_suppressible: bool,
        source_vocabularies: Vec<DisplayName>,
        search_type: SearchType,
        semantic_types: Vec<DisplayName>,
        semantic_groups: Vec<DisplayName>,
    ) -> Self {
        Self {
            input,
            include_obsolete,
            include_suppressible,
            source_vocabularies,
            search_type,
            semantic_types,
            semantic_groups,
            _partial_search: PhantomData,
        }
    }

    pub fn builder() -> Builder<return_type::Concept, U> {
        Builder::new()
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn include_obsolete(&self) -> bool {
        self.include_obsolete
    }

    pub fn include_suppressible(&self) -> bool {
        self.include_suppressible
    }

    pub fn source_vocabularies(&self) -> &[DisplayName] {
        &self.source_vocabularies
    }

    pub fn search_type(&self) -> SearchType {
        self.search_type
    }

    pub fn semantic_types(&self) -> &[DisplayName] {
        &self.semantic_types
    }

    pub fn semantic_groups(&self) -> &[DisplayName] {
        &self.semantic_groups
    }
}
