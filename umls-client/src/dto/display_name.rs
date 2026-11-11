use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
#[error("Empty string error")]
/// Indicates that a textual value was empty after trimming whitespace.
pub struct EmptyString;

/// Trims a string and verifies that at least one non-whitespace character remains.
///
/// # Errors
///
/// Returns [`EmptyString`] when `value` is empty after trimming.
pub fn get_trimmed_non_empty_string(value: &str) -> Result<&str, EmptyString> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmptyString);
    }
    Ok(value)
}

/// A human-readable display name returned by the UMLS API.
///
/// Unlike UMLS identifiers, a display name is intended for presentation and
/// does not uniquely identify a concept or source vocabulary entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DisplayName(String);

impl DisplayName {
    /// Creates a display name after trimming surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    pub fn try_new(value: &str) -> Result<Self, EmptyString> {
        get_trimmed_non_empty_string(value).map(|x| Self(x.into()))
    }

    /// Returns the display name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the display name and returns its underlying string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for DisplayName {
    /// Returns the display name as a string slice.
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for DisplayName {
    /// Formats the human-readable display name.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DisplayName {
    type Err = EmptyString;

    /// Parses a display name after trimming surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_new(value)
    }
}

impl TryFrom<&str> for DisplayName {
    type Error = EmptyString;

    /// Creates a display name from a borrowed string.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for DisplayName {
    type Error = EmptyString;

    /// Creates a display name from an owned string.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyString`] when `value` is empty or contains only
    /// whitespace.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(&value)
    }
}

impl<'de> serde::Deserialize<'de> for DisplayName {
    /// Deserializes and validates a display name.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_new(&value).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for DisplayName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}
