//! Integration tests for the reqwest HTTP transport, enabled by the `reqwest` feature.
//!
//! Local HTTP servers exercise JSON request and response handling, shared client
//! configuration and connection reuse, typed error sources, and incremental streaming
//! with truncated-body errors. These tests do not contact external services.

#![cfg(feature = "reqwest")]

use std::{error::Error, time::Duration};

use futures_util::StreamExt;
use httpmock::{Method::POST, MockServer};
use serde_json::{Value, json};
use umls_http::reqwest::Client;

fn client() -> Client {
    Client::new(
        reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap(),
    )
}

#[tokio::test]
async fn json_request_and_response_preserve_body_method_headers_and_status() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/json")
                .header("content-type", "application/json")
                .header("x-request", "test")
                .json_body(json!({"name":"UMLS"}));
            then.status(201)
                .header("x-response", "preserved")
                .json_body(json!({"ok":true}));
        })
        .await;
    let request = http::Request::post(server.url("/json"))
        .header("content-type", "application/json")
        .header("x-request", "test")
        .body(json!({"name":"UMLS"}))
        .unwrap();
    let response = client().execute::<_, Value>(request).await.unwrap();
    assert_eq!(response.status(), 201);
    assert_eq!(response.headers()["x-response"], "preserved");
    assert_eq!(response.into_body(), json!({"ok":true}));
    mock.assert_async().await;
}

#[tokio::test]
async fn clones_reuse_configuration_and_unit_requests_have_no_body() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/empty")
                .body("")
                .header("x-shared", "configured");
            then.json_body(json!([]));
        })
        .await;
    let mut headers = http::HeaderMap::new();
    headers.insert("x-shared", "configured".parse().unwrap());
    let client = Client::new(
        reqwest::Client::builder()
            .no_proxy()
            .default_headers(headers)
            .build()
            .unwrap(),
    );
    for client in [client.clone(), client] {
        client
            .execute::<_, Value>(http::Request::post(server.url("/empty")).body(()).unwrap())
            .await
            .unwrap();
    }
    mock.assert_calls_async(2).await;
}

#[tokio::test]
async fn http_status_and_decoding_errors_preserve_sources() {
    let server = MockServer::start_async().await;
    for status in [400, 404, 500, 503] {
        let path = format!("/{status}");
        let mock = server
            .mock_async(|when, then| {
                when.path(&path);
                then.status(status);
            })
            .await;
        let error = client()
            .execute::<_, Value>(http::Request::get(server.url(&path)).body(()).unwrap())
            .await
            .unwrap_err();
        assert!(error.source().is_some());
        assert!(
            matches!(error, umls_http::reqwest::Error::Reqwest(ref e) if e.status().unwrap().as_u16() == status)
        );
        mock.assert_async().await;
    }
    let mock = server
        .mock_async(|when, then| {
            when.path("/invalid");
            then.body("not json");
        })
        .await;
    let error = client()
        .execute::<_, Value>(http::Request::get(server.url("/invalid")).body(()).unwrap())
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    assert!(matches!(error, umls_http::reqwest::Error::Reqwest(ref e) if e.is_decode()));
    mock.assert_async().await;
}

#[tokio::test]
async fn connection_failure_preserves_transport_error() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let error = client()
        .execute::<_, Value>(
            http::Request::get(format!("http://{address}/"))
                .body(())
                .unwrap(),
        )
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    assert!(matches!(error, umls_http::reqwest::Error::Reqwest(ref e) if e.is_connect()));
}

// The server cannot finish until the test explicitly releases it. This proves
// incremental delivery without relying on a sleep or network chunk boundaries.
#[tokio::test]
async fn stream_delivers_before_completion_and_reports_truncated_body() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    for truncate in [false, true] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (release, wait) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nX-Stream: yes\r\nConnection: close\r\n\r\nabc").await.unwrap();
            wait.await.unwrap();
            if !truncate {
                socket.write_all(b"def").await.unwrap();
            }
        });
        let response = client()
            .execute_stream(
                http::Request::get(format!("http://{address}/"))
                    .body(())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["x-stream"], "yes");
        let mut stream = std::pin::pin!(response.into_body());
        let first = tokio::time::timeout(Duration::from_secs(3), stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(&first[..], b"abc");
        release.send(()).unwrap();
        let mut remaining = Vec::new();
        let mut failed = false;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => remaining.extend_from_slice(&chunk),
                Err(error) => {
                    assert!(error.source().is_some());
                    assert!(matches!(error, umls_http::reqwest::Error::Reqwest(_)));
                    failed = true;
                }
            }
        }
        assert_eq!(failed, truncate);
        if !truncate {
            assert_eq!(remaining, b"def");
        }
        server.await.unwrap();
    }
}

#[tokio::test]
async fn cloned_clients_reuse_the_same_tcp_connection() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        // Accept only once: a second connection cannot receive a response.
        let (mut socket, _) = listener.accept().await.unwrap();
        for _ in 0..2 {
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            let request = String::from_utf8(request).unwrap().to_ascii_lowercase();
            assert!(request.starts_with("get /reuse http/1.1\r\n"));
            assert!(!request.contains("transfer-encoding:"));
            assert!(
                !request.contains("content-length:") || request.contains("content-length: 0\r\n")
            );
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: application/json\r\n\r\n[]").await.unwrap();
        }
    });
    let client = client();
    for client in [client.clone(), client] {
        let response = client
            .execute::<_, Value>(
                http::Request::get(format!("http://{address}/reuse"))
                    .body(())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.into_body(), json!([]));
    }
    server.await.unwrap();
}
