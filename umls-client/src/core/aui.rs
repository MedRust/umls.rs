use crate::search::core::DisplayName;

fn is_valid_aui(value: &DisplayName) -> bool {
    let value = value.as_str();
    [8, 9].contains(&value.len())
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
        if !is_valid_aui(&aui) {
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn validate_aui() {
        let strings = [
            "A1234567",
            "A12345678",
            "  A1234567",
            "A1234567   ",
            "    A1234567    ",
        ];
        for value in strings {
            let value = is_valid_aui(&value.try_into().unwrap());
            assert!(value);
        }

        let invalid_strings = ["1234567A", "A12345", "A123456789", "1234A56"];
        for invalid_string in invalid_strings {
            let value = is_valid_aui(&invalid_string.try_into().unwrap());
            assert!(!value);
        }
    }

    #[test]
    fn create_new_aui() {
        let strings = ["A1234567", "A12345678"];
        for value in strings {
            let result = Aui::new(value.try_into().unwrap());
            match result {
                Ok(aui) => {
                    assert_eq!(aui.as_str(), value);
                }
                Err(_) => panic!("Invalid aui format: {}", value),
            }
        }

        let invalid_strings = ["1234567A", "A123456", "A123456789", "1234A56", "1234A567"];
        for invalid_string in invalid_strings {
            let result = Aui::new(invalid_string.try_into().unwrap());
            assert!(matches!(result, Err(InvalidFormat(_))));
        }
    }

    #[test]
    fn format_representation() {
        for string in ["A1234567", "A12345678"] {
            let display_name: DisplayName = string.parse().unwrap();
            let aui = Aui::new(display_name).unwrap();
            let display_string = format!("{}", aui);
            assert_eq!(display_string, string);
        }
    }

    #[test]
    fn as_ref_presentation() {
        for string in ["A1234567", "A12345678"] {
            let display_name: DisplayName = string.parse().unwrap();
            let aui = Aui::new(display_name).unwrap();
            assert_eq!(aui.as_ref(), string);
        }
    }
}
