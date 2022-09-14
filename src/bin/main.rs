extern crate rust_chainlink_ea_api;

use anyhow::{anyhow, Result};
use banyan_shared::eth::VitalikProvider;
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
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set.");
    let contract_address =
        std::env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set.");
    let provider = Arc::new(
        VitalikProvider::new(api_key, contract_address)
            .map_err(|e| anyhow!("error with creating provider: {e}"))?,
    );

    let _rocket = rocket::build()
        .mount("/", rocket::routes![validate, compute])
        .manage(WebserverState(provider))
        .launch()
        .await?;

    Ok(())
}
