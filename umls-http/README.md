# UMLS HTTP

HTTP transport used by `umls-downloader` and `umls-downloader-cli`.
Without features, the crate exposes only `WithoutClient`, a marker for builders
that have no transport attached. Enable `reqwest` to use the HTTP client.

## Setup

```toml
[dependencies]
umls-http = { version = "0.1", features = ["reqwest"] }
http = "1"
serde_json = "1"
futures-util = "0.3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## JSON requests

```rust,no_run
use umls_http::reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default();
    let request = http::Request::get("https://uts-ws.nlm.nih.gov/releases").body(())?;
    let response = client.execute::<(), serde_json::Value>(request).await?;
    println!("Status: {}", response.status());
    println!("{}", response.into_body());
    Ok(())
}
```

`execute()` decodes the complete JSON response. `execute_stream()` returns an
`http::Response` containing a byte stream instead. Both preserve status and
headers and reject HTTP 4xx and 5xx responses. Request bodies are serialized as
JSON; `()` serializes to `null` and is omitted. Set `Content-Type` explicitly
when sending JSON that requires it.

## Streaming responses

```rust,no_run
use futures_util::StreamExt;
use umls_http::reqwest::Client;

async fn read_chunks(client: &Client, uri: http::Uri) -> Result<(), Box<dyn std::error::Error>> {
    let request = http::Request::get(uri).body(())?;
    let response = client.execute_stream(request).await?;
    let mut stream = std::pin::pin!(response.into_body());
    while let Some(chunk) = stream.next().await {
        println!("Received {} bytes", chunk?.len());
    }
    Ok(())
}
```

The stream owns the response and reads chunks as it is polled. Body read errors
are yielded by the stream. For authenticated UTS release downloads, use
`umls_downloader::ReleaseDownloader`, which constructs the Download API request.

## Configuration and errors

`Client::default()` uses reqwest defaults. To customize timeouts or other
settings, add `reqwest = "0.13"` and pass a configured `reqwest::Client` to
`umls_http::reqwest::Client::new(inner)`. Clones share the connection pool.

`umls_http::reqwest::Error` distinguishes JSON serialization (`Json`), transport,
HTTP status, response decoding and stream failures (`Reqwest`), and HTTP response
construction (`Http`). Each variant retains its underlying error as its source.

## API documentation

```sh
cargo doc -p umls-http --features reqwest --no-deps --open
cargo test -p umls-http --all-features --doc
```
