//! UMLS endpoint URL validation.
//!
//! This module provides validation for UMLS endpoint URLs against a [`Config`].
//! It ensures that incoming or constructed URLs target the expected base URL,
//! adhere to the expected path schema, and contain valid identifiers (such as
//! [`Cui`] or [`Aui`]) without invalid trailing segments.
//!
//! Validators are typically accessed through [`Config`] helper methods:
//! - [`Config::info_url_validator`]
//! - [`Config::atom_url_validator`]
//! - [`Config::definition_url_validator`]
//! - [`Config::relation_url_validator`]
//! - [`Config::parent_url_validator`]
//!
//! # Example
//!
//! ```
//! use umls_client::config::Config;
//! use url::Url;
//!
//! let config = Config::builder("test-api-key").build()?;
//! let url: Url = "https://uts-ws.nlm.nih.gov/rest/content/current/CUI/C0010200".parse()?;
//!
//! let validator = config.info_url_validator::<umls_client::types::Concept>(&url);
//! assert!(validator.validate().is_ok());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::types;
use std::marker::PhantomData;

use url::Url;

use crate::{config::Config, core::aui::Aui, dto::display_name::DisplayName};

/// Validate url against with the base url.
///
/// # Errors
/// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
/// - [`Error::MismatchHost`] - When url and base url host mismatch.
/// - [`Error::CannotBeABase`] - When url is cannot be a base.
/// - [`Error::InvalidPath`] - When url path mismatch.
fn validate_against_base_url<'a>(
    base_url: &'a url::Url,
    url: &'a url::Url,
) -> Result<core::str::Split<'a, char>, Error> {
    if base_url.cannot_be_a_base() || url.cannot_be_a_base() {
        return Err(Error::CannotBeABase);
    }

    if base_url.scheme() != url.scheme() {
        return Err(Error::MismatchScheme);
    }

    if base_url.host_str() != url.host_str() {
        return Err(Error::MismatchHost);
    }

    if base_url.port_or_known_default() != url.port_or_known_default() {
        return Err(Error::MismatchPort);
    }

    let Some(mut segments) = url.path_segments() else {
        return Err(Error::CannotBeABase);
    };

    let Some(base_segments) = base_url.path_segments() else {
        return Err(Error::CannotBeABase);
    };

    for base_segment in base_segments.filter(|s| !s.is_empty()) {
        if segments.next() != Some(base_segment) {
            return Err(Error::InvalidPath);
        }
    }

    Ok(segments)
}

struct Validator<'a> {
    split: core::str::Split<'a, char>,
}

impl<'a> Validator<'a> {
    fn try_validation_against_base_url(
        base_url: &'a url::Url,
        url: &'a Url,
    ) -> Result<Self, Error> {
        validate_against_base_url(base_url, url).map(|x| Validator { split: x })
    }

    fn static_text_check(mut self, text: &str) -> Result<Self, Error> {
        self.split
            .next()
            .ok_or(Error::InvalidPath)
            .map(|x| x == text)
            .and_then(|x| x.ok_or(Error::InvalidPath))
            .map(|_| self)
    }

    fn umls_version_check(mut self) -> Result<Self, Error> {
        self.split.next().ok_or(Error::InvalidPath).and_then(|x| {
            if let Err(e) = x.parse::<crate::config::Version>() {
                return Err(Error::Version(e));
            }
            Ok(self)
        })
    }

    fn cui_check(mut self) -> Result<Self, Error> {
        self.split.next().ok_or(Error::InvalidPath).and_then(|x| {
            if let Err(e) = x.parse::<crate::core::cui::Cui>() {
                return Err(Error::Cui(e));
            }
            Ok(self)
        })
    }

    fn non_empty_check(mut self) -> Result<Self, Error> {
        self.split.next().ok_or(Error::InvalidPath).and_then(|x| {
            if x.is_empty() {
                return Err(Error::InvalidPath);
            }
            Ok(self)
        })
    }

