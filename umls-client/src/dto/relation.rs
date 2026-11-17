//! Relation Data Transfer Objects (DTOs) for UMLS relationships.
//!
//! This module provides models and builders for deserializing and working with
//! UMLS relations returned by the REST API, including concept relations, atom
//! relations, and atom cluster relations.
//!
//! # Examples
//!
//! ```
//! use umls_client::dto::relation::{Builder, ClassType};
//!
//! let relation = Builder::new()
//!     .ui("R12345678".parse().unwrap())
//!     .class_type(ClassType::ConceptRelation)
//!     .root_source("MTH".parse().unwrap())
//!     .relation_label("RO".parse().unwrap())
//!     .build()
//!     .unwrap();
//!
//! assert!(relation.is_concept_relation_class_type());
//! ```

use crate::{
    dto::{
        display_name::DisplayName,
        serde::{optional_display_name, optional_url},
    },
    info::Info,
    types,
};
use serde::{Deserialize, Serialize};
use umls_http::WithoutClient;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SerializableRelation {
    /// UMLS Relationship Unique Identifier (RUI)
    ui: DisplayName,
    /// Represents respective object is suppressible.
    suppressible: bool,
    /// The source unique identifier if present.
    #[serde(with = "optional_display_name")]
    source_ui: Option<DisplayName>,
    /// Represents respective object is obsolete.
    obsolete: bool,
    /// Represents respective object is source originated.
    source_originated: bool,
    /// The root source vocabularies.
    root_source: DisplayName,
    /// The grou id if present.
    #[serde(with = "optional_display_name")]
    group_id: Option<DisplayName>,
    /// The attribute count.
    attribute_count: u32,
    /// The class type.
    class_type: ClassType,
    /// From Id
    #[serde(with = "optional_url", default)]
    related_from_id: Option<Url>,
    /// From name
    #[serde(with = "optional_display_name", default)]
    related_from_id_name: Option<DisplayName>,
    /// The relationship label.
    relation_label: DisplayName,
    /// The additional relationship label.
    #[serde(with = "optional_display_name")]
    additional_relation_label: Option<DisplayName>,
    /// The related id.
    #[serde(with = "optional_url", default)]
    related_id: Option<Url>,
    /// The related id name
    #[serde(with = "optional_display_name", default)]
    related_id_name: Option<DisplayName>,
}

impl TryFrom<SerializableRelation> for RelationDto {
    type Error = Error;

    fn try_from(value: SerializableRelation) -> Result<Self, Self::Error> {
        Builder::default()
            .ui(value.ui)
            .suppressible(value.suppressible)
            .source_ui(value.source_ui)
            .obsolete(value.obsolete)
            .source_originated(value.source_originated)
            .root_source(value.root_source)
            .group_id(value.group_id)
            .attribute_count(value.attribute_count)
            .class_type(value.class_type)
            .related_from_id(value.related_from_id)
            .related_from_id_name(value.related_from_id_name)
            .relation_label(value.relation_label)
            .additional_relation_label(value.additional_relation_label)
            .related_id(value.related_id)
            .related_id_name(value.related_id_name)
            .build()
    }
}

/// Returns a list of all supported relation class types.
///
/// This helper is used by callers to discover which [`ClassType`] variants are
/// supported when constructing or validating [`RelationDto`] instances.
pub fn supporeted_class_types() -> Vec<ClassType> {
    vec![
        ClassType::ConceptRelation,
        ClassType::AtomRelation,
        ClassType::AtomClusterRelation,
    ]
}

/// A fluent builder for constructing [`RelationDto`] instances.
#[derive(Debug, Default, Clone)]
pub struct Builder {
    ui: Option<DisplayName>,
    suppressible: Option<bool>,
    source_ui: Option<DisplayName>,
    obsolete: Option<bool>,
    source_originated: Option<bool>,
    root_source: Option<DisplayName>,
    group_id: Option<DisplayName>,
    attribute_count: Option<u32>,
    class_type: Option<ClassType>,
    related_from_id: Option<Url>,
    related_from_id_name: Option<DisplayName>,
    relation_label: Option<DisplayName>,
    additional_relation_label: Option<DisplayName>,
    related_id: Option<Url>,
    related_id_name: Option<DisplayName>,
}

impl Builder {
    /// Creates a new empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the relationship unique identifier (RUI).
    pub fn ui(mut self, ui: DisplayName) -> Self {
        self.ui = Some(ui);
        self
    }

    /// Sets whether the relation is suppressible.
    pub fn suppressible(mut self, suppressible: bool) -> Self {
        self.suppressible = Some(suppressible);
        self
    }

    /// Sets the source-asserted unique identifier, if present.
    pub fn source_ui(mut self, source_ui: impl Into<Option<DisplayName>>) -> Self {
        self.source_ui = source_ui.into();
        self
    }

    /// Sets whether the relation is marked as obsolete.
    pub fn obsolete(mut self, obsolete: bool) -> Self {
        self.obsolete = Some(obsolete);
        self
    }

