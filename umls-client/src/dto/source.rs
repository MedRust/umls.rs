use serde::{Deserialize, Serialize};

use crate::search::core::DisplayName;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SerializableSourceAtomCluster {
    ui: String,
    suppressible: bool,
    obsolete: bool,
    root_source: String,
    atom_count: u64,
    /// Explicitly renamed because UTS capitalizes the 'V' ("cVMemberCount").
    #[serde(rename = "cVMemberCount")]
    cv_member_count: u64,
    name: String,
}

/// A source-asserted concept or atom cluster defined by a source vocabulary (such as SNOMED CT or MeSH).
#[derive(Debug, Clone)]
pub struct SourceAtomCluster {
    ui: DisplayName,
    /// Human-readable name.
    name: DisplayName,
    root_source: DisplayName,
    suppressible: bool,
    obsolete: bool,
    atom_count: u64,
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

impl TryFrom<SerializableSourceAtomCluster> for SourceAtomCluster {
    type Error = crate::search::core::EmptyString;

    fn try_from(wire: SerializableSourceAtomCluster) -> Result<Self, Self::Error> {
        Ok(Self {
            ui: wire.ui.parse()?,
            name: wire.name.parse()?,
            root_source: wire.root_source.parse()?,
            suppressible: wire.suppressible,
            obsolete: wire.obsolete,
            atom_count: wire.atom_count,
            cv_member_count: wire.cv_member_count,
        })
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

#[cfg(test)]
mod tests {

    use crate::search::core::EmptyString;

    use super::*;

    #[test]
    fn success_on_serializable_source_atom_cluster_to_source_atom_cluster() {
        let object = SerializableSourceAtomCluster {
            ui: "82065001".into(),
            suppressible: false,
            obsolete: false,
            root_source: "SNOMEDCT_US".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "Fracture of carpal bone".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_ok());

        let object = result.unwrap();
        assert_eq!(object.ui().as_str(), "82065001");
        assert_eq!(object.name().as_str(), "Fracture of carpal bone");
        assert!(!object.suppressible());
        assert!(!object.obsolete());
        assert_eq!(object.root_source().as_str(), "SNOMEDCT_US");
    }

    #[test]
    fn failed_on_empty_ui() {
        let object = SerializableSourceAtomCluster {
            ui: "".into(),
            suppressible: false,
            obsolete: false,
            root_source: "SNOMEDCT_US".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "Fracture of carpal bone".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn failed_on_empty_name() {
        let object = SerializableSourceAtomCluster {
            ui: "82065001".into(),
            suppressible: false,
            obsolete: false,
            root_source: "SNOMEDCT_US".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn failed_on_empty_root_source() {
        let object = SerializableSourceAtomCluster {
            ui: "82065001".into(),
            suppressible: false,
            obsolete: false,
            root_source: "".into(),
            atom_count: 7,
            cv_member_count: 0,
            name: "Fracture of carpal bone".into(),
        };

        let result: Result<SourceAtomCluster, EmptyString> = object.try_into();
        assert!(result.is_err());
    }
}
