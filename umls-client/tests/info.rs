//! An integration test for Infomation APi's.
#![cfg(feature = "reqwest")]

use httpmock::{Method::GET, MockServer};
use serde_json::json;
use umls_client::{concept::Info, config, core::aui::Aui, search::core::DisplayName};

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
