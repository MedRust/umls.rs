//! A list of integration test for raw search function apis.

#![cfg(feature = "reqwest")]

use httpmock::{Method::GET, MockServer};
use serde_json::json;

use umls_client::{config::Config, dto::batch, search::Search, types};
use umls_http::WithoutClient;
use uuid::Uuid;

#[tokio::test]
async fn successful_search_in_umls() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let mock_server = MockServer::start_async().await;

    let api_key = Uuid::new_v4();

    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/search/current")
                .query_param("string", "cough");

            then.status(200).header("CONTENT-TYPE", "application/json").json_body(json!({
                "pageSize": 1,
                "pageNumber": 1,
                "result": {
                    "classType": "searchResults",
                    "results": [
                        {
                            "ui": "C0010200",
                            "rootSource": "MTH",
                            "uri": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0010200",
                            "name": "Coughing",
                            "semanticTypes": [
                                "Sign or Symptom"
                            ]
                        },
                    ],
                    "recCount": 1523
                }
            }));
        })
        .await;

    let base_url = mock_server.base_url().parse()?;

    let config = Config::builder(api_key.to_string())
        .base_url(base_url)
        .build()?;

    let term = "cough".try_into()?;
    let concept = umls_client::dto::search::concept::Config::<
        umls_client::dto::search::partial_search::NoPartial,
    >::builder()
    .input(umls_client::dto::search::Input::Atom(term))
    .build()?;

    let batch = batch::Batch::new(25.try_into()?, 1.try_into()?, concept);

    let response = Search::<WithoutClient, types::Concept, _>::new(batch)
        .with_client(umls_http::reqwest::Client::default())
        .fetch(&config)
        .await?
        .into_page();

    assert_eq!(response.size().get(), 1);
    assert_eq!(response.number().get(), 1);
    assert_eq!(response.count().map(|x| x.get()), Some(1523));

    let results = response.content().list();

    assert_eq!(results.len(), 1);

    let search_object = results
        .first()
        .expect("The results vector should contain one element");

    assert_eq!(search_object.ui().as_str(), "C0010200");

    assert_eq!(search_object.root_source().as_str(), "MTH");
    assert_eq!(
        search_object.uri().value().as_str(),
        "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0010200"
    );
    assert_eq!(search_object.name().as_str(), "Coughing");

    let semantic_types = search_object.semantic_types();
    assert!(!semantic_types.is_empty());

    let semantic_type = semantic_types
        .first()
        .expect("The non empty list should contain at least one semantic type.");

    assert_eq!(semantic_type.as_str(), "Sign or Symptom");

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn search_umls_concepts() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let mock_server = MockServer::start_async().await;

    let api_key = Uuid::new_v4();

    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/search/current")
                .header("ACCEPT", "application/json")
                .query_param("string", "Cough")
                .query_param("inputType", "atom")
                .query_param("includeObsolete", "false")
                .query_param("includeSuppressible", "false")
                .query_param("returnIdType", "concept")
                .query_param("sabs", "")
                .query_param("searchType", "words")
                .query_param("semanticTypes", "")
                .query_param("semanticGroups", "")
                .query_param("partialSearch", "false")
                .query_param("pageSize", "25")
                .query_param("pageNumber", "1")
                .query_param("apiKey", api_key.to_string());

            then.status(200).header("CONTENT-TYPE", "application/json").json_body(json!({
                "pageSize": 25,
                "pageNumber": 1,
                "result": {
                    "classType": "searchResults",
                    "results": [
                        {
                            "ui": "C0010200",
                            "rootSource": "MTH",
                            "uri": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0010200",
                            "name": "Coughing",
                            "semanticTypes": [
                                "Sign or Symptom"
                            ]
                        },
                    ],
                    "recCount": 1523
                }
            }));
        })
        .await;

    let base_url = mock_server.base_url().parse()?;

    let config = Config::builder(api_key.to_string())
        .base_url(base_url)
        .build()?;

    let concept = umls_client::dto::search::concept::Config::<
        umls_client::dto::search::partial_search::NoPartial,
    >::builder()
    .input(umls_client::dto::search::Input::Atom("Cough".parse()?))
    .build()?;

    let batch = batch::Batch::new(25.try_into()?, 1.try_into()?, concept);

    let response = Search::<WithoutClient, types::Concept, _>::new(batch)
        .with_client(umls_http::reqwest::Client::default())
        .fetch(&config)
        .await?
        .into_page();

    assert_eq!(response.size().get(), 25);
    assert_eq!(response.number().get(), 1);
    assert_eq!(response.count().map(|x| x.get()), Some(61));
    assert!(!response.content().list().is_empty());
    let object = response
        .content()
        .list()
        .first()
        .expect("The non empty list should contain at least one object");

    assert_eq!(object.ui().as_str(), "C0010200");
    assert_eq!(object.root_source().as_str(), "MTH");
    assert_eq!(
        object.uri().value().as_str(),
        "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0010200"
    );
    assert_eq!(object.name().as_str(), "Coughing");

    let semantic_types = object.semantic_types();
    assert!(!semantic_types.is_empty());
    assert_eq!(semantic_types.len(), 1);

    let semantic_type = semantic_types
        .first()
        .expect("The non empty list should contain at least one semantic type");
    assert_eq!(semantic_type.as_str(), "Sign or Symptom");

    mock.assert_async().await;

    Ok(())
}
