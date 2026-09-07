# UMLS.rs

Rust tools for querying concepts and atoms, and discovering and downloading
releases from the U.S. National Library of Medicine's UMLS Terminology
Services (UTS).

The workspace supports release files for:

- UMLS
- RxNorm
- SNOMED CT

## Workspace crates

### `umls-client`

An asynchronous client for UMLS concept search, concept retrieval, and
concept atom retrieval, including filtering and pagination.

Create a client with `umls_client::config::Config::builder(api_key)`. The
release defaults to `Version::Current`; pass a typed `Version::Release` to
`.version()` before `.build()` to select a specific release. CUI-based all-atoms and
preferred-atom requests honor this setting. Requests built from an explicit
atom-list URL retain the release in that URL.

```rust
use umls_client::config::{Config, ReleaseType, Version};

let config = Config::builder(std::env::var("UMLS_API_KEY")?)
    .version(Version::Release { year: 2018, kind: ReleaseType::AA })
    .build()?;
```

`Config::version()` now returns `&Version`. Use `.as_str()`
when a formatted identifier is needed. The builder rejects years before 2018. It does not yet enforce the current
year as an upper bound or check whether a release has been published. Parsing
explicit releases from strings is currently incomplete; use typed construction
for pinned releases. The CLI examples default to `current`; selecting an explicit
release through `UMLS_VERSION` or `--version` currently fails parsing.

The default REST base URL is `https://uts-ws.nlm.nih.gov/rest/`. When overriding
it, supply a URL whose path ends with a slash:

```rust
let config = umls_client::config::Config::builder(std::env::var("UMLS_API_KEY")?)
    .base_url("https://uts-ws.nlm.nih.gov/rest/".parse()?)
    .build()?;
```

`Config::Builder::build()` rejects base URL paths without a trailing slash,
which is required to preserve the full path prefix when joining atom endpoints.

The `umls_client::atom` module documentation includes examples for filtered
atom retrieval, and `Preferred::for_concept` documents preferred-atom retrieval.
Generate the client documentation with `cargo doc -p umls-client --no-deps`.

### `umls-downloader`

An asynchronous, HTTP-client-agnostic library for:

- Discovering UTS release types
- Fetching current or historical release versions
- Streaming release archives
- Downloading archives to local files

See [`umls-downloader/README.md`](umls-downloader/README.md) for library usage
and HTTP-client integration.

### `umls-downloader-cli`

An interactive terminal application for browsing and downloading releases.
It presents release types and available versions side by side.

Install it from crates.io:

```sh
cargo install umls-downloader-cli
```

Then start it with an API key:

```sh
umls-download --api-key YOUR_KEY
```

Alternatively, provide the key through the environment:

```sh
UMLS_API_KEY=YOUR_KEY umls-download
```

The API key is only required when a download begins. If it was not supplied by
an argument or environment variable, the application opens a masked API-key
prompt and requires a non-empty value.

## CLI controls

| Key | Action |
| --- | --- |
| `Up` / `Down` | Change the selected release type or version |
| `Tab` / `Shift+Tab` | Switch between the two tables |
| `Left` / `Right` | Switch between the two tables |
| `Enter` | Download the selected release version |
| `Esc` | Close the destination prompt or quit the main screen |
| `q` | Quit the main screen |

While downloading:

- Press `Esc` to cancel and return to the release browser.
- Press `q` to cancel and exit the application.

A cancelled or failed download may leave a partial destination file.

When prompted for a destination, press `Enter` with an empty field to save the
archive in the current directory using its original filename. You can also
enter an existing directory or a complete destination filename.

See [`umls-downloader-cli/README.md`](umls-downloader-cli/README.md) for more
CLI details.

## Development

Build the complete workspace:

```sh
cargo build --workspace
```

Run all tests:

```sh
cargo test --workspace
```

Run the CLI directly from the workspace:

```sh
cargo run -p umls-downloader-cli
```

Generate the library documentation:

```sh
cargo doc --workspace --open
```

## UTS API key

Downloads require a UTS API key. You can obtain one from your
[UTS profile](https://uts.nlm.nih.gov/uts/profile).

Avoid committing API keys to source control. Prefer the `UMLS_API_KEY`
environment variable for repeated local use.

## License

The downloader library and CLI are available under the MIT License. See
[`umls-downloader/LICENCE`](umls-downloader/LICENCE) and
[`umls-downloader-cli/LICENCE`](umls-downloader-cli/LICENCE).
