use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::post;
use std::time::Duration;
use ethers::providers::{Http, Middleware, Provider};

/* Example data structures */
#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    Success,
    Faliure
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExampleRequestData {
    pub block_num: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ExampleRequest {
    pub id: String,
    pub data: ExampleRequestData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExampleResponse {
    pub data: Duration,
    pub status: Status,
    pub result: String,
}

/* Function to construct an error response to be returned to Chainlink */
fn construct_error(reason: String) -> Json<ExampleResponse> {
    Json(ExampleResponse {
        data: Duration::new(0, 0),
        status: Status::Faliure,
        result: reason,
    })
}


async fn compute_internal(
    input_data: Json<ExampleRequest>,
) -> Result<Json<ExampleResponse>, String> {
    let block_num = input_data.into_inner().data.block_num;
    let api_token = std::env::var("API_KEY").expect("API_KEY must be set.");
    let provider =
        Provider::<Http>::try_from(api_token)
            .expect("could not instantiate HTTP Provider");

    // Getting the most recent block number
    let current_block_num = provider.get_block_number().await.map_or_else(
        |e| Err(format!("Could not get block number: {e}")),
        |n| Ok(n.as_u64()),
    )?;

    // Checking if the input block number is valid
    if block_num > current_block_num {
        return Err(format!("Block number {block_num} is invalid."));
    };

    // Getting the most recent block
    let current_block = provider
        .get_block(current_block_num)
        .await
        .map_err(|e| format!("Could not get block number {}: {}", current_block_num, e))?
        .ok_or(format!("Could not get block number {}", current_block_num))?;

    // Getting the time of the most recent block
    let current_block_time = current_block.time().map_err(|e| format!("{e}"))?;

    // Getting the actual input block
    let target_block = provider
        .get_block(block_num)
        .await
        .map_err(|e| format!("Could not get block number {}: {}", block_num, e))?
        .ok_or(format!("Could not get block number {}", block_num))?;

    // Getting the time of the input block
    let target_block_time = target_block.time().map_err(|e| format!("{e}"))?;

    // Time since target block
    let data = current_block_time
        .signed_duration_since(target_block_time)
        .to_std()
        .map_err(|e| format!("Could not convert time to standard: {e}"))?;

    Ok(Json(ExampleResponse {
        data,
        status: Status::Success,
        result: "Ok".into(),
    }))
}

#[post("/compute", format = "json", data = "<input_data>")]
pub async fn compute(input_data: Json<ExampleRequest>) -> Json<ExampleResponse> {
    compute_internal(input_data)
        .await
        .map_or_else(|e| construct_error(e), |v| v)
}