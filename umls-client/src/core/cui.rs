use core::str;
use std::str::FromStr;

use crate::search::core::{EmptyString, get_trimmed_non_empty_string};

#[derive(Debug, thiserror::Error)]
/// An error produced while validating a [`crate::core::cui::Cui`].
pub enum Error {
    /// The supplied identifier was empty after trimming.
    #[error("Empty String Error")]
    EmptyString,

    /// The supplied identifier did not match `C` followed by seven digits.
    #[error("invalid CUI format: {0}")]
    InvalidFormat(String),
}

impl From<EmptyString> for Error {
    fn from(_value: EmptyString) -> Self {
        Self::EmptyString
    }
}

fn is_valid_cui(value: &str) -> bool {
    value.len() == 8
        && value.starts_with('C')
        && value[1..].bytes().all(|byte| byte.is_ascii_digit())
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A validated UMLS Concept Unique Identifier.
///
/// CUIs consist of an uppercase `C` followed by seven ASCII digits.
pub struct Cui(String);

impl Cui {
    /// Creates a CUI after trimming surrounding whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyString`] for an empty value or
    /// [`Error::InvalidFormat`] when the identifier has an invalid shape.
    ///
    /// # Example
    ///
    /// ```
    /// use umls_client::core::cui::Cui;
    ///
    /// let cui = Cui::new(" C0010200 ")?;
    /// assert_eq!(cui.as_str(), "C0010200");
    /// # Ok::<(), umls_client::core::cui::Error>(())
    /// ```
    pub fn new(value: &str) -> Result<Self, Error> {
        let trimmed = get_trimmed_non_empty_string(value)?;

        if !is_valid_cui(trimmed) {
            return Err(Error::InvalidFormat(trimmed.into()));
        }

        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the validated identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Cui {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Cui {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<&str> for Cui {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Cui {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl std::fmt::Display for Cui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_cui() {
        let cui = Cui::new("C0010200").unwrap();

        assert_eq!(cui.as_str(), "C0010200");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        for value in [" C0010200", "C0010200 ", "  C0010200  ", "\tC0010200\n"] {
            let cui = Cui::new(value).unwrap();

            assert_eq!(cui.as_str(), "C0010200");
        }
    }

    #[test]
    fn rejects_empty_cui() {
        assert!(matches!(Cui::new(""), Err(Error::EmptyString)));
    }

    #[test]
    fn rejects_whitespace_only_cui() {
        for value in [" ", "   ", "\t", "\n", " \t\n "] {
            assert!(
                matches!(Cui::new(value), Err(Error::EmptyString)),
                "{value:?} should be rejected as empty"
            );
        }
    }

    #[test]
    fn rejects_wrong_prefix() {
        for value in ["A0010200", "S0010200", "c0010200", "X0010200", "00010200"] {
            assert!(
                matches!(Cui::new(value), Err(Error::InvalidFormat(_))),
                "{value} should have been rejected"
            );
        }
    }

    #[test]
    fn rejects_incorrect_length() {
        for value in ["C", "C0", "C001020", "C00102000", "C000000000"] {
            assert!(
                matches!(Cui::new(value), Err(Error::InvalidFormat(_))),
                "{value} should have been rejected"
            );
        }
    }

    #[test]
    fn rejects_non_digit_characters() {
        for value in [
            "C001O200", // letter O instead of zero
            "C00102A0", "C001-200", "C001 200", "C001.200", "C001_200",
        ] {
            assert!(
                matches!(Cui::new(value), Err(Error::InvalidFormat(_))),
                "{value} should have been rejected"
            );
        }
    }

    #[test]
    fn rejects_non_ascii_digits() {
        let result = Cui::new("C１２３４５６７");

        assert!(matches!(result, Err(Error::InvalidFormat(_))));
    }

    #[test]
    fn parses_cui_from_string_slice() {
        let cui: Cui = "  C0010200  ".parse().unwrap();

        assert_eq!(cui.as_str(), "C0010200");
    }

    #[test]
    fn converts_from_borrowed_string() {
        let cui = Cui::try_from("  C0010200  ").unwrap();

        assert_eq!(cui.as_str(), "C0010200");
    }

    #[test]
    fn converts_from_owned_string() {
        let cui = Cui::try_from("  C0010200  ".to_owned()).unwrap();

        assert_eq!(cui.as_str(), "C0010200");
    }

    #[test]
    fn exposes_cui_through_as_ref() {
        let cui = Cui::new("C0010200").unwrap();

        assert_eq!(cui.as_ref(), "C0010200");
    }

    #[test]
    fn formats_cui_for_display() {
        let cui = Cui::new("C0010200").unwrap();

        assert_eq!(cui.to_string(), "C0010200");
    }

    #[test]
    fn validates_cui_format() {
        assert!(is_valid_cui("C0010200"));
        assert!(is_valid_cui("C0000000"));

        assert!(!is_valid_cui("C001020"));
        assert!(!is_valid_cui("c0010200"));
        assert!(!is_valid_cui("C00102A0"));
    }
}
