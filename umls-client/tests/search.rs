//! A list of integration test for raw search function apis.

use httpmock::{Method::GET, MockServer};
use serde_json::json;

use umls_client::{
    config::Config,
    search::{Request, raw::search},
};
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

            then.status(200).json_body(json!({
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

    let request = Request::builder("cough".parse()?).build();

    let response = search(&config, &request).await?;

    assert_eq!(response.page_size.get(), 1);
    assert_eq!(response.page_number.get(), 1);
    assert_eq!(response.result.class_type, "searchResults".parse()?);
    assert_eq!(response.result.results.len(), 1);

    assert!(response.result.rec_count.is_some());
    assert_eq!(response.result.rec_count, Some(1523));

    let search_object = response
        .result
        .results
        .first()
        .expect("The results vector should contain one element");

    assert_eq!(search_object.ui.as_str(), "C0010200");

    assert!(search_object.root_source.is_some());
    assert_eq!(search_object.root_source, Some("MTH".parse()?));
    assert_eq!(
        search_object.uri.as_ref(),
        "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0010200"
    );
    assert_eq!(search_object.name.as_str(), "Coughing");

    assert!(search_object.semantic_types.is_some());
    assert_eq!(search_object.semantic_types.as_ref().unwrap().len(), 1);

    let semantic_type = search_object
        .semantic_types
        .as_ref()
        .unwrap()
        .first()
        .expect("Semantic types list should contain one name");

    assert_eq!(semantic_type.as_str(), "Sign or Symptom");

    mock.assert_async().await;

    Ok(())
}