    fn aui_check(mut self) -> Result<Self, Error> {
        self.split
            .next()
            .ok_or(Error::InvalidPath)
            .and_then(|x| match x.parse::<DisplayName>() {
                Ok(value) => {
                    if let Err(e) = Aui::new(value) {
                        return Err(Error::Aui(e));
                    }
                    Ok(self)
                }
                Err(_) => Err(Error::InvalidPath),
            })
    }

    fn concept_url_validator(base_url: &'a Url, url: &'a Url) -> Result<Self, Error> {
        Validator::try_validation_against_base_url(base_url, url)?
            .static_text_check("content")?
            .umls_version_check()?
            .static_text_check("CUI")?
            .cui_check()
    }

    fn source_url_validator(base_url: &'a Url, url: &'a Url) -> Result<Self, Error> {
        Validator::try_validation_against_base_url(base_url, url)?
            .static_text_check("content")?
            .umls_version_check()?
            .static_text_check("source")?
            .non_empty_check()?
            .non_empty_check()
    }

    fn finish(self) -> Result<(), Error> {
        let mut split = self.split;
        if split.next().is_some() {
            return Err(Error::Incomplete);
        }
        Ok(())
    }
}

/// Represents Info url validator.
pub struct Info<'a, T> {
    config: &'a Config,
    url: &'a Url,
    _type: PhantomData<T>,
}

impl<'a, T> Info<'a, T> {
    /// Creates new [`Info`] validator object.
    pub fn new(config: &'a Config, url: &'a Url) -> Self {
        Self {
            config,
            url,
            _type: PhantomData,
        }
    }

    /// Returns a UMLS configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns an url.
    pub fn url(&self) -> &'a Url {
        self.url
    }
}

/// Represents url validation error.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Represents mismatch scheme
    #[error("Mismatch scheme")]
    MismatchScheme,

    /// Represents mismatch host
    #[error("Mismatch host")]
    MismatchHost,

    /// Represents url is cannot be a base.
    #[error("Cannot be a base")]
    CannotBeABase,

    /// Represents invalid path.
    #[error("Invalid path")]
    InvalidPath,

    /// Represents Invalid version format.
    #[error("Invalid version format: {0}")]
    Version(#[source] crate::config::InvalidVersion),

    /// Represents invalid cui format
    #[error("Invalid CUI format: {0}")]
    Cui(#[source] crate::core::cui::Error),

    /// Represents invalid AUI format.
    #[error("Invalid AUI format error: {0}")]
    Aui(#[source] crate::core::aui::InvalidFormat),

    /// Represents incomplete validation of url.
    #[error("Incomplete validation of url")]
    Incomplete,

    /// Represents mismatch port
    #[error("Mismatch port")]
    MismatchPort,
}

impl<'a> Info<'a, types::Concept> {
    /// Validate concept info url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::Cui`] - Invalid CUI format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::concept_url_validator(self.config().base_url(), self.url())?.finish()
    }
}

impl<'a> Info<'a, types::Source> {
    /// Validate source concept info url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::source_url_validator(self.config().base_url(), self.url())?.finish()
    }
}

/// Represents atom url validator.
pub struct Atom<'a, M, T> {
    config: &'a Config,
    url: &'a Url,
    _mode: PhantomData<M>,
    _type: PhantomData<T>,
}

impl<'a, M, T> Atom<'a, M, T> {
    /// Creates new [`Atom`] object.
    pub fn new(config: &'a Config, url: &'a Url) -> Self {
        Self {
            config,
            url,
            _mode: PhantomData,
            _type: PhantomData,
        }
    }

    /// Returns configuration object.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns url object.
    pub fn url(&self) -> &'a Url {
        self.url
    }
}

impl<'a> Atom<'a, types::Concept, types::atom::All> {
    /// Validate concept atoms url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::Cui`] - Invalid CUI format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::concept_url_validator(self.config().base_url(), self.url())?
            .static_text_check("atoms")?
            .finish()
    }
}

impl<'a> Atom<'a, super::types::Concept, super::types::atom::Preferred> {
    /// Validate concept preferred atom url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::Cui`] - Invalid CUI format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::concept_url_validator(self.config().base_url(), self.url())?
            .static_text_check("atoms")?
            .static_text_check("preferred")?
            .finish()
    }
}

