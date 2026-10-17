//! An example to fetch a relationships of Cough(C0010200). with default page number and default
//! page size defined by UMLS REST API's. [See documentation](https://documentation.uts.nlm.nih.gov/rest/relations/)
//! for more inforamtion
mod common;
use clap::Parser;
use umls_client::relation::{Relation, config::Config};
use umls_http::reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = common::Args::parse();
    let umls_config = common::config_from_args(args)?;

    let client = Client::default();
    let cui = "C0010200".parse()?;
    let config = Config::builder(cui).build();

    let response = Relation::new(config)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    println!("Response: {:#?}", response);

    Ok(())
}