    /// Sets whether the relation originated from the source vocabulary.
    pub fn source_originated(mut self, source_originated: bool) -> Self {
        self.source_originated = Some(source_originated);
        self
    }

    /// Sets the root source vocabulary (e.g., `"SNOMEDCT_US"`, `"MTH"`).
    pub fn root_source(mut self, root_source: DisplayName) -> Self {
        self.root_source = Some(root_source);
        self
    }

    /// Sets the relation group identifier, if present.
    pub fn group_id(mut self, group_id: impl Into<Option<DisplayName>>) -> Self {
        self.group_id = group_id.into();
        self
    }

    /// Sets the relation attribute count.
    pub fn attribute_count(mut self, attribute_count: u32) -> Self {
        self.attribute_count = Some(attribute_count);
        self
    }

    /// Sets the relation class type.
    pub fn class_type(mut self, class_type: ClassType) -> Self {
        self.class_type = Some(class_type);
        self
    }

    /// Sets the URL endpoint of the originating entity (`related_from_id`), if present.
    pub fn related_from_id(mut self, related_from_id: impl Into<Option<Url>>) -> Self {
        self.related_from_id = related_from_id.into();
        self
    }

    /// Sets the human-readable name of the originating entity, if present.
    pub fn related_from_id_name(
        mut self,
        related_from_id_name: impl Into<Option<DisplayName>>,
    ) -> Self {
        self.related_from_id_name = related_from_id_name.into();
        self
    }

    /// Sets the relationship label (e.g., `"PAR"`, `"CHD"`, `"RO"`).
    pub fn relation_label(mut self, relation_label: DisplayName) -> Self {
        self.relation_label = Some(relation_label);
        self
    }

    /// Sets the additional relationship label (e.g., `"isa"`, `"inverse_isa"`), if present.
    pub fn additional_relation_label(
        mut self,
        additional_relation_label: impl Into<Option<DisplayName>>,
    ) -> Self {
        self.additional_relation_label = additional_relation_label.into();
        self
    }

    /// Sets the URL endpoint of the target entity (`related_id`), if present.
    pub fn related_id(mut self, related_id: impl Into<Option<Url>>) -> Self {
        self.related_id = related_id.into();
        self
    }

    /// Sets the human-readable name of the target entity, if present.
    pub fn related_id_name(mut self, related_id_name: impl Into<Option<DisplayName>>) -> Self {
        self.related_id_name = related_id_name.into();
        self
    }

    /// Consumes the builder and returns a `RelationDto`.
    ///
    /// If suppressible, obsolete and source_originated is not specified it defaults to false.
    /// If attribute count is not specified it defaults to 0.
    ///
    /// # Errors
    /// - [`Error::MissingClassType`]: On missing class type.
    /// - [`Error::UnsupportedClassType`]: When sepcified class type is not supported.
    /// - [`Error::MissingUi`]: On unique identifier is not specified.
    /// - [`Error::MissingRootSource`]: On missing root source.
    /// - [`Error::MissingRelationLabel`]: On missing relation label.
    pub fn build(self) -> Result<RelationDto, Error> {
        let class_type = self.class_type.ok_or(Error::MissingClassType)?;

        if !supporeted_class_types().contains(&class_type) {
            return Err(Error::UnsupportedClassType(class_type));
        }

        Ok(RelationDto {
            ui: self.ui.ok_or(Error::MissingUi)?,
            suppressible: self.suppressible.unwrap_or(false),
            source_ui: self.source_ui,
            obsolete: self.obsolete.unwrap_or(false),
            source_originated: self.source_originated.unwrap_or(false),
            root_source: self.root_source.ok_or(Error::MissingRootSource)?,
            group_id: self.group_id,
            attribute_count: self.attribute_count.unwrap_or(0),
            class_type,
            related_from_id: self.related_from_id,
            related_from_id_name: self.related_from_id_name,
            relation_label: self.relation_label.ok_or(Error::MissingRelationLabel)?,
            additional_relation_label: self.additional_relation_label,
            related_id: self.related_id,
            related_id_name: self.related_id_name,
        })
    }
}

/// A relationship Data Transfer Object (DTO) representing a UMLS relation response.
#[derive(Debug, Clone)]
pub struct RelationDto {
    /// UMLS Relationship Unique Identifier (RUI)
    ui: DisplayName,
    /// Represents respective object is suppressible.
    suppressible: bool,
    /// The source unique identifier if present.
    source_ui: Option<DisplayName>,
    /// Represents respective object is obsolete.
    obsolete: bool,
    /// Represents respective object is source originated.
    source_originated: bool,
    /// The root source vocabularies.
    root_source: DisplayName,
    /// The grou id if present.
    group_id: Option<DisplayName>,
    /// The attribute count.
    attribute_count: u32,
    /// The class type.
    class_type: ClassType,
    /// From Id
    related_from_id: Option<Url>,
    /// From name
    related_from_id_name: Option<DisplayName>,
    /// The relationship label.
    relation_label: DisplayName,
    /// The additional relationship label.
    additional_relation_label: Option<DisplayName>,
    /// The related id.
    related_id: Option<Url>,
    /// The related id name
    related_id_name: Option<DisplayName>,
}