/// Represents definition url validator.
pub struct Definition<'a, T> {
    config: &'a Config,
    url: &'a Url,
    _type: PhantomData<T>,
}

impl<'a, T> Definition<'a, T> {
    /// Create new [`Definition`] object.
    pub fn new(config: &'a Config, url: &'a Url) -> Self {
        Self {
            config,
            url,
            _type: PhantomData,
        }
    }

    /// Returns configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns url.
    pub fn url(&self) -> &'a Url {
        self.url
    }
}

impl<'a> Definition<'a, types::Concept> {
    /// Validate concept definitions url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::Cui`] - Invalid CUI format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::concept_url_validator(self.config().base_url(), self.url())?
            .static_text_check("definitions")?
            .finish()
    }
}

/// Represents relation url validator.
pub struct Relation<'a, T> {
    config: &'a Config,
    url: &'a Url,
    _type: PhantomData<T>,
}

impl<'a, T> Relation<'a, T> {
    /// Creates new [`Relation`] object.
    pub fn new(config: &'a Config, url: &'a Url) -> Self {
        Self {
            config,
            url,
            _type: PhantomData,
        }
    }

    /// Returns configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns url.
    pub fn url(&self) -> &'a Url {
        self.url
    }
}

impl<'a> Relation<'a, super::types::Concept> {
    /// Validate concept relations url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::Cui`] - Invalid CUI format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::concept_url_validator(self.config().base_url(), self.url())?
            .static_text_check("relations")?
            .finish()
    }
}

/// Represents parent url validator.
pub struct Parent<'a, T> {
    config: &'a Config,
    url: &'a Url,
    _type: PhantomData<T>,
}

impl<'a, T> Parent<'a, T> {
    /// Creates new [`Parent`] object.
    pub fn new(config: &'a Config, url: &'a Url) -> Self {
        Self {
            config,
            url,
            _type: PhantomData,
        }
    }

    /// Returns configuration.
    pub fn config(&self) -> &'a Config {
        self.config
    }

    /// Returns url.
    pub fn url(&self) -> &'a Url {
        self.url
    }
}

impl<'a> Parent<'a, types::Source> {
    /// Validate concept source-asserted parent url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch or empty.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::source_url_validator(self.config().base_url(), self.url())?
            .static_text_check("parents")?
            .finish()
    }
}

impl<'a> Parent<'a, types::Atom> {
    /// Validate atom parent url.
    ///
    /// The method validate against the base url.
    ///
    /// # Errors
    /// - [`Error::MismatchScheme`] - When url and base url scheme mismatch.
    /// - [`Error::MismatchHost`] - When url and base url host mismatch.
    /// - [`Error::CannotBeABase`] - When url is cannot be a base.
    /// - [`Error::InvalidPath`] - When url path mismatch.
    /// - [`Error::Version`] - Invalid version format in url path.
    /// - [`Error::Aui`] - Invalid AUI format in url path.
    /// - [`Error::MismatchPort`] - When url port (if any) is mismatch.
    pub fn validate(&self) -> Result<(), Error> {
        Validator::try_validation_against_base_url(self.config().base_url(), self.url())?
            .static_text_check("content")?
            .umls_version_check()?
            .static_text_check("AUI")?
            .aui_check()?
            .static_text_check("parents")?
            .finish()
    }
}

#[cfg(test)]
mod tests {

    use crate::config;

    use super::*;

    #[test]
    fn validates_url_against_base_url() -> Result<(), Box<dyn std::error::Error>> {
        let base_url: url::Url = "http://localhost:8080/rest".parse()?;
        let url: url::Url =
            "http://localhost:8080/rest/content/current/CUI/C0155502/atoms".parse()?;

        _ = validate_against_base_url(&base_url, &url)?;

        Ok(())
    }

    #[test]
    fn rejects_invalid_scheme() -> Result<(), Box<dyn std::error::Error>> {
        let base_url: url::Url = "http://localhost:8080/rest".parse()?;
        let url: url::Url =
            "https://localhost:8080/rest/content/current/CUI/C0155502/atoms".parse()?;
        let result = validate_against_base_url(&base_url, &url);
        assert!(matches!(result, Err(Error::MismatchScheme)));
        Ok(())
    }

