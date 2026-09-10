# UMLS Downloader

`umls-downloader` provides asynchronous helpers for discovering and downloading
UMLS, RxNorm, and SNOMED CT releases through the UTS Release and Download APIs.

## Installation

Enable the optional `reqwest` feature to execute discovery and download requests:

```toml
[dependencies]
umls-downloader = { version = "0.1", features = ["reqwest"] }
umls-http = { version = "0.1", features = ["reqwest"] }
http = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3" # For consuming download streams
```

The transport is provided by the separate `umls-http` crate; add it as a direct
dependency to import its types. The downloader’s `reqwest` feature forwards to
`umls-http/reqwest`.

Run the async examples below inside a Tokio runtime. Without `reqwest`, the crate
exposes metadata and builder types, but no HTTP execution methods.

## HTTP client configuration

`umls_http::reqwest::Client` handles both JSON metadata and streamed
responses. Create it with `Client::default()`, or wrap a configured
`reqwest::Client` with `Client::new(inner)`. Clones share a connection pool.
The transport also exposes `execute()` for JSON requests and `execute_stream()`
for byte streams, each returning an `http::Response` with status and headers.

`umls_http::WithoutClient` marks builders before a client is attached.
`release_type::Client` discovers categories; it is distinct from the HTTP client.

## Discovering releases

```rust,no_run
use umls_downloader::{ReleaseFetcher, kind, release_type};
use umls_http::reqwest::Client;

async fn discover() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default();
    let categories = release_type::Client::default()
        .with_client(client.clone())
        .fetch()
        .await?;

    for category in categories {
        println!("{}: {}", category.product(), category.name());
    }

    let current = ReleaseFetcher::new(client.clone(), "umls-full-release", kind::Current)
        .send()
        .await?;
    if let Some(release) = current {
        println!("{}: {}", release.file_name, release.download_url);
    }

    let releases = ReleaseFetcher::new(client, "rxnorm-weekly-updates", kind::All)
        .send()
        .await?;
    println!("{} releases", releases.len());
    Ok(())
}
```

`ReleaseFetcher::new(client, category, kind)` owns its arguments, and `.send()`
consumes the fetcher. Categories may be `&str`, `String`, `http::Uri`, or
`release_type::ReleaseType` values. `kind::Current` returns `Option<Release>`;
`kind::All` returns `Vec<Release>`.

For category names, `Current` adds `current=true` to the request. Explicit URIs
and `ReleaseType::endpoint` values are used as supplied: `Current` selects the
first returned release without adding a filter, while `All` returns the full
response. Include any desired filtering in those endpoints.

## Streaming a download

Pass the release's `download_url` as an `http::Uri` to `ReleaseDownloader`.
The returned stream yields `bytes::Bytes` chunks and typed
`umls_http::reqwest::Error` values. Pin it before consuming it.

```rust,no_run
use futures_util::StreamExt;
use umls_downloader::{Release, ReleaseDownloader};
use umls_http::reqwest::Client;

async fn stream_release(release: &Release) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("UMLS_API_KEY")?;
    let source = release.download_url.parse::<http::Uri>()?;
    let downloader = ReleaseDownloader::new(api_key, source)
        .with_client(Client::default());
    let stream = downloader.download().await?;
    let mut stream = std::pin::pin!(stream);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        // Forward each chunk to the destination selected by your application.
        println!("Received {} bytes", chunk.len());
    }
    Ok(())
}
```

Add `http = "1"` when naming `http::Uri` directly, as in this example.
The stream owns the response body; the caller chooses its storage destination.
Obtain a UTS API key from your [UTS profile](https://uts.nlm.nih.gov/uts/profile).

## Downloading to a local file

Use `DownloadResource` to associate the source URI with a destination path:

```rust,no_run
use umls_downloader::{DownloadResource, Release, ReleaseDownloader};
use umls_http::reqwest::Client;

async fn save_release(release: &Release) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("UMLS_API_KEY")?;
    let resource = DownloadResource::new(release.download_url.parse()?, "umls-release.zip");
    ReleaseDownloader::new(api_key, resource)
        .with_client(Client::default())
        .download()
        .await?;
    Ok(())
}
```

This operation consumes the downloader and creates or replaces the destination
file. A failure or cancellation after file creation can leave a partial file.
File creation, writes, and flushing use synchronous filesystem I/O.

## Errors

Discovery and download operations return `Error<umls_http::reqwest::Error>`.
The outer error distinguishes client failures (`Client`), URL parsing (`Url`),
URI construction (`Uri`), and local file I/O (`Io`). The transport error
distinguishes JSON serialization (`Json`), reqwest failures (`Reqwest`), and
HTTP response construction (`Http`). Errors while consuming a returned
stream use `umls_http::reqwest::Error` directly.

## API documentation

Generate the API documentation, including HTTP execution methods:

```sh
cargo doc -p umls-downloader --features reqwest --no-deps --open
```
