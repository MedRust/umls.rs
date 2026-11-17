//! Custom serialization and deserialization implementations.

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

/// Serializes and deserializes [`DisplayName`](crate::dto::display_name::DisplayName).
///
/// The module deserializes the [`String`] to optional [`DisplayName`](crate::dto::display_name::DisplayName).
/// The function validates the string is not "NONE" or not.
///
/// The module serializes the Optional [`DisplayName`](crate::dto::display_name::DisplayName) to
/// [`String`]. If the value is [`Option::None`] the function deserializes string to "NONE" string.
pub mod optional_display_name {
    use crate::dto::display_name::DisplayName;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serializes the Optional String to specified type.
    pub fn serialize<S>(value: &Option<DisplayName>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = value.as_ref().map(|x| x.as_str());
        serializer.serialize_str(value.unwrap_or("NONE"))
    }

    /// Deserializes the string to Optional string
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DisplayName>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        Ok(match value {
            Some(val) if val == "NONE" => None,
            Some(val) => val.parse::<DisplayName>().ok(),
            None => None,
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

/// Serializes to and deserialize from Optional [`url::Url`].
///
/// The module deserializes the [`String`] to Optional [`url::Url`]. The function validate if the
/// given string is "NONE" then return [`Option::None`] else parse the string to [`url::Url`].
///
/// The module serializes the optional [`url::Url`] to [`String`]. if the value is [`Option::None`]
/// then deserializes to "NONE" else string representation of [`url::Url`].
pub mod optional_url {
    use serde::{Deserialize, Deserializer, Serializer};
    use url::Url;

    /// Serializes the Optional String to specified type.
    pub fn serialize<S>(value: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = value.as_ref().map(|x| x.as_str()).unwrap_or("NONE");
        serializer.serialize_str(s)
    }

    /// Deserializes the string to Optional string
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<String>::deserialize(deserializer)?;
        Ok(match value {
            Some(val) if val == "NONE" => None,
            Some(val) => {
                let url: Url = val
                    .parse()
                    .map_err(|x| format!("URL parser error: {}", x))
                    .map_err(serde::de::Error::custom)?;
                Some(url)
            }
            None => None,
        })
    }
}
