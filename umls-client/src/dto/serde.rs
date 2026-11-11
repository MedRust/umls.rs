/// De/serializes action for string to optional string.
///
/// In deserialization the module validate the if it's not `NONE` and convert it to `Option::None`.
/// else return string.
///
/// In serialization if optional string is `Option::None` then conver to `NONE` and serialize the
/// string.
pub mod optional_string {
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes the Optional String to specified type.
    pub fn serialize<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value.as_deref().unwrap_or("NONE"))
    }

    /// Deserializes the string to Optional string
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        Ok(match value {
            Some(val) if val == "NONE" => None,
            other => other,
        })
    }
}

/// De/serialize boolean value as string
pub mod bool_as_string {
    use serde::{Deserialize, Deserializer, Serializer, de};

    /// Serialize boolean value to string.
    pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(if *value { "true" } else { "false" })
    }

    /// Deserialize string to boolean.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(de::Error::custom("expected \"true\" or \"false\"")),
        }
    }
}
