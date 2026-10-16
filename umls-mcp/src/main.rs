//! Stdio and HTTP entry point for the UMLS MCP server.
mod http;
mod server;

use clap::{Parser, ValueEnum};
use rmcp::{ServiceExt, transport::stdio};
use std::{error::Error, net::SocketAddr, time::Duration};
use umls_client::config::Config;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Transport {
    Stdio,
    Http,
}

#[derive(Debug, Parser)]
#[command(version, about = "UMLS MCP server over stdio or Streamable HTTP")]
struct Args {
    /// MCP transport to serve.
    #[arg(long, env = "UMLS_MCP_TRANSPORT", value_enum, default_value = "stdio")]
    transport: Transport,
    /// HTTP listening address (used with --transport http).
    #[arg(long, env = "UMLS_MCP_BIND", default_value = "127.0.0.1:3000")]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let key =
        std::env::var("UMLS_API_KEY").map_err(|_| "Set UMLS_API_KEY before starting umls-mcp")?;
    if key.trim().is_empty() {
        return Err("UMLS_API_KEY must not be empty".into());
    }
    let mut builder = Config::builder(key);
    if let Ok(version) = std::env::var("UMLS_VERSION") {
        builder = builder.try_version(version.as_str())?;
    }
    if let Ok(base_url) = std::env::var("UMLS_BASE_URL") {
        builder = builder.base_url(base_url.parse()?);
    }
    let config = builder.build()?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let server = server::UmlsServer::new(config, umls_http::reqwest::Client::new(client));
    match args.transport {
        Transport::Stdio => {
            let service = server.serve(stdio()).await?;
            service.waiting().await?;
        }
        Transport::Http => {
            let listener = tokio::net::TcpListener::bind(args.bind).await?;
            let address = listener.local_addr()?;
            let cancellation = tokio_util::sync::CancellationToken::new();
            let router = http::router(server, address, cancellation.clone());
            eprintln!("UMLS MCP server listening at http://{address}/mcp");
            axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = tokio::signal::ctrl_c().await;
                    cancellation.cancel();
                })
                .await?;
        }
    }
    Ok(())
}
