use crate::dto::serde::bool_as_string;
use crate::search::core::DisplayName;
use serde::{Deserialize, Serialize};

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
