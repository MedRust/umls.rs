//! The example fetches concept search content for given term.
//!
//! ## Requirement:
//!
//! ### Environment Variable:
//! - `UMLS_BASE_URL`: The UMLS REST API server base URL.
//! - `UMLS_VERSION`: The UMLS version in following format: "<year>{AA/AB}" like "2025AA" or "2025AB".
//! - `UMLS_API_KEY`: The UMLS REST API server api key.
//!
//! ## Run:
//! `cargo run --example=concept_search_content`

mod common;

use clap::Parser;
use umls_client::search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = common::Args::parse();
    let config = common::config_from_args(args)?;

    let term = "Cough".parse()?;

    let content = search::Builder::new(term)
        .concept(&config)
        .with_client(Default::default())
        .get()
        .await?
        .into_content()?;

    println!("First 25 concept search result:");
    for index in 0..content.list().len() {
        let concept = content
            .list()
            .get(index)
            .expect("Concept should be present at index");
        println!("{:02}: {}", index + 1, concept);
    }

    Ok(())
}