impl RelationDto {
    /// Returns `true` if the relation class type is [`ClassType::AtomRelation`].
    pub fn is_atom_relation_class_type(&self) -> bool {
        self.class_type == ClassType::AtomRelation
    }

    /// Returns `true` if the relation class type is [`ClassType::ConceptRelation`].
    pub fn is_concept_relation_class_type(&self) -> bool {
        self.class_type == ClassType::ConceptRelation
    }

    /// Returns `true` if the relation class type is [`ClassType::AtomClusterRelation`].
    pub fn is_atom_cluster_relation_class_type(&self) -> bool {
        self.class_type == ClassType::AtomClusterRelation
    }

    /// Returns the relationship [`ClassType`].
    pub fn class_type(&self) -> ClassType {
        self.class_type
    }

    /// Consumes `self` and returns an [`AtomRelation`] object.
    ///
    /// # Errors
    ///
    /// - [`Error::NotAtomRelation`]: When class type is not [`ClassType::AtomRelation`].
    /// - [`Error::MissingRelatedFromId`]: Related from id is missing.
    /// - [`Error::MissingRelatedFromName`]: Related from id name is missing.
    /// - [`Error::MissingRelatedId`]: Related id is missing.
    /// - [`Error::MissingRelatedIdName`]: Related id name is missing.
    pub fn try_into_atom_relation(self) -> Result<AtomRelation, Error> {
        self.try_into()
    }

    /// Consumes `self` and returns a [`ConceptRelation`] object.
    ///
    /// # Errors
    ///
    /// - [`Error::NotConceptRelation`]: When class type is not [`ClassType::ConceptRelation`].
    /// - [`Error::MissingRelatedId`]: Related id is missing.
    /// - [`Error::MissingRelatedIdName`]: Related id name is missing.
    pub fn try_into_concept_relation(self) -> Result<ConceptRelation, Error> {
        self.try_into()
    }

    /// Consumes `self` and returns an [`AtomClusterRelation`] object.
    ///
    /// # Errors
    ///
    /// - [`Error::NotAtomClusterRelation`]: When class type is not [`ClassType::AtomClusterRelation`].
    /// - [`Error::MissingRelatedFromId`]: Related from id is missing.
    /// - [`Error::MissingRelatedFromName`]: Related from id name is missing.
    /// - [`Error::MissingRelatedId`]: Related id is missing.
    /// - [`Error::MissingRelatedIdName`]: Related id name is missing.
    pub fn try_into_atom_cluster_relation(self) -> Result<AtomClusterRelation, Error> {
        self.try_into()
    }
}

/// Represents UMLS relation class types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum ClassType {
    /// Concept relation class type (`ConceptRelation`).
    ConceptRelation,
    /// Atom cluster relation class type (`AtomClusterRelation`).
    AtomClusterRelation,
    /// Atom relation class type (`AtomRelation`).
    AtomRelation,
}

impl ClassType {
    /// Returns string representation of class type.
    pub fn as_str(&self) -> &str {
        match self {
            ClassType::ConceptRelation => "ConceptRelation",
            ClassType::AtomClusterRelation => "AtomClusterRelation",
            ClassType::AtomRelation => "AtomRelation",
        }
    }
}

impl std::fmt::Display for ClassType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An atom-level UMLS relation, linking two atoms via a typed relationship.
///
/// Produced by calling [`RelationDto::try_into_atom_relation`].
pub struct AtomRelation {
    ui: DisplayName,
    suppressible: bool,
    source_ui: Option<DisplayName>,
    obsolete: bool,
    source_originated: bool,
    root_source: DisplayName,
    group_id: Option<DisplayName>,
    attribute_count: u32,
    related_from_id: Info<WithoutClient, types::Atom, Url>,
    related_from_id_name: DisplayName,
    relation_label: DisplayName,
    additional_relation_label: Option<DisplayName>,
    related_id: Info<WithoutClient, types::Atom, Url>,
    related_id_name: DisplayName,
}

impl AtomRelation {
    /// Returns the relationship unique identifier (RUI).
    ///
    /// This corresponds to the `ui` field in the UMLS REST response.
    pub fn ui(&self) -> &DisplayName {
        &self.ui
    }

    /// Returns suppressible.
    pub fn suppressible(&self) -> bool {
        self.suppressible
    }

    /// Returns source unique identifier.
    pub fn source_ui(&self) -> Option<&DisplayName> {
        self.source_ui.as_ref()
    }

    /// Returns whether the relation is obsolete.
    pub fn obsolete(&self) -> bool {
        self.obsolete
    }

