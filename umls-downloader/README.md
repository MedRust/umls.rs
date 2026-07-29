# UMLS Downloader

`umls-downloader` provides asynchronous, HTTP-client-agnostic helpers for
discovering and downloading releases from the
[UMLS Terminology Services (UTS)](https://documentation.uts.nlm.nih.gov/automating-downloads.html)
Release and Download APIs.

The crate supports release files for UMLS, RxNorm, and SNOMED CT.

## Features

- Discover the release types advertised by UTS.
- Fetch the current release or all releases for a release type.
- Stream a release to any caller-selected destination.
- Download directly to a local file when desired.
- Integrate with any HTTP stack by implementing small client traits.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
umls-downloader = "0.1"
```

## HTTP client integration

The crate does not select an HTTP transport or async runtime. Applications
provide adapters for:

- `HttpClient`, which returns deserialized release metadata.
- `StreamHttpClient`, which returns release data as a `ByteStream`.

This keeps the library compatible with different HTTP clients, runtimes, and
storage services.

## Discovering releases

Start by retrieving the release types supported by UTS:

```rust,ignore
use umls_downloader::get_release_types;

let release_types = get_release_types(&http_client).await?;

for release_type in release_types {
    println!("{release_type}");
}
```

Use `ReleaseFetcher` with `kind::Current` to request the current release:

```rust,ignore
use umls_downloader::{ReleaseFetcher, kind::Current};

let fetcher = ReleaseFetcher::new("umls-full-release", Current);
let release = fetcher.send(&http_client).await?;

if let Some(release) = release {
    println!("{}: {}", release.file_name, release.download_url);
}
```

Use `kind::All` instead to request current and past releases:

```rust,ignore
use umls_downloader::{ReleaseFetcher, kind::All};

let fetcher = ReleaseFetcher::new("rxnorm-weekly-updates", All);
let releases = fetcher.send(&http_client).await?;
```

## Streaming a download

`download_release` authenticates with the UTS Download API and returns a
`ByteStream`. The caller owns the storage decision and can forward the stream
to cloud object storage, another remote service, an HTTP response, memory, or a
local file.

```rust,ignore
use umls_downloader::download_release;

let source_uri = release.download_url.parse()?;
let stream = download_release(&stream_http_client, source_uri, &api_key).await?;

// Pass `stream` to the destination selected by your application.
cloud_storage.upload("umls-release.zip", stream).await?;
```

The UTS API key can be obtained from your
[UTS profile](https://uts.nlm.nih.gov/uts/profile).

## Downloading to a local file

For local storage, use the convenience wrapper:

```rust,ignore
use umls_downloader::download_release_to_path;

let source_uri = release.download_url.parse()?;

download_release_to_path(
    &stream_http_client,
    source_uri,
    &api_key,
    "umls-release.zip",
)
.await?;
```

The destination file is created or replaced. If the download fails after the
file is created, a partial file may remain.

## Errors

Download operations return `umls_downloader::Error`, which distinguishes:

- HTTP request errors
- URL and URI construction errors
- response-stream errors
- local file I/O errors

Individual errors from a returned `ByteStream` are reported as `BoxError`
values while the caller consumes the stream.

## API documentation

Generate and open the Rust API documentation locally:

```sh
cargo doc --open
```
