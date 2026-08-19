use std::{fmt, str::FromStr};

use thiserror::Error;

/// Specifies how the UMLS search should interpret the search value.
///
/// This corresponds to the `inputType` parameter accepted by the UMLS
/// Metathesaurus search endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    /// Interprets the search value as an atom name or identifier.
    Atom,

    /// Interprets the search value as a code from a source vocabulary.
    ///
    /// When using this variant, the request should normally identify the
    /// relevant source vocabulary.
    Code,

    /// Interprets the search value as a source concept identifier.
    SourceConcept,

    /// Interprets the search value as a source descriptor identifier.
    SourceDescriptor,

    /// Interprets the search value as a general source vocabulary
    /// identifier.
    ///
    /// Use this when it is unknown whether the supplied identifier is a
    /// code, source concept identifier, or source descriptor identifier.
    SourceUi,

    /// Interprets the search value as a term-type identifier.
    ///
    /// This is intended for advanced searches that use a vocabulary's term
    /// type to identify associated codes.
    Tty,
}

impl InputType {
    /// Returns the string representation expected by the UMLS REST API.
    pub fn as_str(&self) -> &str {
        match self {
            InputType::Atom => "atom",
            InputType::Code => "code",
            InputType::SourceConcept => "sourceConcept",
            InputType::SourceDescriptor => "sourceDescriptor",
            InputType::SourceUi => "sourceUi",
            InputType::Tty => "tty",
        }
    }
}

impl AsRef<str> for InputType {
    /// Returns the string representation expected by the UMLS REST API.
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Specifies the identifier type returned by a UMLS search.
///
/// This corresponds to the `returnIdType` parameter accepted by the UMLS
/// Metathesaurus search endpoint.
///
/// The UMLS API uses [`ReturnIdType::Concept`] by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnIdType {
    /// Returns Atom Unique Identifiers (AUIs).
    Aui,

    /// Returns Concept Unique Identifiers (CUIs).
    Concept,

    /// Returns codes assigned by source vocabularies.
    Code,

    /// Returns source concept identifiers (SCUIs).
    SourceConcept,

    /// Returns source descriptor identifiers (SDUIs).
    SourceDescriptor,

    /// Returns source-asserted identifiers.
    ///
    /// Use this when the required source identifier may be a code, source
    /// concept identifier, or source descriptor identifier.
    SourceUi,
}

impl ReturnIdType {
    /// Returns the string representation expected by the UMLS REST API.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Aui => "aui",
            Self::Concept => "concept",
            Self::Code => "code",
            Self::SourceConcept => "sourceConcept",
            Self::SourceDescriptor => "sourceDescriptor",
            Self::SourceUi => "sourceUi",
        }
    }
}

impl AsRef<str> for ReturnIdType {
    /// Returns the string representation expected by the UMLS REST API.
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Specifies the matching strategy used by a UMLS search.
///
/// This corresponds to the `searchType` parameter accepted by the UMLS
/// Metathesaurus search endpoint.
///
/// The UMLS API uses [`SearchType::Words`] by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    /// Returns results containing an exact match for the search value.
    ///
    /// This should normally be used when searching by a source code or
    /// another source-asserted identifier.
    Exact,

    /// Returns results in which all search words occur in a concept name.
    Words,

    /// Returns results containing terms that end with the search value.
    ///
    /// For example, searching for `itis` can match terms such as
    /// `bronchitis`.
    LeftTruncation,

    /// Returns results containing terms that begin with the search value.
    ///
    /// For example, searching for `bronch` can match terms such as
    /// `bronchitis` and `bronchial artery`.
    RightTruncation,

    /// Matches an English search value after normalizing lexical variations.
    ///
    /// Normalization accounts for differences such as letter case and
    /// singular or plural forms.
    NormalizedString,

    /// Matches the normalized words of an English search value.
    ///
    /// This applies lexical normalization to individual words before
    /// comparing them with the Metathesaurus normalized word index.
    NormalizedWords,
}

impl SearchType {
    /// Returns the string representation expected by the UMLS REST API.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Words => "words",
            Self::LeftTruncation => "leftTruncation",
            Self::RightTruncation => "rightTruncation",
            Self::NormalizedString => "normalizedString",
            Self::NormalizedWords => "normalizedWords",
        }
    }
}

impl AsRef<str> for SearchType {
    /// Returns the string representation expected by the UMLS REST API.
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Controls whether a category of UMLS records is included in search results.
///
/// This can represent inclusion options such as `includeObsolete` and
/// `includeSuppressible` in the UMLS Metathesaurus search API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inclusion {
    /// Include matching records in the search results.
    Include,

    /// Exclude matching records from the search results.
    Exclude,
}

impl Inclusion {
    /// Returns `true` when the associated records should be included.
    pub const fn is_included(&self) -> bool {
        matches!(self, Self::Include)
    }

    /// Return `true` when the associated records should be expected.
    pub const fn is_excluded(&self) -> bool {
        matches!(self, Self::Exclude)
    }
}

