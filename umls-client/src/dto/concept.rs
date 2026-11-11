use crate::{atom, definition, relation, search::core::get_trimmed_non_empty_string, types};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConceptResponse {
    pub(crate) result: SerializableConcept,
}

/// A full concept returned by the UMLS content endpoint.
pub struct Payload {
    /// The response object's UMLS class discriminator.
    pub class_type: ConceptClass,
    /// The concept identifier returned by UMLS.
    pub ui: String,
    /// Whether the concept is suppressible.
    pub suppressible: bool,
    /// Date on which the concept was added, in the API's textual format.
    pub date_added: String,
    /// Date of the concept's latest major revision.
    pub major_revision_date: String,
    /// Current concept status reported by UMLS.
    pub status: String,
    /// Semantic types assigned to the concept.
    pub semantic_types: Vec<SemanticType>,
    /// Number of atoms associated with the concept.
    pub atom_count: u64,
    /// Number of attributes associated with the concept.
    pub attribute_count: u64,
    /// Number of content-view memberships associated with the concept.
    pub cv_member_count: u64,
    /// Endpoint for the concept's atoms, when available.
    pub atoms: atom::Atom<umls_http::WithoutClient, types::atom::All, Url>,
    /// Endpoint for the concept's definitions, when available.
    pub definitions: Option<definition::Definition<umls_http::WithoutClient, Url>>,
    /// Endpoint for the concept's relations, when available.
    pub relations: Option<relation::Relation<umls_http::WithoutClient, Url>>,
    /// Endpoint for the default preferred atom, when available.
    pub default_preferred_atom: atom::Atom<umls_http::WithoutClient, types::atom::Preferred, Url>,
    /// Number of relations associated with the concept.
    pub relation_count: u64,
    /// Human-readable preferred name of the concept.
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A full concept returned by the UMLS content endpoint.
pub(crate) struct SerializableConcept {
    /// The response object's UMLS class discriminator.
    class_type: ConceptClass,
    /// The concept identifier returned by UMLS.
    ui: String,
    /// Whether the concept is suppressible.
    suppressible: bool,
    /// Date on which the concept was added, in the API's textual format.
    date_added: String,
    /// Date of the concept's latest major revision.
    major_revision_date: String,
    /// Current concept status reported by UMLS.
    status: String,
    /// Semantic types assigned to the concept.
    semantic_types: Vec<SemanticType>,
    /// Number of atoms associated with the concept.
    atom_count: u64,
    /// Number of attributes associated with the concept.
    attribute_count: u64,
    /// Number of content-view memberships associated with the concept.
    cv_member_count: u64,
    /// Endpoint for the concept's atoms, when available.
    atoms: Url,

    /// Endpoint for the concept's definitions, when available.
    definitions: OptionalResource,

    /// Endpoint for the concept's relations, when available.
    relations: OptionalResource,
    /// Endpoint for the default preferred atom, when available.
    default_preferred_atom: Url,
    /// Number of relations associated with the concept.
    relation_count: u64,
    /// Human-readable preferred name of the concept.
    name: String,
}

impl From<SerializableConcept> for Payload {
    fn from(value: SerializableConcept) -> Self {
        let definitions = match value.definitions {
            OptionalResource::Available(url) => Some(definition::Definition::new(url)),
            OptionalResource::Unavailable => None,
        };

        let relations = match value.relations {
            OptionalResource::Available(url) => Some(relation::Relation::new(url)),
            OptionalResource::Unavailable => None,
        };

        let atoms = atom::Atom::new(value.atoms);

        let default_preferred_atom = atom::Atom::new(value.default_preferred_atom);

        Payload {
            class_type: value.class_type,
            ui: value.ui,
            suppressible: value.suppressible,
            date_added: value.date_added,
            major_revision_date: value.major_revision_date,
            status: value.status,
            semantic_types: value.semantic_types,
            atom_count: value.atom_count,
            attribute_count: value.attribute_count,
            cv_member_count: value.cv_member_count,
            definitions,
            relations,
            relation_count: value.relation_count,
            name: value.name,
            atoms,
            default_preferred_atom,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A resource link that the UMLS API may represent with the `NONE` sentinel.
pub enum OptionalResource {
    /// The resource is available at the contained URL.
    Available(Url),
    /// The API reported that the resource is unavailable.
    Unavailable,
}

impl<'de> Deserialize<'de> for OptionalResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let trimmed = get_trimmed_non_empty_string(&string).map_err(serde::de::Error::custom)?;
        if trimmed == "NONE" {
            return Ok(OptionalResource::Unavailable);
        }

        let url: Url = trimmed.parse().map_err(serde::de::Error::custom)?;
        Ok(OptionalResource::Available(url))
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
/// The UMLS class discriminator for a concept response.
pub enum ConceptClass {
    /// A UMLS concept resource.
    Concept,
}

#[derive(Debug, Deserialize)]
/// A semantic type assigned to a concept.
pub struct SemanticType {
    /// Human-readable semantic-type name.
    pub name: String,
    /// UMLS resource URL for the semantic type.
    pub uri: OptionalResource,
}