    /// Returns source originated.
    pub fn source_originated(&self) -> bool {
        self.source_originated
    }

    /// Returns root source vocabulary.
    pub fn root_source(&self) -> &DisplayName {
        &self.root_source
    }

    /// Returns group identifier.
    pub fn group_id(&self) -> Option<&DisplayName> {
        self.group_id.as_ref()
    }

    /// Returns attribute count.
    pub fn attribute_count(&self) -> u32 {
        self.attribute_count
    }

    /// Returns atom info object for source.
    pub fn related_from_id(&self) -> &Info<WithoutClient, types::Atom, Url> {
        &self.related_from_id
    }

    /// Returns human readable name of respective from id.
    pub fn related_from_id_name(&self) -> &DisplayName {
        &self.related_from_id_name
    }

    /// Returns relation label.
    pub fn relation_label(&self) -> &DisplayName {
        &self.relation_label
    }

    /// Returns additional relation label.
    pub fn additional_relation_label(&self) -> Option<&DisplayName> {
        self.additional_relation_label.as_ref()
    }

    /// Returns atom info object of destination.
    pub fn related_id(&self) -> &Info<WithoutClient, types::Atom, Url> {
        &self.related_id
    }

    /// Returns human readable name of respective to id.
    pub fn related_id_name(&self) -> &DisplayName {
        &self.related_id_name
    }
}

/// Errors that can occur when building or converting relation DTOs.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The relation's class type is not [`ClassType::AtomRelation`].
    #[error("Not a atom realtion class type")]
    NotAtomRelation,
    /// The `related_from_id` field is absent but required.
    #[error("Missing related from id")]
    MissingRelatedFromId,
    /// The `related_from_id_name` field is absent but required.
    #[error("Missing related from name")]
    MissingRelatedFromName,
    /// The `related_id` field is absent but required.
    #[error("Missing related id")]
    MissingRelatedId,
    /// The `related_id_name` field is absent but required.
    #[error("Missing related id name")]
    MissingRelatedIdName,
    /// The relation's class type is not [`ClassType::ConceptRelation`].
    #[error("Not a concept relation")]
    NotConceptRelation,
    /// The relation's class type is not [`ClassType::AtomClusterRelation`].
    #[error("Not a atom cluster relation")]
    NotAtomClusterRelation,
    /// The `ui` (relationship unique identifier) field is absent but required.
    #[error("Missing ui")]
    MissingUi,
    /// The `class_type` field is absent but required.
    #[error("Missing class type")]
    MissingClassType,
    /// The provided class type is not in the list of supported types.
    #[error("Unsupported class type: {0}")]
    UnsupportedClassType(ClassType),
    /// The `root_source` field is absent but required.
    #[error("Missing root source")]
    MissingRootSource,
    /// The `relation_label` field is absent but required.
    #[error("Missing relation label")]
    MissingRelationLabel,
}

#[allow(clippy::infallible_try_from)]
impl TryFrom<RelationDto> for AtomRelation {
    type Error = Error;

    fn try_from(value: RelationDto) -> Result<Self, Self::Error> {
        if !value.is_atom_relation_class_type() {
            return Err(Error::NotAtomRelation);
        }

        let ui = value.ui;
        let suppressible = value.suppressible;
        let source_ui = value.source_ui;
        let obsolete = value.obsolete;
        let source_originated = value.source_originated;
        let root_source = value.root_source;
        let group_id = value.group_id;
        let attribute_count = value.attribute_count;
        let related_from_id = Info::<_, types::Atom, _>::new(
            value.related_from_id.ok_or(Error::MissingRelatedFromId)?,
        );
        let related_from_id_name = value
            .related_from_id_name
            .ok_or(Error::MissingRelatedFromName)?;
        let relation_label = value.relation_label;
        let additional_relation_label = value.additional_relation_label;
        let related_id =
            Info::<_, types::Atom, _>::new(value.related_id.ok_or(Error::MissingRelatedId)?);
        let related_id_name = value.related_id_name.ok_or(Error::MissingRelatedIdName)?;

        Ok(AtomRelation {
            ui,
            suppressible,
            source_ui,
            obsolete,
            source_originated,
            root_source,
            group_id,
            attribute_count,
            related_from_id,
            related_from_id_name,
            relation_label,
            additional_relation_label,
            related_id,
            related_id_name,
        })
    }
}

/// A concept-level UMLS relation, linking two concepts via a typed relationship.
///
/// Produced by calling [`RelationDto::try_into_concept_relation`].
pub struct ConceptRelation {
    /// UMLS Relationship Unique Identifier
    ui: DisplayName,
    /// Represents respective object is suppressible.
    suppressible: bool,
    /// The source unique identifier if present.
    source_ui: Option<DisplayName>,
    /// Represents respective object is obsolete.
    obsolete: bool,
    /// Represents respective object is source originated.
    source_originated: bool,
    /// The root source vocabularies.
    root_source: DisplayName,
    /// The group id if present.
    group_id: Option<DisplayName>,
    /// The attribute count.
    attribute_count: u32,
    /// The relationship label.
    relation_label: DisplayName,
    /// The additional relationship label.
    additional_relation_label: Option<DisplayName>,
    /// The related concept ID URL.
    related_id: Info<WithoutClient, types::Concept, url::Url>,
    /// The related concept ID name.
    related_id_name: DisplayName,
}

