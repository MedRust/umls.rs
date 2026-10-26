//! Fetch source asserted atoms of Snomed CT US edition with unique identifier "9468002" with page
//! size 1. The example retrieve first and sencond page with 5 seconds delay.
//!
//! The example requires UMLS server base url, api key and version. These requirements can be
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
//! cargo run --example=get_source_parent_concepts -F reqwest -- --base-url=<base_url> --api-key=<api_key> --version=2026AA
//! ```
use std::time::Duration;

use clap::Parser;
use tokio::time::sleep;
use umls_client::{
    atom::Batch,
    parent::{Parent, SourceConcept},
    search::core::DisplayName,
};

mod common;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = common::Args::parse();
    let umls_config = common::config_from_args(args)?;

    let client = umls_http::reqwest::Client::default();

    let source_vocabulary: DisplayName = "SNOMEDCT_US".parse()?;
    let code: DisplayName = "9468002".parse()?;

    let source_concept = SourceConcept::new(source_vocabulary, code);
    let batch = Batch::new(1.try_into()?, 1.try_into()?, source_concept);

    let page = Parent::new(batch)
        .with_client(client)
        .fetch(&umls_config)
        .await?
        .into_page();

    println!("Page 1: {:#?}", page.content());

    // Sleep for 5 seconds.
    sleep(Duration::from_secs(5)).await;

    let page = page.next()?.fetch(&umls_config).await?.into_page();

    println!("Page 2: {:#?}", page.content());

    Ok(())
}