/// Specifies how much of the query must match a UMLS concept name.
///
/// This corresponds to the `partialSearch` parameter accepted by the UMLS
/// Metathesaurus search endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermMatch {
    /// Requires all query terms to match.
    ///
    /// This corresponds to `partialSearch=false`, the UMLS API default.
    AllTerms,

    /// Allows results that match only part of the query.
    ///
    /// This corresponds to `partialSearch=true`.
    PartialTerms,
}

impl TermMatch {
    /// Returns `true` when results may match only part of the query.
    pub const fn allows_partial(self) -> bool {
        matches!(self, Self::PartialTerms)
    }

    /// Returns `true` when results must match all query terms.
    pub const fn allows_all_terms(self) -> bool {
        matches!(self, Self::AllTerms)
    }
}

#[derive(Debug, Error)]
#[error("Empty string error")]
/// Indicates that a textual value was empty after trimming whitespace.
pub struct EmptyString;

fn get_trimmed_non_empty_string(value: &str) -> Result<&str, EmptyString> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmptyString);
    }
    Ok(value)
}

/// A human-readable display name returned by the UMLS API.
///
/// Unlike UMLS identifiers, a display name is intended for presentation and
/// does not uniquely identify a concept or source vocabulary entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DisplayName(String);

impl DisplayName {
    /// Creates a display name after trimming surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    pub fn try_new(value: &str) -> Result<Self, EmptyString> {
        get_trimmed_non_empty_string(value).map(|x| Self(x.into()))
    }

    /// Returns the display name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the display name and returns its underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for DisplayName {
    /// Returns the display name as a string slice.
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for DisplayName {
    /// Formats the human-readable display name.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DisplayName {
    type Err = EmptyString;

    /// Parses a display name after trimming surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for DisplayName {
    type Error = EmptyString;

    /// Creates a display name from a borrowed string.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for DisplayName {
    type Error = EmptyString;

    /// Creates a display name from an owned string.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(&value)
    }
}

impl<'de> serde::Deserialize<'de> for DisplayName {
    /// Deserializes and validates a display name.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <&str>::deserialize(deserializer)?;
        Self::try_new(value).map_err(serde::de::Error::custom)
    }
}

/// A validated search term accepted by the UMLS search API.
///
/// A `Term` is guaranteed to contain at least one non-whitespace character.
/// Leading and trailing whitespace is removed during construction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Term(String);

impl Term {
    /// Creates a term after trimming surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace after trimming.
    pub fn try_new(value: &str) -> Result<Self, EmptyString> {
        get_trimmed_non_empty_string(value).map(|value| Self(value.into()))
    }

