/*
1. when it gets a request, find the deal_id’s info on chain
2. check that the deal is either FINISHED (current_block_num > deal_start_block
   + deal_length_in_blocks) or CANCELLED (and do the computations below with
   deal_length_in_blocks := (agreed_upon_cancellation_block - deal_start).
3. start iterating over proof_blocks  from window_num \in (0, num_windows),
   num_windows = ceiling(deal_length_in_blocks / window_size)
        a. if there isn’t a proof recorded in proof_blocks under that window, continue
        b. find the proof in that block’s logs, stick it in proof_bytes
        c. if there is, set target_window_start to window_num * window_size + deal_start_block
        d. get the target_block_hash as block_hash(target_window_start)
        e. get the chunk_offset and chunk_size according to the function
           compute_random_block_choice_from_hash(target_block_hash, deal_info.file_length)
           defined in my code here: https://github.com/banyancomputer/ipfs-proof-buddy/blob/9f0ae728f7a103da615c5eedf37491267f470e48/src/proof_utils.rs#L17
           (by the way let’s not copy-paste or reimplement this- let’s make a banyan-shared
            crate when you get to this)
        f. validate the proof, and if you pass, increment success_count
4. then once you get done with iterating over all the proofs, return
   (success_count, num_windows)  and whatever id/deal_id you need in order
   to identify the computation performed back to chain
*/

/* lazy static macro - needs to be thread safe, maybe use to instantiate a provider
or maybe instance methods
open provider once

cargo fmt, then cargo check, then cargo clippy
cargo build - default is debug mode, does overflow checking
cargo build --release for benchmarking

codecov, cicd
*/
use anyhow::Result;
use banyan_shared::{eth::VitalikProvider, proofs, proofs::window, types::*};
use dotenv::dotenv;
use ethers::types::{Filter, H256, Bytes};
use rocket::{
    post,
    serde::{json::Json, Deserialize, Serialize},
};
use std::io::{Cursor, Read, Seek};

const WORD: usize = 32;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainlinkRequest {
    pub job_run_id: String,
    pub data: RequestData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestData {
    pub deal_id: DealID,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub deal_id: DealID,
    pub success_count: u64,
    pub num_windows: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResult {
    pub data: ResponseData,
    pub status: Status,
    pub result: String,
}

/* Function to construct an error response to return to Chainlink */
fn construct_error(deal_id: DealID, reason: String) -> Json<MyResult> {
    Json(MyResult {
        data: ResponseData {
            deal_id,
            success_count: 0,
            num_windows: 0,
        },
        status: Status::Failure,
        result: reason,
    })
}

/* Function to check if the deal is over or not */
fn deal_over(current_block_num: BlockNum, deal_info: OnChainDealInfo) -> bool {
    current_block_num > deal_info.deal_start_block + deal_info.deal_length_in_blocks
}

async fn validate_deal_internal(deal_id: DealID) -> Result<Json<MyResult>, String> {
    dotenv().ok();
    let mut success_count = 0;
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set.");
    let contract_address =
        std::env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set.");
    let provider = VitalikProvider::new(api_key, contract_address)
        .map_err(|e| format!("error with creating provider: {e}"))?;

    let deal_info = provider
        .get_onchain(deal_id)
        .await
        .map_err(|e| format!("Error in get_onchain: {:?}", e))?;

    // checking that deal is either finished or cancelled
    let current_block_num = provider
        .get_latest_block_num()
        .await
        .map_err(|e| format!("Couldn't get most recent block number: {e}"))?;

    let deal_over = deal_over(current_block_num, deal_info);
    let deal_cancelled = false; // need to figure out how to get this

    if !deal_over && !deal_cancelled {
        return Ok(construct_error(deal_id, "Deal is ongoing".to_string()));
    }

    let agreed_upon_cancellation_block: BlockNum = BlockNum(0u64); // need to figure out how to get this
    let deal_length_in_blocks = match deal_cancelled {
        false => deal_info.deal_length_in_blocks,
        true => agreed_upon_cancellation_block - deal_info.deal_start_block,
    };
    let window_size = deal_info.proof_frequency_in_blocks;
    // this should be in window_uril.rs and tested meticulously (it might eb in the proof_gen.rs thing)
    let num_windows = window::get_num_windows(deal_length_in_blocks, window_size)
        .map_err(|e| format!("Could not get number of windows: {e}"))?;

    // iterating over proof blocks (by window)
    for window_num in 1..num_windows + 1 {
        // step b. above
        println!("window_num: {}", window_num);
        let block_num = provider
            .get_block_num_from_window(deal_id, window_num as u64)
            .await
            .map_err(|e| format!("Could not get block: {e}"))?;

        println!("block_num: {:?}", block_num);
        /*
        let block_num = match provider.get_block_num_from_window(deal_id, window_num as u64).await {
            Ok(it) => it,
            Err(e) => println!("Could not get block: {e}"),
        };
        */

        let filter: Filter = Filter::new()
            .select(block_num.0)
            .topic1(H256::from_low_u64_be(deal_id.0));

        let block_logs = provider
            .get_logs_from_filter(&filter)
            .await
            .map_err(|e| format!("Couldn't get logs from block {}: {}", block_num.0, e))?;
       
        println!("LOGs {:?}", block_logs);
        // The first two 32 byte words of log data are a pointer and the size of the data. 
        println!("error past here");
        let data = &block_logs[0].data;
        let data_size = data.get(WORD..(WORD*2)).unwrap();

        let hex_data = hex::encode(data_size);
        let size_int = usize::from_str_radix(&hex_data, 16).unwrap();
        let end: usize = size_int + (WORD*2);
        let data_bytes = data.get((WORD*2)..end).unwrap();
        let proof_bytes = Cursor::new(data_bytes);
        println!("error not past here");
        // step c. above
        let target_window_start: BlockNum = window_size * window_num + deal_info.deal_start_block;

        // step d. above
        let target_block_hash = provider
            .get_block_hash_from_num(target_window_start)
            .await
            .map_err(|e| {
                format!(
                    "Could not get hash of block number {}: {}",
                    target_window_start.0, e
                )
            })?;

        // step e. above
        let (chunk_offset, chunk_size) =
            proofs::compute_random_block_choice_from_hash(target_block_hash, deal_info.file_size);
        
        println!("target window: {}", target_window_start.0);
        println!("chunk offset val: {}", chunk_offset);
        // step f. above
        let mut decoded = Vec::new();
        let mut decoder = bao::decode::SliceDecoder::new(
            proof_bytes,
            &(deal_info.blake3_checksum),
            chunk_offset,
            chunk_size,
        );

        match decoder.read_to_end(&mut decoded) {
            Ok(_res) => success_count += 1,
            Err(_e) => println!("Error in decoding: {:?}", window_num),
        };
    }
    if num_windows > 0 {
        Ok(Json(MyResult {
            data: ResponseData {
                deal_id,
                success_count,
                num_windows: num_windows as u64,
            },
            status: Status::Success,
            result: "Ok".to_string(),
        }))
    } else {
        Ok(Json(MyResult {
            data: ResponseData {
                deal_id,
                success_count,
                num_windows: num_windows as u64,
            },
            status: Status::Failure,
            result: "No windows found".to_string(),
        }))
    }
}

async fn validate_deal(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {
    let deal_id = input_data.into_inner().data.deal_id;
    validate_deal_internal(deal_id)
        .await
        .map_or_else(|e| construct_error(deal_id, e), |v| v)
}

#[post("/validate", format = "json", data = "<input_data>")]
pub async fn validate(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {
    validate_deal(input_data).await
}