impl TryFrom<RelationDto> for ConceptRelation {
    type Error = Error;

    fn try_from(value: RelationDto) -> Result<Self, Self::Error> {
        if !matches!(value.class_type, ClassType::ConceptRelation) {
            return Err(Error::NotConceptRelation);
        }

        let ui = value.ui;
        let suppressible = value.suppressible;
        let source_ui = value.source_ui;
        let obsolete = value.obsolete;
        let source_originated = value.source_originated;
        let root_source = value.root_source;
        let group_id = value.group_id;
        let attribute_count = value.attribute_count;
        let relation_label = value.relation_label;
        let additional_relation_label = value.additional_relation_label;
        let related_id =
            Info::<_, types::Concept, _>::new(value.related_id.ok_or(Error::MissingRelatedId)?);
        let related_id_name = value.related_id_name.ok_or(Error::MissingRelatedIdName)?;

        Ok(Self {
            ui,
            suppressible,
            source_ui,
            obsolete,
            source_originated,
            root_source,
            group_id,
            attribute_count,
            relation_label,
            additional_relation_label,
            related_id,
            related_id_name,
        })
    }
}

impl ConceptRelation {
    /// Returns the relationship unique identifier (RUI).
    pub fn ui(&self) -> &str {
        self.ui.as_ref()
    }

    /// Returns whether the relation is suppressible.
    pub fn suppressible(&self) -> bool {
        self.suppressible
    }

    /// Returns the source-asserted unique identifier, if present.
    pub fn source_ui(&self) -> Option<&DisplayName> {
        self.source_ui.as_ref()
    }

    /// Returns whether the relation is marked as obsolete.
    pub fn obsolete(&self) -> bool {
        self.obsolete
    }

    /// Returns whether the relation originated from the source vocabulary.
    pub fn source_originated(&self) -> bool {
        self.source_originated
    }

    /// Returns the root source vocabulary (e.g., `"MTH"`, `"SNOMEDCT_US"`).
    pub fn root_source(&self) -> &DisplayName {
        &self.root_source
    }

    /// Returns the relation group identifier, if present.
    pub fn group_id(&self) -> Option<&DisplayName> {
        self.group_id.as_ref()
    }

    /// Returns the total number of attributes associated with this relation.
    pub fn attribute_count(&self) -> u32 {
        self.attribute_count
    }

    /// Returns the relationship label (e.g., `"RO"`, `"PAR"`, `"CHD"`).
    pub fn relation_label(&self) -> &DisplayName {
        &self.relation_label
    }

    /// Returns the additional relationship label (e.g., `"isa"`, `"part_of"`), if present.
    pub fn additional_relation_label(&self) -> Option<&DisplayName> {
        self.additional_relation_label.as_ref()
    }

    /// Returns the [`Info`] handle for the target concept, containing its URL.
    pub fn related_id(&self) -> &Info<WithoutClient, types::Concept, url::Url> {
        &self.related_id
    }

    /// Returns the human-readable name of the target concept.
    pub fn related_id_name(&self) -> &DisplayName {
        &self.related_id_name
    }
}

/// An atom-cluster-level UMLS relation, linking two source-asserted atoms via a typed relationship.
///
/// Produced by calling [`RelationDto::try_into_atom_cluster_relation`].
pub struct AtomClusterRelation {
    ui: DisplayName,
    suppressible: bool,
    source_ui: Option<DisplayName>,
    obsolete: bool,
    source_originated: bool,
    root_source: DisplayName,
    group_id: Option<DisplayName>,
    attribute_count: u32,
    related_from_id: Info<WithoutClient, types::Source, Url>,
    related_from_id_name: DisplayName,
    relation_label: DisplayName,
    additional_relation_label: Option<DisplayName>,
    related_id: Info<WithoutClient, types::Source, Url>,
    related_id_name: DisplayName,
}

impl AtomClusterRelation {
    /// Returns the relationship unique identifier (RUI).
    pub fn ui(&self) -> &DisplayName {
        &self.ui
    }

    /// Returns whether the relation is suppressible.
    pub fn suppressible(&self) -> bool {
        self.suppressible
    }

    /// Returns the source-asserted unique identifier, if present.
    pub fn source_ui(&self) -> Option<&DisplayName> {
        self.source_ui.as_ref()
    }

    /// Returns whether the relation is marked as obsolete.
    pub fn obsolete(&self) -> bool {
        self.obsolete
    }

    /// Returns whether the relation originated from the source vocabulary.
    pub fn source_originated(&self) -> bool {
        self.source_originated
    }

