#[macro_use]
extern crate rocket;
extern crate rust_chainlink_ea_api;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
// alphabetize and organize
use ethers::providers::{Http, Middleware, Provider};
use eyre::Result;
use rocket::{serde::{json::Json, Deserialize, Serialize}};
use rust_chainlink_ea_api::validate;
use std::time::Duration;
use validate::{ChainlinkRequest, MyResult};

//command line configuration with clap??
// config is the library for conf files (i might be wrong but i'm definitely using the right one in my code so check there)

/* Example data structures */
#[derive(Serialize, Deserialize, Debug)]
struct ExampleRequestData {
    block_num: u64,
}
#[derive(Serialize, Deserialize, Debug)]
struct ExampleRequest {
    id: String,
    data: ExampleRequestData,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExampleResponse {
    data: Duration,
    status: u16,
    result: String,
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
        result: reason,
    })
}

/* let blabla = Result<Json<ER>, String> .... ;
blabla.map_err(|x| construct_error(x));

let blabla = {
statement_that_can_give_a_construct_err()?;
doodoodoo.....
eventually the nice thing that builds the real response!!!;
}

 */
// remove logic from main.rs
async fn compute_internal(input_data: Json<ExampleRequest>) -> Result<Json<ExampleResponse>, String> {
    let block_num = input_data.into_inner().data.block_num;

    let provider =
        Provider::<Http>::try_from("https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe")
            .expect("could not instantiate HTTP Provider");

    let current_block_num = provider.get_block_number().await
        .map_or_else(
            |e| Err(format!("Could not get block number: {e}")),
            |n| Ok(n.as_u64()))?;
    println!("current block num: {}", current_block_num);
    if block_num > current_block_num {
        return Err(format!("Block number {block_num} is invalid."))
    };

    let current_block = provider.get_block(current_block_num).await.map_err(|e| format!("Could not get block number {}: {}", current_block_num, e))?.ok_or(format!("Could not get block number {}", current_block_num))?;

    let current_block_time = current_block.time().map_err(|e| format!("{e}"))?;

    let target_block = provider.get_block(block_num).await.map_err(|e| format!("Could not get block number {}: {}", block_num, e))?.ok_or(format!("Could not get block number {}", block_num))?;

    let target_block_time = target_block.time().map_err(|e| format!("{e}"))?;

    println!("target block time: {}", target_block_time); //use log package when the time comes to ship
    println!("current block time: {}", current_block_time);
    let data = current_block_time.signed_duration_since(target_block_time).to_std().map_err(|e| format!("Could not convert time to standard: {e}"))?;

    println!("difference: {:?}", data);

    Ok(Json(ExampleResponse {
        data,
        status: 200,
        result: "Ok".into(),
    }))
}

#[post("/compute", format = "json", data = "<input_data>")]
async fn compute(input_data: Json<ExampleRequest>) -> Json<ExampleResponse> {
    compute_internal(input_data).await.map_or_else(|e| construct_error(e), |v| v)
}

#[rocket::main]
async fn main() -> Result<()> {
    let _rocket = rocket::build()
        .mount("/", routes![val, compute])
        .launch()
        .await?;

    Ok(())
}
