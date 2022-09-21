extern crate rust_chainlink_ea_api;

use anyhow::Result;
use banyan_shared::eth::EthClient;
use rust_chainlink_ea_api::{
    example::compute,
    validate::{validate, WebserverState},
};
use std::sync::Arc;

#[rocket::main]
async fn main() -> Result<()> {
    // TODO command line configuration with clap??
    // TODO use config to handle env variables and conf files!
    dotenv::dotenv().ok();
    let eth_client = Arc::new(EthClient::default());

    let _rocket = rocket::build()
        .mount("/", rocket::routes![validate, compute])
        .manage(WebserverState(eth_client))
        .launch()
        .await?;

    Ok(())
}
