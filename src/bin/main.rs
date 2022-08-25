
#[macro_use] 
extern crate rocket;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate rust_chainlink_ea_api;

use rust_chainlink_ea_api::validate;
use rocket::serde::json::Json;
use eyre::Result;
use validate::{ChainlinkRequest, MyResult};

// check about timeouts with chainlink 

#[post("/val", format = "json", data = "<input_data>")]
async fn val(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {

    /* Call your own function that returns a Json<MyResult>
       (MyResult is consistent with the Chainlink EA specs) */
    validate::validate_deal(input_data).await

}

#[rocket::main]
async fn main() -> Result<()> {

    let _rocket = rocket::build()
        .mount("/", routes![val])
        .launch()
        .await?;

    Ok(())
}
