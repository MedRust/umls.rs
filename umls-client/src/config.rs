use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use url::Url;

#[derive(Debug)]
/// Runtime configuration used for UMLS HTTP requests.
pub struct Config {
    client: reqwest::Client,
    base_url: Url,
    api_key: SecretString,
    version: String,
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
    pub fn version(&self) -> &str {
        &self.version
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
    version: String,
}

impl Builder {
    /// Creates a builder with the default endpoint and `current` release.
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://uts-ws.nlm.nih.gov/rest".parse().unwrap(),
            api_key: api_key.into(),
            version: "current".into(),
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
    pub fn version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    /// Validates the builder and creates a [`Config`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyApiKey`] when the API key is empty and
    /// [`Error::EmptyVersion`] when the version is empty.
    pub fn build(self) -> Result<Config, Error> {
        if self.api_key.expose_secret().is_empty() {
            return Err(Error::EmptyApiKey);
        }

        if self.version.is_empty() {
            return Err(Error::EmptyVersion);
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
    /// The configured UMLS release was empty.
    #[error("Empty Version")]
    EmptyVersion,
}
