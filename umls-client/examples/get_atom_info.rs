//! Fetch atom information of atom "A8345234"(Benign paroxysmal positional vertigo).
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
//! cargo run --example=get_atom_info -- --base-url=<base_url> --api-key=<api_key> --version=2026AA
//! ```
mod common;
use clap::Parser;
use umls_client::{concept::Info, core::aui::Aui, search::core::DisplayName};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = common::Args::parse();
    let config = common::config_from_args(args)?;

    let client = umls_http::reqwest::Client::default();

    let string: DisplayName = "A8345234".parse()?;
    let aui = Aui::new(string)?;

    let response = Info::new(aui).with_client(client).fetch(&config).await?;

    println!("AUI information: {:#?}", response);

    Ok(())
}
