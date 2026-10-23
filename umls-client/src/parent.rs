#[cfg(feature = "reqwest")]
use secrecy::ExposeSecret;
#[cfg(feature = "reqwest")]
use url::Url;

#[cfg(feature = "reqwest")]
use crate::{
    atom::{AtomResponse, Page},
    config,
    search::Content,
};

/// UMLS parent client.
pub struct Parent<C, T> {
    client: C,
    value: T,
}

impl<T> Parent<umls_http::WithoutClient, T> {
    /// Create and return a new instance of [`Parent`] for given value.
    ///
    /// The client is created with http client [`umls_http::WithoutClient`].
    pub fn new(value: T) -> Self {
        Self {
            client: umls_http::WithoutClient,
            value,
        }
    }

    /// Create a new [`Parent`] object with specified client.
    pub fn with_client<C>(self, client: C) -> Parent<C, T> {
        Parent {
            client,
            value: self.value,
        }
    }
}

impl<C, T> Parent<C, T> {
    /// Returns a reference of client object.
    pub fn client(&self) -> &C {
        &self.client
    }

    /// Returns a reference of value object.
    pub fn value(&self) -> &T {
        &self.value
    }
}

#[cfg(feature = "reqwest")]
impl Parent<umls_http::reqwest::Client, Url> {
    /// Fetch parents of UMLS atom.
    ///
    /// The method doesn't validate the URL at runtime. The method only fullfill the requirements
    /// and call specified http client.
    pub async fn fetch(
        self,
        umls_config: &config::Config,
    ) -> Result<Page<Content<AtomResponse>>, Error<umls_http::reqwest::Error>> {
        let url = Url::parse_with_params(
            self.value().as_str(),
            &[("apiKey", umls_config.api_key().expose_secret().to_owned())],
        )
        .map_err(Error::Url)?;

        let request = http::Request::builder()
            .method("GET")
            .uri(url.as_str())
            .body(())
            .map_err(Error::Http)?;

        let response: http::Response<Page<Content<AtomResponse>>> = self
            .client()
            .execute(request)
            .await
            .map_err(Error::Client)?;

        Ok(response.into_body())
    }
}

#[cfg(feature = "reqwest")]
/// An error object to fetch data.
#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    /// A Client error.
    #[error("Client Error: {0}")]
    Client(T),

    /// Url Error.
    #[error("Url Error: {0}")]
    Url(url::ParseError),

    /// Http error.
    #[error("Http Error: {0}")]
    Http(http::Error),
}
