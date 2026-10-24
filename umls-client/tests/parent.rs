//! An integration tests for Parent API's.
#![cfg(feature = "reqwest")]

use httpmock::{Method::GET, MockServer};
use umls_client::{
    config,
    parent::{Error, Parent},
};
use url::Url;

#[tokio::test]
async fn fetch_atom_parents() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/AUI/A10134087/parents")
                .query_param_exists("apiKey");

            let value = include_str!("fixtures/atom_parents.json");
            let body: serde_json::Value = serde_json::from_str(value).unwrap();
            then.status(200).json_body(body);
        })
        .await;

    let client = umls_http::reqwest::Client::default();

    let base_url: Url = mock_server.base_url().parse()?;
    let url = base_url.join("content/current/AUI/A10134087/parents")?;

    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .version(config::Version::Current)
        .build()?;

    let pages = Parent::new(url)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    assert_eq!(pages.size().get(), 25);
    assert_eq!(pages.number().get(), 1);
    assert_eq!(pages.content().list().len(), 1);

    let response = pages.content().list().first().unwrap();
    assert_eq!(response.ui.as_str(), "A19720766");
    assert!(!response.suppressible);
    assert!(!response.obsolete);
    assert_eq!(response.root_source.as_str(), "HL7V3.0");
    assert_eq!(response.term_type.as_str(), "CSY");
    assert_eq!(response.language.as_str(), "ENG");
    assert_eq!(response.name.as_str(), "RouteOfAdministration");

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn validation_url_object() -> Result<(), Box<dyn std::error::Error>> {
    let client = umls_http::reqwest::Client::default();

    let base_url: Url = "https://uts-ws.nlm.nih.gov/rest/".parse()?;
    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let urls: Vec<Url> = vec![
        "http://localhost:8080/rest/content/current/AUI/A10134087/parents".parse()?,
        "https://localhost:8080/rest/content/current/AUI/A10134087/parents".parse()?,
        "https://uts-ws.nlm.nih.gov:8080/rest/content/current/AUI/A10134087/parents".parse()?,
    ];

    for url in urls {
        let result = Parent::new(url)
            .with_client(client.clone())
            .fetch(&umls_config)
            .await;

        assert!(matches!(
            result,
            Err(Error::<umls_http::reqwest::Error>::Validation(_))
        ));
    }

    Ok(())
}