    /// Returns the root source vocabulary (e.g., `"SNOMEDCT_US"`).
    pub fn root_source(&self) -> &DisplayName {
        &self.root_source
    }

    /// Returns the relation group identifier, if present.
    pub fn group_id(&self) -> Option<&DisplayName> {
        self.group_id.as_ref()
    }

    /// Returns the total number of attributes associated with this relation.
    pub fn attribute_count(&self) -> u32 {
        self.attribute_count
    }

    /// Returns the [`Info`] handle for the originating source atom, containing its URL.
    pub fn related_from_id(&self) -> &Info<WithoutClient, types::Source, Url> {
        &self.related_from_id
    }

    /// Returns the human-readable name of the originating source atom.
    pub fn related_from_id_name(&self) -> &DisplayName {
        &self.related_from_id_name
    }

    /// Returns the relationship label (e.g., `"PAR"`, `"RO"`).
    pub fn relation_label(&self) -> &DisplayName {
        &self.relation_label
    }

    /// Returns the additional relationship label (e.g., `"inverse_isa"`), if present.
    pub fn additional_relation_label(&self) -> Option<&DisplayName> {
        self.additional_relation_label.as_ref()
    }

    /// Returns the [`Info`] handle for the target source atom, containing its URL.
    pub fn related_id(&self) -> &Info<WithoutClient, types::Source, Url> {
        &self.related_id
    }

    /// Returns the human-readable name of the target source atom.
    pub fn related_id_name(&self) -> &DisplayName {
        &self.related_id_name
    }
}

impl TryFrom<RelationDto> for AtomClusterRelation {
    type Error = Error;

