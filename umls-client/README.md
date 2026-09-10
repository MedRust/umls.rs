# UMLS Client

Asynchronous concept search, concept retrieval, and atom retrieval with pagination.

## Setup

```toml
[dependencies]
umls-client = { version = "0.1", features = ["reqwest"] }
umls-http = { version = "0.1", features = ["reqwest"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

`Config` holds only the API key, REST base URL, and release version. Create an
HTTP client separately and attach an owned clone to each request. Clones share
the wrapper's connection pool. Search, concept, and atom execution accept
`umls_http::reqwest::Client`. Add `umls-http` as a direct dependency to import it.
The same client can also be cloned into `umls-downloader` requests.

## Search and pagination

```rust,no_run
use umls_http::reqwest;
use umls_client::{config::Config, search::Request};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
    let client = reqwest::Client::default();
    let mut page = Request::builder("cough".parse()?)
        .concept(&config)
        .with_client(client.clone())
        .get().await?
        .into_page()?;
    loop {
        for concept in page.content().list() {
            println!("{}: {}", concept.cui(), concept.name());
        }
        if !page.has_next() {
            break;
        }
        page = page.next().await?.into_page()?;
    }
    Ok(())
}
```

Search `next()` and `previous()` consume the page and execute the next request,
retaining its client and filters. For a single result list, use `into_content()`.
For the wire response, create `search::raw::Search::default().with_client(client)`
and call `.search(&config, &request).await?`; this executor can be reused.

## Concepts and atoms

```rust,no_run
use umls_http::reqwest;
use umls_client::{atom, concept::{Info, cui::Cui}, config::Config};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder(std::env::var("UMLS_API_KEY")?).build()?;
    let client = reqwest::Client::default();
    let concept = Info::new(Cui::new("C0010200")?)
        .with_client(client.clone()).fetch(&config).await?;
    println!("{}", concept.name);

    let page = atom::Config::builder(Cui::new("C0010200")?)
        .try_add_source_vocabulary("MSH")?
        .language("ENG")?
        .atoms(&config)
        .with_client(client.clone())
        .fetch().await?.into_page();
    if page.has_next() {
        let next_page = page.next()?.fetch().await?.into_page();
        println!("{} atoms", next_page.content().list().len());
    }

    let preferred = atom::Preferred::for_concept(&config, Cui::new("C0010200")?)
        .with_client(client.clone()).fetch().await?;
    println!("{}", preferred.name);
    Ok(())
}
```

Atom page navigation returns a request: call `next()?.fetch().await?` or
`previous()?.fetch().await?`. The request retains the client and filters.
Concept `fetch()` consumes its request. `Info::new` accepts a CUI, resource URL,
or concept snapshot; `snapshot.info()` returns a borrowed unconfigured request.

## Configuration and migration

- Replace `Config::builder(key).client(client)` with client attachment on requests.
  The configuration client getter and builder setter have been removed.
- Add `.with_client(client.clone())` before search `get()` or atom/concept `fetch()`.
- Replace the raw search function with the reusable `search::raw::Search` executor.
- Explicit request types now include a client parameter: `Search<'a, C, T>`,
  `Atom<'a, C, S, T>`, and `Info<C, T>`. Builders initially use
  `umls_http::WithoutClient`.
- `Atom::value()`, `Atom::umls_config()`, and `Atom::client()` expose request
  state. `Info::value()` and `Info::client()` remain available after attachment.
- Concept, atom, and raw-search errors now carry a client error type parameter.
  HTTP failures are reported through `Client(umls_http::reqwest::Error)`; inspect
  the underlying error to distinguish status, transport, and decoding failures.

The release defaults to `current`. Use `.try_version("2026AA")?` or a typed
`config::Version` to select a release. Build-time validation checks years from
2018 through the current UTC year, but does not check publication status.
CUI requests use the configured release; explicit resource URLs retain theirs.
Custom REST base URLs must have a trailing slash.

Enable the crate's `reqwest` feature for execution methods. It forwards to
`umls-http/reqwest`; the transport is not enabled by this crate without that
feature. Configuration and generic request builders remain available, but some
response types and atom operation markers also require `reqwest`. Example
programs and HTTP integration tests require this feature.

## Development

```sh
cargo test -p umls-client --features reqwest
cargo check -p umls-client --examples --features reqwest
cargo doc -p umls-client --features reqwest --no-deps --open
UMLS_API_KEY=YOUR_KEY cargo run -p umls-client --features reqwest --example concept_search_pages
```

The interactive examples accept `UMLS_VERSION` or `--version`, defaulting to
`current`. Use `concept_search_content` for a result list or
`concept_info_using_cui` for concept details.