    #[test]
    fn rejects_mismatch_host() {
        let base_url: url::Url = "https://uts-ws.nlm.nih.gov/rest"
            .parse()
            .expect("The HTTP url string should be parsed into `url::Url`.");
        let url: url::Url = "https://host.new/rest"
            .parse()
            .expect("The HTTP url string should be parsed into `url::Url`");

        let result = validate_against_base_url(&base_url, &url);
        assert!(matches!(result, Err(Error::MismatchHost)));
    }

    #[test]
    fn rejects_base_url_that_cannot_be_a_base() {
        let base_url: url::Url = "data:text/plain,Hello?World#"
            .parse()
            .expect("The aribary string should be parsed into `url::Url`");
        let url: url::Url = "https://uts-ws.nlm.nih.gov/rest/content/current/CUI/C0155502/atoms"
            .parse()
            .expect("The HTTP url should be parsed into `url::Url`.");
        let result = validate_against_base_url(&base_url, &url);
        assert!(matches!(result, Err(Error::CannotBeABase)));
    }

    #[test]
    fn rejects_url_that_cannot_be_a_base() {
        let base_url: url::Url = "http://localhost:8080/rest"
            .parse()
            .expect("The HTTP url shoul dbe parsed into `url::Url`.");
        let url: url::Url = "data:text/plain,Hello?World#"
            .parse()
            .expect("The aribary string should be parsed into `url::Url`.");
        let result = validate_against_base_url(&base_url, &url);
        assert!(matches!(result, Err(Error::CannotBeABase)));
    }

    #[test]
    fn rejects_mismatch_base_url_path() {
        let base_url: url::Url = "http://localhost:8080/rest"
            .parse()
            .expect("The HTTP url should be parsed into `url::Url`.");
        let url: url::Url = "http://localhost:8080/api"
            .parse()
            .expect("The HTTP url should be parsed into `url::Url`.");
        let result = validate_against_base_url(&base_url, &url);
        assert!(matches!(result, Err(Error::InvalidPath)));
    }

    #[test]
    fn validates_concept_information_url() {
        let base_url: url::Url = "https://localhost:8080/rest"
            .parse()
            .expect("The HTTP url should be parsed into `url::Url`.");
        let url: url::Url = "https://localhost:8080/rest/content/current/CUI/C0009044"
            .parse()
            .expect("The HTTP url should be parsed into `url::Url`.");
        validate_against_base_url(&base_url, &url)
            .expect("The concept info url against base url should be successful.");
    }

    fn url_from_str(url: &str) -> url::Url {
        url.parse()
            .expect("The HTTP url should be parsed into `url::Url`.")
    }

    fn umls_config(base_url: &str) -> config::Config {
        let base_url = url_from_str(base_url);

        config::Config::builder("some_api_key")
            .base_url(base_url)
            .build()
            .expect("The UMLS configuration should be created for non empty api key.")
    }

    #[test]
    fn rejects_invalid_concept_info_path_segments() {
        let config = umls_config("http://localhost:8080/rest");

        let urls = [
            "http://localhost:8080/rest/chunk/current/CUI/C0009044",
            "http://localhost:8080/rest/content/current/AUI/C0009044",
        ];

        for url in urls {
            let url = url_from_str(url);
            let result = Info::<types::Concept>::new(&config, &url).validate();
            assert!(matches!(result, Err(Error::InvalidPath)));
        }
    }

    #[test]
    fn rejects_invalid_concept_info_url_path_version_format() {
        let config = umls_config("http://localhost:8080/rest");

        let url = url_from_str("http://localhost:8080/rest/content/AA2024/CUI/C0009044");
        let result = Info::<types::Concept>::new(&config, &url).validate();

        assert!(matches!(result, Err(Error::Version(_))));
    }

