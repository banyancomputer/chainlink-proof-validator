use anyhow::{anyhow, Result};
use ethers::prelude::Middleware;
use ethers::providers::{Http, Provider};
use rocket::serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// CHANGE ME TO WHAT YOU EXPECT TO GET FROM CHAIN!
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExampleRequestData {
    pub block_num: u64,
}

/// CHANGE ME TO WHAT YOU WANT TO RETURN TO CHAIN!
#[derive(Serialize, Deserialize, Debug)]
pub struct ExampleResponseData {
    pub duration: Duration,
}

/// leave the types the same on this function, but change the body to do what you want.
/// in our example, we let the user put in a block number, and we return the elapsed time (according to ethereum) since that block.
/// this is a very simple example, but you can do anything you want here.
pub(crate) async fn compute_internal(
    provider: Arc<Provider<Http>>,
    input_data: ExampleRequestData,
) -> Result<ExampleResponseData> {
    let block_num = input_data.block_num;

    // Getting the most recent block number
    let current_block_num = provider.get_block_number().await.map_or_else(
        |e| Err(anyhow!("Could not get block number: {e}")),
        |n| Ok(n.as_u64()),
    )?;

    // Checking if the input block number is valid- i.e., has already been mined.
    if block_num > current_block_num {
        Err(anyhow!("Block number {block_num} is invalid."))?
    };

    // Getting the most recent block
    let current_block = provider
        .get_block(current_block_num)
        .await?
        .ok_or_else(|| anyhow!("Block number {current_block_num} is invalid."))?;

    // Getting the input block
    let target_block = provider
        .get_block(block_num)
        .await?
        .ok_or_else(|| anyhow!("Could not get block number {}", block_num))?;

    // Getting the time of the most recent block
    let current_block_time = current_block.time()?;

    // Getting the time of the input block
    let target_block_time = target_block.time()?;

    // Time since target block
    let duration = current_block_time
        .signed_duration_since(target_block_time)
        .to_std()
        .map_err(|e| anyhow!("Could not convert duration to std::time::Duration: {e}"))?;

    Ok(ExampleResponseData { duration })
}
