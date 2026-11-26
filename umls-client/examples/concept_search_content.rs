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
use umls_client::{search, types};
use umls_http::WithoutClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().try_init();
    let args = common::Args::parse();
    let config = common::config_from_args(args)?;

    let term = "Cough".parse()?;

    let client = umls_http::reqwest::Client::default();

    let concept = umls_client::dto::search::concept::Config::<
        umls_client::dto::search::partial_search::NoPartial,
    >::builder()
    .input(umls_client::dto::search::Input::Atom(term))
    .build()?;

    let batch = umls_client::dto::batch::Batch::new(25.try_into()?, 1.try_into()?, concept);

    let content = search::Search::<WithoutClient, types::Concept, _>::new(batch)
        .with_client(client)
        .fetch(&config)
        .await?
        .into_content();

    println!("First 25 concept search result:");
    for index in 0..content.list().len() {
        let concept = content
            .list()
            .get(index)
            .expect("Concept should be present at index");
        println!("{:02}: {}", index + 1, concept.name());
    }

    Ok(())
}
