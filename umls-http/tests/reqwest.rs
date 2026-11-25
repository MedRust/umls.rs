//! Test requwest http api's.
#![cfg(feature = "reqwest")]

use httpmock::MockServer;
use serde_json::json;
use umls_http::reqwest::{ClientError, ReqwestClient};

#[tokio::test]
async fn get_success_on_200_ok() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method("GET")
                .path("/")
                .header("ACCEPT", "application/json");

            let body = json!({});

            then.status(200)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let base_url = mock_server.base_url();

    let _ = ReqwestClient::default()
        .with_type::<umls_http::reqwest::Json>()
        .get::<_, serde_json::Value>(base_url)
        .await?;

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn validation_error_on_400_bad_request() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let body = json!({
        "name": "ValidationError",
        "status": 400,
        "message": "Missing 'string'"
    });

    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method("GET")
                .path("/")
                .header("ACCEPT", "application/json");
            then.status(400)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let result = ReqwestClient::default()
        .with_type::<umls_http::reqwest::Json>()
        .get::<_, serde_json::Value>(mock_server.base_url())
        .await;

    match result {
        Err(ClientError::Validation(message)) => {
            assert_eq!(message, "Missing 'string'");
        }
        other => panic!("Expected validation error, but got: {:?}", other),
    }

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn unauthorized_error_on_401() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let body = json!({
        "name": "UnauthorizedError",
        "status": 401,
        "message": "Missing Service Ticket or API Key. Service Ticket or API Key must be provided - Documentation: https://documentation.uts.nlm.nih.gov/rest/authentication.html"
    });

    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method("GET")
                .path("/")
                .header("ACCEPT", "application/json");
            then.status(401)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let result = ReqwestClient::default()
        .with_type::<umls_http::reqwest::Json>()
        .get::<_, serde_json::Value>(mock_server.base_url())
        .await;

    match result {
        Err(ClientError::Unauthorized(message)) => {
            assert_eq!(
                message,
                "Missing Service Ticket or API Key. Service Ticket or API Key must be provided - Documentation: https://documentation.uts.nlm.nih.gov/rest/authentication.html"
            );
        }
        other => panic!("Expected unauthorized error but, got: {:?}", other),
    }

    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn not_found_error_on_404() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let body = json!({
        "name": "NotFoundError",
        "status": 404,
        "message": "404 Resource Not Found"
    });

    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.path("/")
                .method("GET")
                .header("ACCEPT", "application/json");
            then.status(404)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let result = ReqwestClient::default()
        .with_type::<umls_http::reqwest::Json>()
        .get::<_, serde_json::Value>(mock_server.base_url())
        .await;

    match result {
        Err(ClientError::NotFound(message)) => {
            assert_eq!(message, "404 Resource Not Found");
        }
        other => panic!("Expected not found error but, got: {:?}", other),
    }

    mock.assert_async().await;
    Ok(())
}
