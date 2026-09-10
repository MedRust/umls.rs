//! Describes release categories and fetches the categories advertised by UTS.

use serde::{Deserialize, Serialize};
#[cfg(feature = "reqwest")]
use umls_http::reqwest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Describes a release category advertised by the UTS Release API.
///
/// A category identifies a product and the endpoint used to list its downloadable releases.
/// The endpoint is returned as supplied by the service, without URL validation.
///
/// # Examples
///
/// ```
/// use umls_downloader::release_type::ReleaseType;
/// let category = ReleaseType {
///     product: "UMLS".into(),
///     name: "umls-full-release".into(),
///     endpoint: "https://uts-ws.nlm.nih.gov/releases?releaseType=umls-full-release".into(),
/// };
/// assert_eq!(category.product(), "UMLS");
/// ```
pub struct ReleaseType {
    /// The product to which this release category belongs, such as UMLS or RxNorm.
    pub product: String,
    /// The value accepted by the Release API's `releaseType` query parameter.
    #[serde(rename = "releaseType")]
    pub name: String,
    /// The Release API endpoint for releases in this category.
    pub endpoint: String,
}

impl std::fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.product, self.name)
    }
}

impl ReleaseType {
    /// Returns the product name.
    ///
    /// The returned string is borrowed from the category and is not normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::release_type::ReleaseType;
    /// let category = ReleaseType {
    ///     product: "UMLS".into(),
    ///     name: "umls-full-release".into(),
    ///     endpoint: "https://uts-ws.nlm.nih.gov/releases".into(),
    /// };
    /// let value = category.product();
    /// assert!(!value.is_empty());
    /// ```
    pub fn product(&self) -> &str {
        &self.product
    }

    /// Returns the Release API's release-type identifier.
    ///
    /// The returned identifier can select releases when constructing a [`crate::ReleaseFetcher`].
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::release_type::ReleaseType;
    /// let category = ReleaseType {
    ///     product: "UMLS".into(),
    ///     name: "umls-full-release".into(),
    ///     endpoint: "https://uts-ws.nlm.nih.gov/releases".into(),
    /// };
    /// let value = category.name();
    /// assert!(!value.is_empty());
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the endpoint used to fetch releases in this category.
    ///
    /// The returned string is borrowed without validating its syntax or adding query parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use umls_downloader::release_type::ReleaseType;
    /// let category = ReleaseType {
    ///     product: "UMLS".into(),
    ///     name: "umls-full-release".into(),
    ///     endpoint: "https://uts-ws.nlm.nih.gov/releases".into(),
    /// };
    /// let value = category.endpoint();
    /// assert!(!value.is_empty());
    /// ```
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReleaseTypes {
    #[serde(rename = "releaseTypes")]
    types: Vec<ReleaseType>,
}

/// Fetches release categories using an attached HTTP client.
///
/// Construction starts without a client. Attaching a supported client enables fetching; the
/// returned categories identify the products and release endpoints advertised by UTS.
///
/// # Examples
///
/// ```
/// use umls_downloader::release_type::Client;
/// let builder = Client::default();
/// ```
pub struct Client<C> {
    client: C,
}

impl Default for Client<umls_http::WithoutClient> {
    fn default() -> Self {
        Self {
            client: umls_http::WithoutClient,
        }
    }
}

impl Client<umls_http::WithoutClient> {
    /// Attaches an HTTP client and returns the resulting release-category client.
    ///
    /// This operation consumes the builder without making a network request. Fetching is available
    /// when the attached type has a supported implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "reqwest")]
    /// # {
    /// use umls_downloader::{release_type::Client};
    /// use umls_http::reqwest;
    /// let client = Client::default().with_client(reqwest::Client::default());
    /// # }
    /// ```
    pub fn with_client<T>(self, client: T) -> Client<T> {
        Client { client }
    }
}

#[cfg(feature = "reqwest")]
impl Client<reqwest::Client> {
    /// Fetches the release categories advertised by UTS.
    ///
    /// Each call makes a new request. An empty category list produces an empty [`Vec`]. The client
    /// remains available for subsequent requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use umls_downloader::{release_type::Client};
    /// use umls_http::reqwest;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::default().with_client(reqwest::Client::default());
    /// for category in client.fetch().await? {
    ///     println!("{}: {}", category.product(), category.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if request construction, serialization, transport, HTTP 4xx or 5xx status
    /// validation, or response decoding fails. Failures preserve typed client errors
    /// in [`crate::Error::Client`].
    pub async fn fetch(&self) -> Result<Vec<ReleaseType>, crate::Error<reqwest::Error>> {
        self.fetch_from_uri(
            "https://uts-ws.nlm.nih.gov/releases"
                .parse()
                .expect("valid release endpoint"),
        )
        .await
    }

    async fn fetch_from_uri(
        &self,
        uri: http::Uri,
    ) -> Result<Vec<ReleaseType>, crate::Error<reqwest::Error>> {
        let request = http::Request::builder()
            .uri(uri)
            .method(http::Method::GET)
            .body(())
            .map_err(reqwest::Error::Http)
            .map_err(crate::Error::Client)?;

        self.client
            .execute::<(), ReleaseTypes>(request)
            .await
            .map(|x| x.into_body())
            .map(|x| x.types)
            .map_err(crate::Error::Client)
    }
}

#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn discovery_decodes_categories_and_empty_lists() {
        let server = MockServer::start_async().await;
        for types in [
            serde_json::json!([]),
            serde_json::json!([{
                "product":"UMLS", "releaseType":"umls-full-release",
                "endpoint":"https://example.test/releases?custom=kept"
            }]),
        ] {
            let mock = server
                .mock_async(|when, then| {
                    when.method(GET).path("/releases");
                    then.json_body(serde_json::json!({"releaseTypes":types}));
                })
                .await;
            let client = Client::default().with_client(reqwest::Client::default());
            let result = client
                .fetch_from_uri(server.url("/releases").parse().unwrap())
                .await
                .unwrap();
            assert_eq!(serde_json::to_value(result).unwrap(), types);
            mock.assert_async().await;
            mock.delete_async().await;
        }
    }
}
