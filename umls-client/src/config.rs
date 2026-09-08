use std::{borrow::Cow, num::ParseIntError, str::FromStr};

use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use url::Url;

#[derive(Debug)]
/// Runtime configuration used for UMLS HTTP requests.
pub struct Config {
    client: reqwest::Client,
    base_url: Url,
    api_key: SecretString,
    version: Version,
}

impl Config {
    /// Starts a configuration builder with the supplied UMLS API key.
    ///
    /// The builder defaults to the public UMLS REST endpoint and the
    /// `current` release.
    pub fn builder(api_key: impl Into<SecretString>) -> Builder {
        Builder::new(api_key)
    }

    /// Returns the HTTP client used to send requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Returns the base URL used to construct UMLS endpoints.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Returns the requested UMLS release, such as `current` or `2025AB`.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns the protected UMLS API key.
    ///
    /// The secret is not exposed unless callers explicitly use
    /// [`secrecy::ExposeSecret`].
    pub fn api_key(&self) -> &SecretString {
        &self.api_key
    }
}

/// Builder for [`Config`].
pub struct Builder {
    client: reqwest::Client,
    base_url: Url,
    api_key: SecretString,
    version: Version,
}

impl Builder {
    /// Creates a builder with the default endpoint and `current` release.
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://uts-ws.nlm.nih.gov/rest/".parse().unwrap(),
            api_key: api_key.into(),
            version: Version::Current,
        }
    }

    /// Replaces the HTTP client used for requests.
    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    /// Replaces the UMLS REST base URL.
    ///
    /// This is primarily useful for proxies and test servers.
    /// When the URL contains a path prefix, include a trailing `/` so relative
    /// atom endpoints are appended to that prefix. For example, use
    /// `https://uts-ws.nlm.nih.gov/rest/`, not
    /// `https://uts-ws.nlm.nih.gov/rest`. The builder stores the URL as supplied.
    /// [`Self::build`] rejects URLs whose paths do not end in `/`.
    ///
    /// # Example
    ///
    /// ```
    /// use umls_client::config::Config;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::builder("example-api-key")
    ///     .base_url("https://example.com/proxy/rest/".parse()?)
    ///     .build()?;
    /// let endpoint = config.base_url().join("content/current/CUI/C0010200/atoms")?;
    /// assert_eq!(endpoint.path(), "/proxy/rest/content/current/CUI/C0010200/atoms");
    /// # Ok(())
    /// # }
    /// ```
    pub fn base_url(mut self, base_url: Url) -> Self {
        self.base_url = base_url;
        self
    }

    /// Replaces the API key.
    pub fn api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = api_key.into();
        self
    }

    /// Selects the UMLS release used in endpoint paths.
    ///
    /// Defaults to `current`. CUI-based concept atom requests, including
    /// preferred-atom requests, use this release. Atom requests built from an
    /// explicit URL retain the release in that URL.
    ///
    /// Construct a release with [`Version::Release`] or use [`Version::Current`].
    /// [`Self::build`] rejects release years before 2018. It does not currently
    /// enforce an upper year bound or check publication availability.
    ///
    /// # Example
    ///
    /// ```
    /// use umls_client::config::{Config, ReleaseType, Version};
    ///
    /// let config = Config::builder("example-api-key")
    ///     .version(Version::Release { year: 2018, kind: ReleaseType::AA })
    ///     .build()?;
    /// assert_eq!(config.version().to_string(), "2018AA");
    /// # Ok::<(), umls_client::config::Error>(())
    /// ```
    pub fn version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Selects the UMLS release used in endpoint paths.
    ///
    /// Defaults to `current`. CUI-based concept atom requests, including
    /// preferred-atom requests, use this release. Atom requests built from an
    /// explicit URL retain the release in that URL.
    ///
    /// Construct a release with [`Version::Release`] or use [`Version::Current`].
    /// [`Self::build`] rejects release years before 2018. It does not currently
    /// enforce an upper year bound or check publication availability.
    ///
    /// # Example
    ///
    /// ```
    /// use umls_client::config::{Config, ReleaseType, Version};
    ///
    /// let config = Config::builder("example-api-key")
    ///     .try_version("2018AA")?
    ///     .build()?;
    /// assert_eq!(config.version().to_string(), "2018AA");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_version(
        mut self,
        version: impl TryInto<Version, Error = InvalidVersion>,
    ) -> Result<Self, InvalidVersion> {
        self.version = version.try_into()?;
        Ok(self)
    }

    /// Validates the builder and creates a [`Config`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyApiKey`] when the API key is empty. Returns
    /// [`Error::MissingBaseUrlTrailingSlash`] when the base URL path does not
    /// end in `/`. Returns [`Error::UnsupportedYear`] for explicit release
    /// years before 2018.
    ///
    /// Validation is local and does not make HTTP requests.
    pub fn build(self) -> Result<Config, Error> {
        if self.api_key.expose_secret().is_empty() {
            return Err(Error::EmptyApiKey);
        }

        if !self.base_url.path().ends_with('/') {
            return Err(Error::MissingBaseUrlTrailingSlash);
        }

        if let Version::Release { year, .. } = self.version
            && year < 2018
        {
            return Err(Error::UnsupportedYear(year));
        }

        Ok(Config {
            client: self.client,
            base_url: self.base_url,
            api_key: self.api_key,
            version: self.version,
        })
    }
}

