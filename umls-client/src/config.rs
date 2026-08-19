use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use url::Url;

pub struct Config {
    client: reqwest::Client,
    base_url: Url,
    api_key: SecretString,
    version: String,
}

impl Config {
    pub fn builder(api_key: impl Into<SecretString>) -> Builder {
        Builder::new(api_key)
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn api_key(&self) -> &SecretString {
        &self.api_key
    }
}

pub struct Builder {
    client: reqwest::Client,
    base_url: Url,
    api_key: SecretString,
    version: String,
}

impl Builder {
    pub fn new(api_key: impl Into<SecretString>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://uts-ws.nlm.nih.gov/rest".parse().unwrap(),
            api_key: api_key.into(),
            version: "current".into(),
        }
    }

    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    pub fn base_url(mut self, base_url: Url) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn api_key(mut self, api_key: impl Into<SecretString>) -> Self {
        self.api_key = api_key.into();
        self
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

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
pub enum Error {
    #[error("Empty API Key")]
    EmptyApiKey,
    #[error("Empty Version")]
    EmptyVersion,
}
