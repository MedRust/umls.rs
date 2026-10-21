use crate::search::core::DisplayName;

fn is_valid_aui(value: &str) -> bool {
    [7, 8].contains(&value.len())
        && value.starts_with('A')
        && value[1..].bytes().all(|byte| byte.is_ascii_digit())
}

/// An Invalid format error.
#[derive(Debug, thiserror::Error)]
#[error("Invalid format: {0}")]
pub struct InvalidFormat(String);

/// An atom unique identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aui(DisplayName);

impl Aui {
    /// Create new atom unique identifier for given object.
    pub fn new(aui: DisplayName) -> Result<Self, InvalidFormat> {
        if !is_valid_aui(aui.as_str()) {
            return Err(InvalidFormat(aui.as_str().into()));
        }

        Ok(Aui(aui))
    }

    /// Returns a reference of [`DisplayName`].
    pub fn display_name(&self) -> &DisplayName {
        &self.0
    }

    /// Returns a string representation of the object.
    pub fn as_str(&self) -> &str {
        self.display_name().as_str()
    }
}

impl AsRef<str> for Aui {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<DisplayName> for Aui {
    type Error = InvalidFormat;

    fn try_from(value: DisplayName) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::fmt::Display for Aui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
