#[macro_use]
extern crate rocket;
extern crate rust_chainlink_ea_api;

use eyre::Result;
use rust_chainlink_ea_api::{example::compute, validate::validate};

//command line configuration with clap??
// config is the library for conf files (i might be wrong but i'm definitely using the right one in my code so check there)

#[rocket::main]
async fn main() -> Result<()> {
    let _rocket = rocket::build()
        .mount("/", routes![validate, compute])
        .launch()
        .await?;

    Ok(())
}
