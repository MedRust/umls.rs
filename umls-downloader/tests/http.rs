#![cfg(feature = "reqwest")]

use httpmock::prelude::*;
use serde_json::{Value, json};
use umls_downloader::{ReleaseFetcher, kind, release_type::ReleaseType};
use umls_http::reqwest::Client;

fn client() -> Client {
    Client::default()
}

fn release(name: &str) -> Value {
    json!({"fileName":name,"releaseVersion":"2026AA","releaseDate":"2026-01-01",
        "downloadUrl":"https://example.test/archive.zip","releaseType":"umls","product":"UMLS"})
}

#[tokio::test]
async fn release_selection_preserves_order_empty_results_and_explicit_queries() {
    let server = MockServer::start_async().await;
    for records in [vec![], vec![release("first.zip"), release("second.zip")]] {
        let mock = server
            .mock_async(|when, then| {
                when.path("/releases")
                    .query_param("custom", "kept")
                    .query_param_missing("current");
                then.json_body(json!(records));
            })
            .await;
        let uri: http::Uri = server.url("/releases?custom=kept").parse().unwrap();
        let current = ReleaseFetcher::new(client(), uri.clone(), kind::Current)
            .send()
            .await
            .unwrap();
        assert_eq!(
            current.as_ref().map(|r| r.file_name.as_str()),
            records.first().map(|_| "first.zip")
        );
        let all = ReleaseFetcher::new(client(), uri.clone(), kind::All)
            .send()
            .await
            .unwrap();
        assert_eq!(
            all.iter().map(|r| r.file_name.clone()).collect::<Vec<_>>(),
            records
                .iter()
                .map(|r| r["fileName"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        );
        let category = ReleaseType {
            product: "UMLS".into(),
            name: "ignored".into(),
            endpoint: uri.to_string(),
        };
        let category_current = ReleaseFetcher::new(client(), category.clone(), kind::Current)
            .send()
            .await
            .unwrap();
        assert_eq!(
            category_current.map(|r| r.file_name),
            current.map(|r| r.file_name)
        );
        assert_eq!(
            ReleaseFetcher::new(client(), category, kind::All)
                .send()
                .await
                .unwrap()
                .len(),
            all.len()
        );
        mock.assert_calls_async(4).await;
        mock.delete_async().await;
    }
}
