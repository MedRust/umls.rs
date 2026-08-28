use core::fmt;
use std::{num::NonZeroU64, str::FromStr};

/// The default non zero page size.
pub const DEFAULT_PAGE_SIZE: NonZeroU64 = NonZeroU64::new(25).unwrap();

/// The default non zero page number.
pub const DEFAULT_PAGE_NUMBER: NonZeroU64 = NonZeroU64::new(1).unwrap();

#[derive(Debug, thiserror::Error)]
#[error("Empty string error")]
/// Indicates that a textual value was empty after trimming whitespace.
pub struct EmptyString;

/// Trims a string and verifies that at least one non-whitespace character remains.
///
/// # Errors
///
/// Returns [`EmptyString`] when `value` is empty after trimming.
pub fn get_trimmed_non_empty_string(value: &str) -> Result<&str, EmptyString> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmptyString);
    }
    Ok(value)
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
        let value = String::deserialize(deserializer)?;
        Self::try_new(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
/// Pagination parameters for a UMLS search request.
pub struct PageRequest {
    size: NonZeroU64,
    number: NonZeroU64,
}

impl PageRequest {
    /// Creates pagination parameters with the given page size and number.
    pub fn new(size: NonZeroU64, number: NonZeroU64) -> Self {
        Self { size, number }
    }

    /// Returns the maximum number of results requested for the page.
    pub fn size(&self) -> NonZeroU64 {
        self.size
    }

    /// Returns the one-based page number.
    pub fn number(&self) -> NonZeroU64 {
        self.number
    }

    /// Increament page count by 1.
    ///
    /// If Return [PageError::Overflow] if the adding 1 overflow.
    pub fn checked_next(&mut self) -> Result<(), PageError> {
        let number = self
            .number
            .get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or(PageError::Overflow)?;

        self.number = number;
        Ok(())
    }

    /// Decreament page count by 1.
    ///
    /// If returns [PageError::NoPreviousPage] if the substracting 1 overflow.
    pub fn checked_previous(&mut self) -> Result<(), PageError> {
        let number = self
            .number
            .get()
            .checked_sub(1)
            .and_then(NonZeroU64::new)
            .ok_or(PageError::NoPreviousPage)?;

        self.number = number;
        Ok(())
    }
}

/// The Page validation error.
#[derive(Debug, thiserror::Error)]
pub enum PageError {
    /// Indicate if overflow while increament.
    #[error("Page overflow")]
    Overflow,

    /// Indicate if overflow while Decreament.
    #[error("No previous page")]
    NoPreviousPage,
}