    /// Returns the validated search term as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Term {
    /// Returns the validated search term as a string slice.
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Term {
    type Err = EmptyString;

    /// Parses and validates a search term.
    ///
    /// Leading and trailing whitespace is removed before validation.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when the supplied value is empty or contains
    /// only whitespace.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for Term {
    type Error = EmptyString;

    /// Creates a validated search term from a string slice.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when the supplied value is empty or contains
    /// only whitespace.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl fmt::Display for Term {
    /// Formats the term using its validated string value.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
/// Pagination parameters for a UMLS search request.
pub struct Page {
    size: u64,
    number: u64,
}

impl Page {
    /// Creates pagination parameters with the given page size and number.
    ///
    /// This constructor does not validate zero values. Requests created by
    /// [`Builder::build`] reject a zero size or page number.
    pub fn new(size: u64, number: u64) -> Self {
        Self { size, number }
    }

    /// Returns the maximum number of results requested for the page.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Returns the one-based page number.
    pub fn number(&self) -> u64 {
        self.number
    }
}

/// A validated set of parameters for a UMLS Metathesaurus search.
///
/// Construct requests with [`Request::builder`] and [`Builder::build`].
pub struct Request {
    term: Term,
    input_type: InputType,
    obsolete: Inclusion,
    suppressible: Inclusion,
    return_id_type: ReturnIdType,
    source_vocabularies: Vec<DisplayName>,
    search_type: SearchType,
    semantic_types: Vec<DisplayName>,
    semantic_groups: Vec<DisplayName>,
    term_match: TermMatch,
    page: Page,
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

    /// Returns the identifier type requested for search results.
    pub fn return_id_type(&self) -> ReturnIdType {
        self.return_id_type
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
    pub fn page(&self) -> &Page {
        &self.page
    }
}

/// Builder for a validated UMLS search [`Request`].
///
/// New builders exclude obsolete and suppressible terms, return concept
/// identifiers, use word matching, require all terms, and request the first
/// page with 25 results.
pub struct Builder {
    term: Term,
    input_type: InputType,
    obsolete: Inclusion,
    suppressible: Inclusion,
    return_id_type: ReturnIdType,
    source_vocabularies: Vec<DisplayName>,
    search_type: SearchType,
    semantic_types: Vec<DisplayName>,
    semantic_groups: Vec<DisplayName>,
    term_match: TermMatch,
    page_size: u64,
    page_number: u64,
}

impl Builder {
    /// Creates a builder with the default UMLS search parameters.
    pub fn new(term: Term) -> Self {
        Self {
            term,
            input_type: InputType::Atom,
            obsolete: Inclusion::Exclude,
            suppressible: Inclusion::Exclude,
            return_id_type: ReturnIdType::Concept,
            source_vocabularies: Vec::new(),
            search_type: SearchType::Words,
            semantic_types: Vec::new(),
            semantic_groups: Vec::new(),
            term_match: TermMatch::AllTerms,
            page_size: 25,
            page_number: 1,
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

    /// Sets the identifier type returned for matching records.
    pub fn return_id_type(mut self, return_id_type: ReturnIdType) -> Self {
        self.return_id_type = return_id_type;
        self
    }

    /// Replaces the source-vocabulary filters.
    pub fn source_vocabularies(mut self, source_vocabularies: Vec<DisplayName>) -> Self {
        self.source_vocabularies = source_vocabularies;
        self
    }

    /// Adds a source-vocabulary filter.
    pub fn add_source_vocabulary(mut self, source_vocabulary: impl Into<DisplayName>) -> Self {
        self.source_vocabularies.push(source_vocabulary.into());
        self
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
    pub fn add_semantic_type(mut self, semantic_type: impl Into<DisplayName>) -> Self {
        self.semantic_types.push(semantic_type.into());
        self
    }

    /// Replaces the semantic-group filters.
    pub fn semantic_groups(mut self, semantic_groups: Vec<DisplayName>) -> Self {
        self.semantic_groups = semantic_groups;
        self
    }

    /// Adds a semantic-group filter.
    pub fn add_semantic_group(mut self, semantic_group: impl Into<DisplayName>) -> Self {
        self.semantic_groups.push(semantic_group.into());
        self
    }

    /// Sets whether all query terms are required or partial matches are
    /// allowed.
    pub fn term_match(mut self, term_match: TermMatch) -> Self {
        self.term_match = term_match;
        self
    }

    /// Sets the maximum number of results requested for a page.
    pub fn page_size(mut self, page_size: u64) -> Self {
        self.page_size = page_size;
        self
    }

    /// Sets the one-based page number to request.
    pub fn page_number(mut self, page_number: u64) -> Self {
        self.page_number = page_number;
        self
    }

    /// Builds and validates the search request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ZeroPageSize`] when the page size is zero, or
    /// [`Error::ZeroPageNumber`] when the page number is zero.
    pub fn build(self) -> Result<Request, Error> {
        if self.page_size == 0 {
            return Err(Error::ZeroPageSize);
        }

        if self.page_number == 0 {
            return Err(Error::ZeroPageNumber);
        }

        let page = Page::new(self.page_size, self.page_number);

        Ok(Request {
            term: self.term,
            input_type: self.input_type,
            obsolete: self.obsolete,
            suppressible: self.suppressible,
            return_id_type: self.return_id_type,
            source_vocabularies: self.source_vocabularies,
            search_type: self.search_type,
            semantic_types: self.semantic_types,
            semantic_groups: self.semantic_groups,
            term_match: self.term_match,
            page,
        })
    }
}

#[derive(Debug, Error)]
/// An error produced while validating a search request.
pub enum Error {
    /// The requested page size was zero.
    #[error("Zero page size")]
    ZeroPageSize,

    /// The requested page number was zero.
    #[error("Zero page number")]
    ZeroPageNumber,
}

#[cfg(test)]
mod tests {

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
            assert_eq!(term.0.as_str(), "term");
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
        let page = Page::new(25, 1);

        assert_eq!(page.size(), 25);
        assert_eq!(page.number(), 1);
    }

    #[test]
    fn builds_request_with_valid_page() -> Result<(), Box<dyn std::error::Error>> {
        let term = Term::try_new("cough")?;

        let request = Request::builder(term).build()?;

        assert_eq!(request.page().size(), 25);
        assert_eq!(request.page().number(), 1);

        Ok(())
    }

    #[test]
    fn rejects_zero_page_size() -> Result<(), Box<dyn std::error::Error>> {
        let term = Term::try_new("cough")?;

        let result = Request::builder(term).page_size(0).build();

        assert!(matches!(result, Err(Error::ZeroPageSize)));

        Ok(())
    }

    #[test]
    fn rejects_zero_page_number() -> Result<(), Box<dyn std::error::Error>> {
        let term = Term::try_new("cough")?;

        let result = Request::builder(term).page_number(0).build();

        assert!(matches!(result, Err(Error::ZeroPageNumber)));

        Ok(())
    }

    #[test]
    fn reports_page_size_error_before_page_number_error() -> Result<(), Box<dyn std::error::Error>>
    {
        let term = Term::try_new("cough")?;

        let result = Request::builder(term).page_size(0).page_number(0).build();

        assert!(matches!(result, Err(Error::ZeroPageSize)));

        Ok(())
    }
}
