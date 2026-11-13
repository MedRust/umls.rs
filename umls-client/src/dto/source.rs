use serde::{Deserialize, Serialize};

use crate::dto::display_name::DisplayName;

/// A source-asserted concept or atom cluster defined by a source vocabulary (such as SNOMED CT or MeSH).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAtomCluster {
    ui: DisplayName,
    /// Human-readable name.
    name: DisplayName,
    root_source: DisplayName,
    suppressible: bool,
    obsolete: bool,
    atom_count: u64,
    #[serde(rename = "cVMemberCount")]
    cv_member_count: u64,
}

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

/// A source-asserted concept identified by its source vocabulary and code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConcept {
    source: DisplayName,
    code: DisplayName,
}

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