    fn try_from(value: RelationDto) -> Result<Self, Self::Error> {
        if value.class_type != ClassType::AtomClusterRelation {
            return Err(Error::NotAtomClusterRelation);
        }

        let ui = value.ui;
        let suppressible = value.suppressible;
        let source_ui = value.source_ui;
        let obsolete = value.obsolete;
        let source_originated = value.source_originated;
        let root_source = value.root_source;
        let group_id = value.group_id;
        let attribute_count = value.attribute_count;
        let related_from_id = Info::<_, types::Source, _>::new(
            value.related_from_id.ok_or(Error::MissingRelatedFromId)?,
        );
        let related_from_id_name = value
            .related_from_id_name
            .ok_or(Error::MissingRelatedFromName)?;
        let relation_label = value.relation_label;
        let additional_relation_label = value.additional_relation_label;
        let related_id =
            Info::<_, types::Source, _>::new(value.related_id.ok_or(Error::MissingRelatedId)?);
        let related_id_name = value.related_id_name.ok_or(Error::MissingRelatedIdName)?;

        Ok(AtomClusterRelation {
            ui,
            suppressible,
            source_ui,
            obsolete,
            source_originated,
            root_source,
            group_id,
            attribute_count,
            related_from_id,
            related_from_id_name,
            relation_label,
            additional_relation_label,
            related_id,
            related_id_name,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_relation_dto() {
        let json = json!({
          "ui": "R162928707",
          "suppressible": false,
          "sourceUi": "780251000124125",
          "obsolete": false,
          "sourceOriginated": false,
          "rootSource": "SNOMEDCT_US",
          "groupId": "0",
          "attributeCount": 0,
          "classType": "AtomClusterRelation",
          "relatedFromId": "https://uts-ws.nlm.nih.gov/rest/content/2022AA/source/SNOMEDCT_US/9468002",
          "relatedFromIdName": "Closed fracture of carpal bone",
          "relationLabel": "PAR",
          "additionalRelationLabel": "inverse_isa",
          "relatedId": "https://uts-ws.nlm.nih.gov/rest/content/2022AA/source/SNOMEDCT_US/10937761000119101",
          "relatedIdName": "Closed fracture of left wrist"
        });

        let relation = serde_json::from_value::<SerializableRelation>(json)
            .expect("The valid Relation JSON should be deserialize into `RelationDto`.");
        assert_eq!(relation.ui.as_str(), "R162928707");
        assert!(!relation.suppressible);
        assert_eq!(
            relation.source_ui.as_ref().map(|x| x.as_str()),
            Some("780251000124125")
        );
        assert!(!relation.obsolete);
        assert!(!relation.source_originated);
        assert_eq!(relation.root_source.as_str(), "SNOMEDCT_US");
        assert_eq!(
            relation.related_from_id.as_ref().map(|x| x.as_str()),
            Some("https://uts-ws.nlm.nih.gov/rest/content/2022AA/source/SNOMEDCT_US/9468002")
        );
        assert_eq!(
            relation.related_from_id_name.as_ref().map(|x| x.as_str()),
            Some("Closed fracture of carpal bone")
        );
        assert_eq!(relation.relation_label.as_str(), "PAR");
        assert_eq!(
            relation
                .additional_relation_label
                .as_ref()
                .map(|x| x.as_str()),
            Some("inverse_isa")
        );
        assert_eq!(
            relation.related_id.as_ref().map(|x| x.as_str()),
            Some(
                "https://uts-ws.nlm.nih.gov/rest/content/2022AA/source/SNOMEDCT_US/10937761000119101"
            )
        );
        assert_eq!(
            relation.related_id_name.as_ref().map(|x| x.as_str()),
            Some("Closed fracture of left wrist")
        );
    }

    #[test]
    fn deserialize_atom_cluster_relation_son() {
        let json = json!({
            "ui": "R210609007",
            "suppressible": false,
            "sourceUi": "15029049022",
            "obsolete": false,
            "sourceOriginated": false,
            "rootSource": "SNOMEDCT_US",
            "groupId": "NONE",
            "attributeCount": 0,
            "classType": "AtomClusterRelation",
            "relatedFromId": "https://uts-ws.nlm.nih.gov/rest/content/2026AB/source/SNOMEDCT_US/49727002",
            "relatedFromIdName": "Cough",
            "relationLabel": "RO",
            "additionalRelationLabel": "cause_of",
            "relatedId": "https://uts-ws.nlm.nih.gov/rest/content/2026AB/source/SNOMEDCT_US/427146008",
            "relatedIdName": "Pain provoked by coughing"
        });
        let relation = serde_json::from_value::<SerializableRelation>(json).expect("The valid JSON atom cluster relation class type object should be deserialize to `RelationDto`");
        assert_eq!(relation.ui.as_str(), "R210609007");
        assert!(!relation.suppressible);
        assert_eq!(
            relation.source_ui.as_ref().map(|x| x.as_str()),
            Some("15029049022")
        );
        assert!(!relation.obsolete);
        assert!(!relation.source_originated);
        assert_eq!(relation.root_source.as_str(), "SNOMEDCT_US");
        assert_eq!(relation.group_id, None);
        assert_eq!(relation.attribute_count, 0);
        assert_eq!(relation.class_type, ClassType::AtomClusterRelation);
        assert_eq!(
            relation.related_from_id.as_ref().map(|x| x.as_str()),
            Some("https://uts-ws.nlm.nih.gov/rest/content/2026AB/source/SNOMEDCT_US/49727002")
        );
        assert_eq!(
            relation.related_from_id_name.as_ref().map(|x| x.as_str()),
            Some("Cough")
        );
        assert_eq!(relation.relation_label.as_str(), "RO");
        assert_eq!(
            relation
                .additional_relation_label
                .as_ref()
                .map(|x| x.as_str()),
            Some("cause_of")
        );
        assert_eq!(
            relation.related_id.as_ref().map(|x| x.as_str()),
            Some("https://uts-ws.nlm.nih.gov/rest/content/2026AB/source/SNOMEDCT_US/427146008")
        );
        assert_eq!(
            relation.related_id_name.as_ref().map(|x| x.as_str()),
            Some("Pain provoked by coughing")
        );
    }

    #[test]
    fn deserialize_concept_relation() {
        let json = json!({
            "ui": "R71518533",
            "suppressible": false,
            "sourceUi": "NONE",
            "obsolete": false,
            "sourceOriginated": false,
            "rootSource": "MTH",
            "groupId": "NONE",
            "attributeCount": 0,
            "classType": "ConceptRelation",
            "relationLabel": "RO",
            "additionalRelationLabel": "",
            "relatedId": "https://uts-ws.nlm.nih.gov/rest/content/2026AB/CUI/C1961131",
            "relatedIdName": "Cough, CTCAE"
        });

        let _ = serde_json::from_value::<SerializableRelation>(json).expect("The valid JSON atom cluster relation class type object should be deserialize to `RelationDto`");
    }

    fn relation_dto(class_type: ClassType) -> RelationDto {
        RelationDto {
            ui: "R113844952".parse().unwrap(),
            suppressible: false,
            source_ui: None,
            obsolete: false,
            source_originated: false,
            root_source: "OMIM".parse().unwrap(),
            group_id: None,
            attribute_count: 0,
            class_type,
            related_from_id: Url::parse(
                "https://uts-ws.nlm.nih.gov/rest/content/2026AB/AUI/A11947203",
            )
            .ok(),
            related_from_id_name: Some("Cough".parse().unwrap()),
            relation_label: "RO".parse().unwrap(),
            additional_relation_label: Some("manifestation_of".parse().unwrap()),
            related_id: Url::parse("https://uts-ws.nlm.nih.gov/rest/content/2026AB/AUI/A13291437")
                .ok(),
            related_id_name: Some("PULMONARY ALVEOLAR PROTEINOSIS, ACQUIRED".parse().unwrap()),
        }
    }

    #[test]
    fn convert_relation_dto_to_atom_relation() {
        let relation = relation_dto(ClassType::AtomRelation);
        let object: AtomRelation = relation.try_into().expect(
            "The AtomRelation class type object should be able to conver to `AtomRelation`.",
        );
        assert_eq!(object.ui().as_str(), "R113844952");
        assert!(!object.suppressible());
        assert!(object.source_ui().is_none());
        assert!(!object.obsolete());
        assert!(!object.source_originated());
        assert_eq!(object.root_source().as_str(), "OMIM");
        assert!(object.group_id().is_none());
        assert_eq!(object.attribute_count(), 0);
        assert_eq!(
            object.related_from_id().value().as_str(),
            "https://uts-ws.nlm.nih.gov/rest/content/2026AB/AUI/A11947203"
        );
        assert_eq!(object.related_from_id_name().as_str(), "Cough");
        assert_eq!(object.relation_label().as_str(), "RO");
        assert_eq!(
            object
                .additional_relation_label()
                .as_ref()
                .map(|x| x.as_str()),
            Some("manifestation_of")
        );
        assert_eq!(
            object.related_id().value().as_str(),
            "https://uts-ws.nlm.nih.gov/rest/content/2026AB/AUI/A13291437"
        );
        assert_eq!(
            object.related_id_name().as_str(),
            "PULMONARY ALVEOLAR PROTEINOSIS, ACQUIRED"
        );
    }

    #[test]
    fn rejects_conversion_on_invalid_class_type() {
        assert_relation_dto_to_atom_relation(
            |r| r.class_type = ClassType::AtomClusterRelation,
            Error::NotAtomRelation,
        );

        assert_try_into_concept_relation_failure(
            |r| r.class_type = ClassType::AtomClusterRelation,
            Error::NotConceptRelation,
        );

        assert_try_into_atom_cluster_relation_failure(
            |r| r.class_type = ClassType::ConceptRelation,
            Error::NotAtomClusterRelation,
        );
    }

    #[test]
    fn rejects_conversion_on_missing_related_from_id() {
        assert_relation_dto_to_atom_relation(
            |r| r.related_from_id = None,
            Error::MissingRelatedFromId,
        );
        assert_relation_dto_conversion_failure::<AtomClusterRelation, _>(
            ClassType::AtomClusterRelation,
            |r| r.related_from_id = None,
            Error::MissingRelatedFromId,
        );
    }

    #[test]
    fn rejects_conversion_on_missing_related_from_name() {
        assert_relation_dto_to_atom_relation(
            |relation| relation.related_from_id_name = None,
            Error::MissingRelatedFromName,
        );
        assert_relation_dto_conversion_failure::<AtomClusterRelation, _>(
            ClassType::AtomClusterRelation,
            |r| r.related_from_id_name = None,
            Error::MissingRelatedFromName,
        );
    }

    fn assert_relation_dto_to_atom_relation<F>(function: F, error: Error)
    where
        F: FnOnce(&mut RelationDto),
    {
        assert_relation_dto_conversion_failure::<AtomRelation, _>(
            ClassType::AtomRelation,
            function,
            error,
        );
    }

    fn assert_relation_dto_conversion_failure<T, F: FnOnce(&mut RelationDto)>(
        class_type: ClassType,
        function: F,
        error: Error,
    ) where
        T: TryFrom<RelationDto, Error = Error>,
    {
        let mut relation = relation_dto(class_type);
        function(&mut relation);
        let result: Result<T, Error> = relation.try_into();
        assert!(matches!(result, Err(ref e) if *e == error));
    }

    fn assert_try_into_concept_relation_failure<F>(function: F, error: Error)
    where
        F: FnOnce(&mut RelationDto),
    {
        assert_relation_dto_conversion_failure::<ConceptRelation, _>(
            ClassType::ConceptRelation,
            function,
            error,
        );
    }

    fn assert_try_into_atom_cluster_relation_failure<F>(function: F, error: Error)
    where
        F: FnOnce(&mut RelationDto),
    {
        assert_relation_dto_conversion_failure::<AtomClusterRelation, _>(
            ClassType::AtomClusterRelation,
            function,
            error,
        );
    }

    #[test]
    fn rejects_conversion_on_missing_related_id() {
        assert_relation_dto_to_atom_relation(|r| r.related_id = None, Error::MissingRelatedId);
        assert_try_into_concept_relation_failure(|r| r.related_id = None, Error::MissingRelatedId);
        assert_relation_dto_conversion_failure::<AtomClusterRelation, _>(
            ClassType::AtomClusterRelation,
            |r| r.related_id = None,
            Error::MissingRelatedId,
        );
    }

    #[test]
    fn rejects_conversion_on_missing_related_id_name() {
        assert_relation_dto_to_atom_relation(
            |r| r.related_id_name = None,
            Error::MissingRelatedIdName,
        );
        assert_try_into_concept_relation_failure(
            |r| r.related_id_name = None,
            Error::MissingRelatedIdName,
        );
        assert_relation_dto_conversion_failure::<AtomClusterRelation, _>(
            ClassType::AtomClusterRelation,
            |r| r.related_id_name = None,
            Error::MissingRelatedIdName,
        );
    }
}
