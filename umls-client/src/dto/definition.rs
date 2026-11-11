use serde::{Deserialize, Serialize};

use crate::dto::display_name::DisplayName;

/// Represents possible the de/serializable class type.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum ClassType {
    /// A difinition class type.
    Definition,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct RawDefinition {
    #[serde(rename = "classType")]
    class_type: ClassType,

    #[serde(rename = "sourceOriginated")]
    source_originated: bool,

    #[serde(rename = "rootSource")]
    root_source: DisplayName,

    value: DisplayName,
}

/// The metadata of the respecitve definition.
pub struct MetaData {
    source_originated: bool,
    root_source: DisplayName,
}

impl MetaData {
    fn new(source_originated: bool, root_source: DisplayName) -> Self {
        Self {
            source_originated,
            root_source,
        }
    }

    /// Returns the definition is source originated or not.
    pub fn is_source_originated(&self) -> bool {
        self.source_originated
    }

    /// Returns the root source vocabulary of the definition.
    pub fn root_source(&self) -> &DisplayName {
        &self.root_source
    }
}

/// The definition object.
pub struct Definition {
    value: DisplayName,
    metadata: MetaData,
}

impl Definition {
    fn new(value: DisplayName, metadata: MetaData) -> Self {
        Self { value, metadata }
    }

    /// Returns human readable representation of definition.
    pub fn value(&self) -> &DisplayName {
        &self.value
    }

    /// Returns the reference to metadata.
    pub fn metadata(&self) -> &MetaData {
        &self.metadata
    }
}

/// Represents unsupported class type.
#[derive(Debug, thiserror::Error)]
#[error("Unsupported class type")]
pub struct UnsupportedClassType;

impl TryFrom<RawDefinition> for Definition {
    type Error = UnsupportedClassType;

    fn try_from(value: RawDefinition) -> Result<Self, Self::Error> {
        if value.class_type != ClassType::Definition {
            return Err(UnsupportedClassType);
        }

        let metadata = MetaData::new(value.source_originated, value.root_source);
        let definition = Definition::new(value.value, metadata);

        Ok(definition)
    }
}
