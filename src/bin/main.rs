
#[macro_use] 
extern crate rocket;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate rust_chainlink_ea_api;

use rust_chainlink_ea_api::validate;
use rocket::serde::{Serialize, Deserialize, json::Json};
use eyre::Result;
use validate::{ChainlinkRequest, MyResult};
use ethers::providers::{Provider, Middleware, Http};
use std::time::Duration;

/* Example data structures */
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ExampleRequestData {
    block_num: u64
}
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ExampleRequest {
    id: String,
    data: ExampleRequestData
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ExampleResponse {
    data: Duration,
    status: u16,
    result: String
}

// check about timeouts with chainlink 

#[post("/val", format = "json", data = "<input_data>")]
async fn val(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {

    /* Call your own function that returns a Json<MyResult>
       (MyResult is consistent with the Chainlink EA specs) */
    validate::validate_deal(input_data).await

}

fn construct_error(reason: String) -> Json<ExampleResponse> {
    Json(ExampleResponse {
        data: Duration::new(0, 0),
        status: 500,
        result: reason
    })
}

#[post("/compute", format = "json", data = "<input_data>")]
async fn compute(input_data: Json<ExampleRequest>) -> Json<ExampleResponse> {

    let block_num = input_data.into_inner().data.block_num;

    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe")
        .expect("could not instantiate HTTP Provider");

    let current_block_num = match provider.get_block_number().await {
        Ok(n) => n.as_u64(),
        Err(e) => 
            return construct_error(format!("Could not get block number: {e}"))
    };
    println!("current block num: {}", current_block_num);
    if block_num < 0 || block_num > current_block_num {
        return construct_error(format!("Block number {block_num} is invalid."))
    }

    let current_block = match provider.get_block(current_block_num).await {
        Ok(b) => b,
        Err(e) => 
            return construct_error(format!("Could not get block number {}: {}", current_block_num, e))
    };
    let current_block_time = match current_block.unwrap().time() {
        Ok(dt) => dt,
        Err(e) => return construct_error(format!("{e}"))
    };
    let target_block = match provider.get_block(block_num).await {
        Ok(b) => b,
        Err(e) => 
            return construct_error(format!("Could not get block number {}: {}", block_num, e))
    };
    let target_block_time = match target_block.unwrap().time() {
        Ok(dt) => dt,
        Err(e) => return construct_error(format!("{e}"))
    };
    println!("target block time: {}", target_block_time);
    println!("current block time: {}", current_block_time);
    let diff = match current_block_time.signed_duration_since(target_block_time).to_std() {
        Ok(d) => d,
        Err(e) => 
            return construct_error(format!("Could not convert time to standard: {e}"))
    };
    println!("difference: {:?}", diff);
    let status = 200;
    let result = "Ok".to_string();

    Json(ExampleResponse {
        data: diff,
        status: status,
        result: result
    })
}

#[rocket::main]
async fn main() -> Result<()> {

    let _rocket = rocket::build()
        .mount("/", routes![val, compute])
        .launch()
        .await?;

    Ok(())
}
