use crate::dto::serde::optional_string;
use serde::{Deserialize, Serialize};

/// A relationship DTO object.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationDto {
    /// UMLS Relationship Unique Identifier (RUI)
    pub ui: String,
    /// Represents respective object is suppressible.
    pub suppressible: bool,
    /// The source unique identifier if present.
    #[serde(with = "optional_string")]
    pub source_ui: Option<String>,
    /// Represents respective object is obsolete.
    pub obsolete: bool,
    /// Represents respective object is source originated.
    pub source_originated: bool,
    /// The root source vocabularies.
    pub root_source: String,
    /// The grou id if present.
    #[serde(with = "optional_string")]
    pub group_id: Option<String>,
    /// The attribute count.
    pub attribute_count: u32,
    /// The class type.
    pub class_type: String,
    /// From Id
    pub related_from_id: String,
    /// From name
    pub related_from_id_name: String,
    /// The relationship label.
    pub relation_label: String,
    /// The additional relationship label.
    #[serde(with = "optional_string")]
    pub additional_relation_label: Option<String>,
    /// The related id.
    pub related_id: String,
    /// The related id name
    pub related_id_name: String,
}