#[derive(Debug, Error)]
/// An error produced while validating client configuration.
pub enum Error {
    /// The configured UMLS API key was empty.
    #[error("Empty API Key")]
    EmptyApiKey,
    /// The explicit release year was earlier than 2018.
    #[error("Unsupported year: {0}")]
    UnsupportedYear(u16),
    /// The configured base URL path did not end in `/`.
    #[error("Base URL path must end with a trailing '/'")]
    MissingBaseUrlTrailingSlash,
}

#[derive(Debug, thiserror::Error)]
/// An error encountered while parsing a UMLS version string.
pub enum InvalidVersion {
    /// The input length does not match the parser's expected release length.
    #[error("The length should be 5 character long")]
    InvalidLength,

    #[error("Year parse error: {0}")]
    /// The year could not be parsed as an unsigned integer.
    YearParser(ParseIntError),

    #[error("Release type parser error: {0}")]
    /// The release suffix could not be parsed.
    ReleaseTypeParser(InvalidRelaseType),
}

#[derive(Debug, thiserror::Error)]
/// An error encountered while parsing a release suffix.
pub enum InvalidRelaseType {
    /// The input failed the release suffix length check.
    #[error("The length should be 2 character long")]
    InvalidLength,

    #[error("Unsupported release type: {0}")]
    /// The input was not a supported release suffix.
    Unsupported(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// The AA or AB suffix of a UMLS release identifier.
pub enum ReleaseType {
    /// The AA release within a year.
    AA,
    /// The AB release within a year.
    AB,
}

impl ReleaseType {
    /// Returns the uppercase release suffix used in endpoint paths.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AA => "AA",
            Self::AB => "AB",
        }
    }
}

impl FromStr for ReleaseType {
    type Err = InvalidRelaseType;

    /// Parses a release suffix.
    ///
    /// # Errors
    ///
    /// The current implementation rejects every input, including `AA` and
    /// `AB`, because its length check is inverted. Use enum variants directly
    /// until parsing is corrected.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(InvalidRelaseType::InvalidLength);
        }

        match s {
            "AA" => Ok(Self::AA),
            "AB" => Ok(Self::AB),
            other => Err(InvalidRelaseType::Unsupported(other.into())),
        }
    }
}

impl std::fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for ReleaseType {
    type Error = InvalidRelaseType;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<&str> for ReleaseType {
    type Error = InvalidRelaseType;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// A request for the current UMLS release or an explicitly selected release.
///
/// The type restricts release suffixes to [`ReleaseType`]. Construction itself
/// does not validate the year; [`Builder::build`] rejects years before 2018.
/// No upper year bound or publication check is currently applied.
///
/// # Example
///
/// ```
/// use umls_client::config::{Config, ReleaseType, Version};
///
/// let config = Config::builder("example-api-key")
///     .try_version("2018AB")?
///     .build()?;
/// match config.version() {
///     Version::Current => println!("Using the current release"),
///     Version::Release { year, kind } => {
///         // Matching a borrowed version gives &u16 and &ReleaseType.
///         assert_eq!(year, 2018);
///         assert_eq!(kind.as_str(), "AB");
///     }
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub enum Version {
    /// Resolve the release through the API's `current` alias.
    Current,
    /// Select a release by year and suffix.
    Release {
        /// The release year; validated against the 2018 minimum when building
        /// a configuration.
        year: u16,
        /// The AA or AB suffix.
        kind: ReleaseType,
    },
}

impl Version {
    /// Formats the version for an endpoint path.
    ///
    /// `Current` borrows `current`; an explicit release allocates its formatted
    /// year and suffix. Years are formatted without zero padding.
    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            Self::Current => Cow::Borrowed("current"),
            Self::Release { year, kind } => Cow::Owned(format!("{}{}", year, kind)),
        }
    }
}

