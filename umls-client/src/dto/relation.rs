use crate::{dto::serde::optional_string, info::Info, types};
use serde::{Deserialize, Serialize};
use umls_http::WithoutClient;

/// FIXME: The object can't be created outside of the crate. Add builder to create this object.
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
    pub related_from_id: Info<WithoutClient, types::Source, url::Url>,
    /// From name
    pub related_from_id_name: String,
    /// The relationship label.
    pub relation_label: String,
    /// The additional relationship label.
    #[serde(with = "optional_string")]
    pub additional_relation_label: Option<String>,
    /// The related id.
    pub related_id: Info<WithoutClient, types::Source, url::Url>,
    /// The related id name
    pub related_id_name: String,
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

        let relation = serde_json::from_value::<RelationDto>(json)
            .expect("The valid Relation JSON should be deserialize into `RelationDto`.");
        assert_eq!(relation.ui.as_str(), "R162928707");
        assert!(!relation.suppressible);
        assert_eq!(relation.source_ui.as_deref(), Some("780251000124125"));
        assert!(!relation.obsolete);
        assert!(!relation.source_originated);
        assert_eq!(relation.root_source, "SNOMEDCT_US");
        assert_eq!(
            relation.related_from_id.value().as_str(),
            "https://uts-ws.nlm.nih.gov/rest/content/2022AA/source/SNOMEDCT_US/9468002"
        );
        assert_eq!(
            relation.related_from_id_name,
            "Closed fracture of carpal bone"
        );
        assert_eq!(relation.relation_label, "PAR");
        assert_eq!(
            relation.additional_relation_label.as_deref(),
            Some("inverse_isa")
        );
        assert_eq!(
            relation.related_id.value().as_str(),
            "https://uts-ws.nlm.nih.gov/rest/content/2022AA/source/SNOMEDCT_US/10937761000119101"
        );
        assert_eq!(relation.related_id_name, "Closed fracture of left wrist");
    }
}
