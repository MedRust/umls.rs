//! Streamable HTTP routing and transport configuration.
use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

use crate::server::UmlsServer;

pub(crate) fn router(
    server: UmlsServer,
    address: SocketAddr,
    cancellation: CancellationToken,
) -> Router {
    let config = StreamableHttpServerConfig::default()
        .with_cancellation_token(cancellation)
        .with_allowed_hosts([
            "localhost".to_owned(),
            "127.0.0.1".to_owned(),
            "::1".to_owned(),
            address.ip().to_string(),
        ])
        .with_allowed_origins([
            format!("http://{address}"),
            format!("http://localhost:{}", address.port()),
        ])
        .enforce_origin_validation();
    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    Router::new().nest_service("/mcp", service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::{
        ServiceExt, model::CallToolRequestParams, transport::StreamableHttpClientTransport,
    };
    use serde_json::json;
    use umls_client::config::Config;

    #[tokio::test]
    async fn http_client_initializes_discovers_tools_and_calls_them() {
        tokio::time::timeout(std::time::Duration::from_secs(15), async {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let cancellation = CancellationToken::new();
            let server = UmlsServer::new(
                Config::builder("http-test-key").build().unwrap(),
                Default::default(),
            );
            let router = router(server, address, cancellation.clone());
            let shutdown = cancellation.clone();
            let task = tokio::spawn(async move {
                axum::serve(listener, router)
                    .with_graceful_shutdown(shutdown.cancelled_owned())
                    .await
                    .unwrap();
            });
            let url = format!("http://{address}/mcp");
            let raw = reqwest::Client::new();
            let blocked = raw
                .post(&url)
                .header("Origin", "https://untrusted.example")
                .header("Accept", "application/json, text/event-stream")
                .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}))
                .send()
                .await
                .unwrap();
            assert_eq!(blocked.status(), reqwest::StatusCode::FORBIDDEN);
            let transport = StreamableHttpClientTransport::from_uri(url);
            let client = ().serve(transport).await.unwrap();
            let tools = client.list_all_tools().await.unwrap();
            assert_eq!(tools.len(), 6);
            let response = client
                .call_tool(
                    CallToolRequestParams::new("get_concept")
                        .with_arguments(json!({"cui": "invalid"}).as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            assert_eq!(response.is_error, Some(true));
            client.cancel().await.unwrap();
            cancellation.cancel();
            task.await.unwrap();
        })
        .await
        .expect("HTTP integration test timed out");
    }
}
