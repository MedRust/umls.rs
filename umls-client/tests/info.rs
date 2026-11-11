//! An integration test for Infomation APi's.
#![cfg(feature = "reqwest")]

use httpmock::{Method::GET, MockServer};
use serde_json::json;
use umls_client::{
    config,
    core::{aui::Aui, cui::Cui},
    dto::{concept::Payload, display_name::DisplayName},
    info::Info,
};
use url::Url;

#[tokio::test]
async fn successfully_get_atom_info() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;

    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/AUI/A8345234")
                .query_param_exists("apiKey");
            then.status(200)
            .json_body(json!({
                "pageSize": 25,
                "pageNumber": 1,
                "pageCount": 1,
                "result": {
                    "classType": "Atom",
                    "ui": "A8345234",
                    "sourceDescriptor": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/ICD9CM/386.11",
                    "sourceConcept": "NONE",
                    "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                    "suppressible": "false",
                    "obsolete": "false",
                    "rootSource": "ICD9CM",
                    "termType": "PT",
                    "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/ICD9CM/386.11",
                    "language": "ENG",
                    "name": "Benign paroxysmal positional vertigo",
                    "ancestors": "NONE",
                    "descendants": "NONE",
                    "attributes": "NONE",
                    "relations": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A8345234/relations",
                    "children": "NONE",
                    "parents": "NONE"
                }
            }));
        })
        .await;

    let base_url = mock_server.base_url().parse()?;

    let config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .version(config::Version::Current)
        .build()?;

    let client = umls_http::reqwest::Client::default();

    let aui_string: DisplayName = "A8345234".parse()?;
    let aui = Aui::new(aui_string)?;

    let response = Info::new(aui).with_client(client).fetch(&config).await?;

    assert_eq!(response.ui.as_str(), "A8345234");
    assert_eq!(
        response.name.as_str(),
        "Benign paroxysmal positional vertigo"
    );
    assert_eq!(response.language.as_str(), "ENG");
    assert_eq!(response.term_type.as_str(), "PT");
    assert_eq!(response.root_source.as_str(), "ICD9CM");
    assert!(!response.suppressible);
    assert!(!response.obsolete);

    mock.assert_async().await;
    Ok(())
}

fn assert_concept(concept: &Payload) {
    assert_eq!(concept.ui, "C0010200");
    assert_eq!(concept.name, "Coughing");
    assert_eq!(concept.date_added, "09-30-1990");
    assert_eq!(concept.major_revision_date, "09-03-2024");
    assert!(!concept.suppressible);
    assert_eq!(concept.status, "R");
    assert!(!concept.semantic_types.is_empty());
    assert_eq!(concept.semantic_types.len(), 1);
    let semantic_type = concept.semantic_types.first().map(|x| x.name.as_str());
    assert_eq!(semantic_type, Some("Sign or Symptom"));
    assert_eq!(concept.atom_count, 242);
    assert_eq!(concept.cv_member_count, 0);
    assert_eq!(concept.attribute_count, 0);
    assert_eq!(concept.relation_count, 29);
    assert_eq!(
        concept.atoms.value().as_str(),
        "https://uts-ws.nlm.nih.gov/rest/content/2026AB/CUI/C0010200/atoms"
    );
    assert_eq!(
        concept.definitions.as_ref().map(|x| x.value().as_str()),
        Some("https://uts-ws.nlm.nih.gov/rest/content/2026AB/CUI/C0010200/definitions")
    );
    assert_eq!(
        concept.relations.as_ref().map(|x| x.value().as_str()),
        Some("https://uts-ws.nlm.nih.gov/rest/content/2026AB/CUI/C0010200/relations")
    );
    assert_eq!(
        concept.default_preferred_atom.value().as_str(),
        "https://uts-ws.nlm.nih.gov/rest/content/2026AB/CUI/C0010200/atoms/preferred"
    );
}

async fn mock_concept_info<'a>(server: &'a MockServer, api_key: &'a str) -> httpmock::Mock<'a> {
    server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/CUI/C0010200")
                .query_param("apiKey", api_key);

            let str = include_str!("fixtures/concept_info.json");
            let json: serde_json::Value = serde_json::from_str(str).unwrap();

            then.status(200)
                .header("CONTENT-TYPE", "application/json")
                .json_body(json);
        })
        .await
}

#[tokio::test]
async fn fetch_concept_info_using_url() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();
    let api_key = "some_api_key";

    let mock_server = MockServer::start_async().await;
    let mock = mock_concept_info(&mock_server, api_key).await;
    let base_url: Url = mock_server.base_url().parse()?;

    let url = base_url.join("/content/current/CUI/C0010200")?;

    let config = config::Config::builder(api_key)
        .base_url(base_url)
        .build()?;

    let info = Info::new(url)
        .with_client(Default::default())
        .fetch(&config)
        .await?;

    assert_concept(&info);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn fetch_concept_info_using_cui() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();
    let api_key = "some_api_key";

    let mock_server = MockServer::start_async().await;
    let mock = mock_concept_info(&mock_server, api_key).await;

    let base_url: Url = mock_server.base_url().parse()?;
    let config = config::Config::builder(api_key)
        .base_url(base_url)
        .build()?;

    let cui: Cui = "C0010200".parse()?;
    let info = Info::new(cui)
        .with_client(Default::default())
        .fetch(&config)
        .await?;
    assert_concept(&info);

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn rejects_invalid_url() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let config = config::Config::builder("some_api_key").build()?;

    let client = umls_http::reqwest::Client::default();
    let url = "https://uts-ws.nlm.nih.gov/rest/content/current/CUI/".parse::<url::Url>()?;

    let result = Info::new(url).with_client(client).fetch(&config).await;
    assert!(matches!(
        result,
        Err(umls_client::info::Error::Validator(_))
    ));
    Ok(())
}