impl FromStr for Version {
    type Err = InvalidVersion;

    /// Parses the case-insensitive `current` alias.
    ///
    /// Explicit release parsing is currently incomplete: six-byte identifiers
    /// such as `2018AA` are rejected. Construct [`Version::Release`] directly
    /// for pinned versions.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidVersion`] when a length, year, or suffix check fails.
    ///
    /// # Panics
    ///
    /// A seven-byte input panics if byte index four is not a UTF-8 character
    /// boundary. Avoid passing unvalidated non-ASCII input until this is fixed.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "current" {
            return Ok(Version::Current);
        }

        if s.len() != 6 {
            return Err(InvalidVersion::InvalidLength);
        }

        let (year, release_type) = s.split_at(4);

        let year: u16 = year.parse().map_err(InvalidVersion::YearParser)?;
        let kind: ReleaseType = release_type
            .parse()
            .map_err(InvalidVersion::ReleaseTypeParser)?;

        Ok(Version::Release { year, kind })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for Version {
    type Error = InvalidVersion;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<&str> for Version {
    type Error = InvalidVersion;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {

    use std::num::IntErrorKind;

    use super::*;

    #[test]
    fn convert_current_version_to_string() {
        let version = Version::Current.as_str();
        assert_eq!(version, "current");
    }

    #[test]
    fn convert_explicity_release_version_to_string() {
        let version = Version::Release {
            year: 2021,
            kind: ReleaseType::AA,
        };
        assert_eq!(version.as_str(), "2021AA");
    }

    #[test]
    fn invalid_version_parser_with_invalid_length_string() {
        let result = "VERSION2020AA".parse::<Version>();
        assert!(matches!(result, Err(InvalidVersion::InvalidLength)));
    }

    #[test]
    fn invalid_version_parser_with_invalid_year() {
        let result = "AA2018".parse::<Version>();
        match result {
            Err(InvalidVersion::YearParser(e)) => {
                assert!(matches!(e.kind(), IntErrorKind::InvalidDigit));
            }
            other => panic!("Expected year parser error, but got {:?}", other),
        }
    }

    #[test]
    fn invalid_version_perser_with_unsupported_release_type() {
        let result = "2021BA".parse::<Version>();
        match result {
            Err(InvalidVersion::ReleaseTypeParser(e)) => {
                assert!(matches!(e, InvalidRelaseType::Unsupported(..)));
            }
            other => panic!("Expected release type parser error, but got {:?}", other),
        }
    }

    #[test]
    fn convert_release_type_from_string() -> Result<(), Box<dyn std::error::Error>> {
        let release_type: ReleaseType = "AA".parse()?;
        assert_eq!(release_type, ReleaseType::AA);

        let release_type: ReleaseType = "AB".parse()?;
        assert_eq!(release_type, ReleaseType::AB);

        Ok(())
    }

    #[test]
    fn default_configuration() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::builder("apiKey").build()?;
        assert_eq!(config.version(), Version::Current);
        assert_eq!(config.api_key().expose_secret(), "apiKey");
        assert_eq!(
            config.base_url().as_str(),
            "https://uts-ws.nlm.nih.gov/rest/"
        );

        Ok(())
    }

    #[test]
    fn failed_on_version_year_less_then_2018() -> Result<(), Box<dyn std::error::Error>> {
        let result = Config::builder("apiKey").try_version("2017AA")?.build();

        match result {
            Err(Error::UnsupportedYear(year)) => {
                assert_eq!(year, 2017);
            }
            other => panic!("Expected unsupported year, but got {:?}", other),
        }

        Ok(())
    }
}
