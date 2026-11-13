//! Fetch source-asserted concept information using URL.
//!
//! The example uses requires UMLS server base url, api key and version. These requirements can be
//! fulfill either by environment variable or by arguments. The program priorities the environment
//! variables then cli arguments.
//!
//! The environment variable with alternate Cli argumnets as follows:
//! | ENV | argument | Description|
//! |:--- |:--- |:--- |
//! | UMLS_BASE_URL | --base-url | The UMLS base url |
//! | UMLS_API_KEY | --api-key | The UMLS API key |
//! | UMLS_VERSION | --version | The UMLS version |
//!
//! # Example
//! ```sh
//! cargo run --example=get_source_concept_info_using_url -- --base-url=<base_url> --api-key=<api_key> --version=2026AA
//! ```
mod common;
use clap::Parser;
use umls_client::{info::Info, types};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = common::Args::parse();
    let config = common::config_from_args(args)?;

    let client = umls_http::reqwest::Client::default();

    let url: url::Url =
        "https://uts-ws.nlm.nih.gov/rest/content/current/source/SNOMEDCT_US/9468002".parse()?;

    let response = Info::<_, types::Source, _>::new(url)
        .with_client(client)
        .fetch(&config)
        .await?;

    println!("Source asserted cluster: {:#?}", response);

    Ok(())
}