    #[test]
    fn rejects_invalid_concept_info_url_path_cui_format() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:8080/rest/content/current/CUI/0009044");
        let result = Info::<types::Concept>::new(&config, &url).validate();
        assert!(matches!(result, Err(Error::Cui(_))));
    }

    #[test]
    fn validates_concept_atom_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:8080/rest/content/current/CUI/C0009044/atoms");
        Atom::<types::Concept, types::atom::All>::new(&config, &url)
            .validate()
            .expect("Validation should be successful for valid url.");
    }

    #[test]
    fn rejects_invalid_concept_atom_url_path_segments() {
        let config = umls_config("http://localhost:8080/rest");
        let urls = [
            "http://localhost:8080/rest/chunks/current/CUI/C0009044/atoms",
            "http://localhost:8080/rest/content/current/AUI/C0009044/atoms",
            "http://localhost:8080/rest/content/current/CUI/C0009044/atom",
            "http://localhost:8080/rest/content/current/CUI/C0009044/aui",
        ];

        for url in urls {
            let url = url_from_str(url);
            let result = Atom::<types::Concept, types::atom::All>::new(&config, &url).validate();
            assert!(matches!(result, Err(Error::InvalidPath)));
        }
    }

    #[test]
    fn rejects_invalid_concept_atom_url_version_format() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:8080/rest/content/A2024A/CUI/C0009044/atoms");
        let result = Atom::<types::Concept, types::atom::All>::new(&config, &url).validate();
        assert!(matches!(result, Err(Error::Version(_))));
    }

    #[test]
    fn rejects_invalid_concept_atom_url_cui_format() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:8080/rest/content/current/CUI/00090C44/atoms");
        let result = Atom::<types::Concept, types::atom::All>::new(&config, &url).validate();
        assert!(matches!(result, Err(Error::Cui(_))));
    }

    #[test]
    fn validates_preferred_concept_atom_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url =
            url_from_str("http://localhost:8080/rest/content/current/CUI/C0155502/atoms/preferred");
        Atom::<types::Concept, types::atom::Preferred>::new(&config, &url)
            .validate()
            .expect("Validation of preferred concept atom url should be successful.");
    }

    #[test]
    fn rejects_invalid_preferred_concept_atom_url_path_segments() {
        let config = umls_config("http://localhost:8080/rest");
        let urls = [
            "http://localhost:8080/rest/chunks/current/CUI/C0155502/atoms/preferred",
            "http://localhost:8080/rest/content/current/CUI/C0155502/atoms/prefere",
            "http://localhost:8080/rest/content/current/AUI/C0155502/atoms/preferred",
            "http://localhost:8080/rest/content/current/CUI/C0155502/atom/preferred",
            "http://localhost:8080/rest/content/current/CUI/C0155502//preferred",
            "http://localhost:8080/rest/content/current/CUI/C0155502/atoms/",
            "http://localhost:8080/rest//current/CUI/C0155502/atoms/preferred",
        ];

        for url in urls {
            let url = url_from_str(url);
            let result =
                Atom::<types::Concept, types::atom::Preferred>::new(&config, &url).validate();
            assert!(matches!(result, Err(Error::InvalidPath)));
        }
    }

    #[test]
    fn rejects_invalid_preferred_concept_atom_url_version_format() {
        let config = umls_config("http://localhost:8080/rest");
        let url =
            url_from_str("http://localhost:8080/rest/content/2024/CUI/C0155502/atoms/preferred");
        let result = Atom::<types::Concept, types::atom::Preferred>::new(&config, &url).validate();
        assert!(matches!(result, Err(Error::Version(_))));
    }

    #[test]
    fn rejects_invalid_preferred_concept_atom_url_cui_format() {
        let config = umls_config("http://localhost:8080/rest");
        let url =
            url_from_str("http://localhost:8080/rest/content/current/CUI/0155502/atoms/preferred");
        let result = Atom::<types::Concept, types::atom::Preferred>::new(&config, &url).validate();
        assert!(matches!(result, Err(Error::Cui(_))));
    }

    #[test]
    fn validates_concept_definition_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url =
            url_from_str("http://localhost:8080/rest/content/current/CUI/C0155502/definitions");
        let is_success = Definition::<types::Concept>::new(&config, &url)
            .validate()
            .is_ok();
        assert!(is_success);
    }

    #[test]
    fn rejects_invalid_concept_definition_path_segments() {
        let config = umls_config("http://localhost:8080/rest");
        let urls = [
            "http://localhost:8080/rest/content/current/CUI/C0155502/definition",
            "http://localhost:8080/rest/content/current/CUI/C0155502/defs",
            "http://localhost:8080/rest/content/current/CUI/C0155502/",
        ];
        for url in urls {
            let url = url_from_str(url);
            let result = Definition::<types::Concept>::new(&config, &url).validate();
            assert!(matches!(result, Err(Error::InvalidPath)));
        }
    }

    #[test]
    fn validates_concept_relation_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:8080/rest/content/current/CUI/C0009044/relations");
        let is_success = Relation::new(&config, &url).validate().is_ok();
        assert!(is_success);
    }

    #[test]
    fn rejects_invalid_concept_relation_path_segments() {
        let config = umls_config("http://localhost:8080/rest");
        let urls = [
            "http://localhost:8080/rest/content/current/CUI/C0155502/relation",
            "http://localhost:8080/rest/content/current/CUI/C0155502/rela",
            "http://localhost:8080/rest/content/current/CUI/C0155502/rel",
            "http://localhost:8080/rest/content/current/CUI/C0155502/",
        ];
        for url in urls {
            let url = url_from_str(url);
            let result = Relation::new(&config, &url).validate();
            assert!(matches!(result, Err(Error::InvalidPath)));
        }
    }

    #[test]
    fn validates_source_asserted_parent_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str(
            "http://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002/parents",
        );
        let is_success = Parent::<types::Source>::new(&config, &url)
            .validate()
            .is_ok();
        assert!(is_success);
    }

    #[test]
    fn rejects_invalid_source_asserted_parent_path_segments() {
        let config = umls_config("http://localhost:8080/rest");
        let urls = [
            "http://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002/parent",
            "http://localhost:8080/rest/chunk/current/sor/SNOMEDCT_US/9468002/parents",
            "http://localhost:8080/rest//current/source/SNOMEDCT_US/9468002/parents",
            "http://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002/",
            "http://localhost:8080/rest/content/current//SNOMEDCT_US/9468002/parents",
            "http://localhost:8080/rest/content/current/src/SNOMEDCT_US/9468002/parents",
            "http://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002/par",
        ];
        for url in urls {
            let url = url_from_str(url);
            let result = Parent::<types::Source>::new(&config, &url).validate();
            assert!(
                matches!(result, Err(Error::InvalidPath)),
                "source-asserted validation not asserted with invalid path for url: {} with result: {:?}",
                url,
                result
            );
        }
    }

    #[test]
    fn validates_atom_parent_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:8080/rest/content/current/AUI/A10134087/parents");
        let is_success = Parent::<types::Atom>::new(&config, &url).validate().is_ok();
        assert!(is_success);
    }

    #[test]
    fn rejects_invalid_atom_parent_path_segments() {
        let config = umls_config("http://localhost:8080/rest");
        let urls = [
            "http://localhost:8080/rest/content/current/AUI/A10134087/parent",
            "http://localhost:8080/rest/content/current/AUI/A10134087/",
            "http://localhost:8080/rest/content/current/CUI/A10134087/parents",
            "http://localhost:8080/rest/chunk/current/AUI/A10134087/parents",
            "http://localhost:8080/rest//current/AUI/A10134087/parents",
            "http://localhost:8080/rest/content/current//A10134087/parents",
        ];
        for url in urls {
            let url = url_from_str(url);
            let result = Parent::<types::Atom>::new(&config, &url).validate();
            assert!(
                matches!(result, Err(Error::InvalidPath)),
                "source-asserted validation not asserted with invalid path for url: {} with result: {:?}",
                url,
                result
            );
        }
    }

    #[test]
    fn rejects_invalid_aui() {
        let base_url = url_from_str("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:8080/rest/content/current/CUI/C0009044/extra");
        let result = Validator::try_validation_against_base_url(&base_url, &url)
            .expect("Validation should be successful when base_url is same in url.")
            .static_text_check("content")
            .expect("Validation should be successful when path content is present")
            .umls_version_check()
            .expect("Validation should be successful when version is same")
            .static_text_check("CUI")
            .expect("Validation should be successful when next path segment is \"CUI\"")
            .cui_check()
            .expect("Validation should be successful when next path segment is valid CUI format")
            .finish();
        assert!(matches!(result, Err(Error::Incomplete)));
    }

    #[test]
    fn rejects_mismatch_port() {
        let base_url = url_from_str("http://localhost:8080/rest");
        let url = url_from_str("http://localhost:80/rest");
        let result = validate_against_base_url(&base_url, &url);
        assert!(matches!(result, Err(Error::MismatchPort)))
    }

    #[test]
    fn validate_port() {
        let base_url = url_from_str("https://example.com");
        let urls = ["https://example.com:443", "https://example.com"];
        for url in urls {
            let url = url_from_str(url);
            validate_against_base_url(&base_url, &url).expect("The port shuold be same.");
        }
    }

    #[test]
    fn validate_base_url_with_trailing_slash() {
        let config = umls_config("http://localhost:8080/rest/");
        let url = url_from_str("http://localhost:8080/rest/content/current/CUI/C0009044");
        Info::<types::Concept>::new(&config, &url).validate().expect("The validation should be successful when configuration with base url contains trailing slash");
    }

    #[test]
    fn validates_source_info_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url =
            url_from_str("http://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002");
        Info::<types::Source>::new(&config, &url)
            .validate()
            .expect("The validation should be successful for valid url.");
    }

    #[test]
    fn rejects_source_info_url_path_segments() {
        let config = umls_config("http://localhost:8080/rest");
        let urls = [
            "http://localhost:8080/rest/chunk/current/source/SNOMEDCT_US/9468002",
            "http://localhost:8080/rest//current/source/SNOMEDCT_US/9468002",
            "http://localhost:8080/rest/content/current//SNOMEDCT_US/9468002",
            "http://localhost:8080/rest/content/current/source//9468002",
            "http://localhost:8080/rest/content/current/source/SNOMEDCT_US/",
        ];

        for url in urls {
            let url = url_from_str(url);
            let result = Info::<types::Source>::new(&config, &url).validate();
            assert!(matches!(result, Err(Error::InvalidPath)));
        }
    }

    #[test]
    fn rejects_invalid_source_info_url_version_format() {
        let config = umls_config("http://localhost:8080/rest");
        let url =
            url_from_str("http://localhost:8080/rest/content/A2024A/source/SNOMEDCT_US/9468002");
        let result = Info::<types::Source>::new(&config, &url).validate();
        assert!(matches!(result, Err(Error::Version(_))));
    }

    #[test]
    fn rejects_mismatch_host_for_source_info_url() {
        let config = umls_config("http://localhost/rest");
        let url = url_from_str(
            "http://uts-ws.nlm.nih.gov/rest/content/current/source/SNOMEDCT_US/9468002",
        );
        let result = Info::<types::Source>::new(&config, &url).validate();
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::MismatchHost)));
    }

    #[test]
    fn rejects_mismatch_scheme_for_source_info_url() {
        let config = umls_config("http://uts-ws.nlm.nih.gov/rest");
        let url = url_from_str(
            "https://uts-ws.nlm.nih.gov/rest/content/current/source/SNOMEDCT_US/9468002",
        );
        let result = Info::<types::Source>::new(&config, &url).validate();
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::MismatchScheme)));
    }

    #[test]
    fn rejects_mismatch_port_for_source_info_url() {
        let config = umls_config("https://uts-ws.nlm.nih.gov/rest");
        let url = url_from_str(
            "https://uts-ws.nlm.nih.gov:8080/rest/content/current/source/SNOMEDCT_US/9468002",
        );
        let result = Info::<types::Source>::new(&config, &url).validate();
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::MismatchPort)));
    }

    #[test]
    fn rejects_incomplete_source_info_url() {
        let config = umls_config("http://localhost:8080/rest");
        let url = url_from_str(
            "http://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002/parents",
        );
        let result = Info::<types::Source>::new(&config, &url).validate();
        assert!(result.is_err());
        assert!(matches!(result, Err(Error::Incomplete)));
    }
}
