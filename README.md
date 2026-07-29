# UMLS.rs

Rust tools for discovering and downloading releases from the U.S. National
Library of Medicine's UMLS Terminology Services (UTS).

The workspace supports release files for:

- UMLS
- RxNorm
- SNOMED CT

## Workspace crates

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

Both workspace crates are available under the MIT License. See
[`umls-downloader/LICENCE`](umls-downloader/LICENCE) and
[`umls-downloader-cli/LICENCE`](umls-downloader-cli/LICENCE).
