//! A example fetches concept relations of Cough(C0010200) with page 2 of size 1.
mod common;
use clap::Parser;
use umls_client::{
    atom::Batch,
    relation::{Relation, config::Config},
};
use umls_http::reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = common::Args::parse();
    let umls_config = common::config_from_args(args)?;

    let client = Client::default();
    let cui = "C0010200".parse()?;
    let config = Config::builder(cui).build();
    let batch = Batch::new(1.try_into()?, 2.try_into()?, config);

    let page = Relation::new(batch)
        .with_client(client)
        .fetch(&umls_config)
        .await?
        .into_page();

    println!("page number of 2 of size 1 content: {:#?}", page.content());

    let content = page.next()?.fetch(&umls_config).await?.into_content();

    println!("page number of 3 of size 1 content: {:#?}", content);

    Ok(())
}
