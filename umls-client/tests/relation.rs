//! Test relations integration tests.
//!
#![cfg(feature = "reqwest")]

use httpmock::{Method::GET, MockServer};
use serde_json::Value;
use umls_client::{
    atom::Batch,
    config,
    relation::{Relation, config::Config},
};
use url::Url;

#[tokio::test]
async fn fetch_concept_relation_first_page() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/CUI/C0010200/relations")
                .query_param_exists("apiKey");
            let json_data = include_str!("fixtures/concept_relations.json");
            let json_value: Value = serde_json::from_str(json_data).unwrap();
            then.status(200).json_body_obj(&json_value);
        })
        .await;

    let base_url = mock_server.base_url().parse()?;
    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let client = umls_http::reqwest::Client::default();

    let cui = "C0010200".parse()?;
    let config = Config::builder(cui).build();

    let response = Relation::new(config)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    assert_eq!(response.list().len(), 25);

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn fetch_concept_relation_batch_pages() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock_page_1 = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/CUI/C0010200/relations")
                .query_param_exists("apiKey")
                .query_param("pageNumber", "1")
                .query_param("pageSize", "25");
            let json_data = include_str!("fixtures/concept_relations.json");
            let json_value: Value = serde_json::from_str(json_data).unwrap();
            then.status(200).json_body_obj(&json_value);
        })
        .await;

    let mock_page_2 = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/CUI/C0010200/relations")
                .query_param_exists("apiKey")
                .query_param("pageNumber", "2")
                .query_param("pageSize", "25");
            let json_data = include_str!("fixtures/concept_relations.json");
            let json_value: Value = serde_json::from_str(json_data).unwrap();
            then.status(200).json_body_obj(&json_value);
        })
        .await;

    let base_url = mock_server.base_url().parse()?;
    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let client = umls_http::reqwest::Client::default();

    let cui = "C0010200".parse()?;
    let config = Config::builder(cui).build();
    let batch = Batch::new(25.try_into()?, 1.try_into()?, config);

    Relation::new(batch)
        .with_client(client)
        .fetch(&umls_config)
        .await?
        .into_page()
        .next()?
        .fetch(&umls_config)
        .await?;

    mock_page_1.assert_async().await;
    mock_page_2.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn fetch_concept_relation_using_url() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/CUI/C0010200/relations")
                .query_param_exists("apiKey");
            let json_data = include_str!("fixtures/concept_relations.json");
            let json_value: Value = serde_json::from_str(json_data).unwrap();
            then.status(200).json_body_obj(&json_value);
        })
        .await;

    let base_url: Url = mock_server.base_url().parse()?;
    let url = base_url.join("content/current/CUI/C0010200/relations")?;
    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let client = umls_http::reqwest::Client::default();

    Relation::new(url)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    mock.assert_async().await;

    Ok(())
}
