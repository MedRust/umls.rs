//! Example fetches source-asserted concept information of code `9468002` with source vocabulary
//! Snomed CT US Edition(SNOMEDCT_US).
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
//! cargo run --example=get_source_concept_info_using_source_concept -- --base-url=<base_url> --api-key=<api_key> --version=2026AA
//! ```

mod common;
use clap::Parser;
use umls_client::{dto::source::SourceConcept, info::Info, types};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = common::Args::parse();
    let config = common::config_from_args(args)?;

    let client = umls_http::reqwest::Client::default();

    let source_concept = SourceConcept::new("SNOMEDCT_US".parse()?, "9468002".parse()?);

    let response = Info::<_, types::Source, _>::new(source_concept)
        .with_client(client)
        .fetch(&config)
        .await?;

    println!("Source asserted cluster: {:#?}", response);

    Ok(())
}
