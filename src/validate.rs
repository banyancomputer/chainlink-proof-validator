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

// TODO codecov, cicd
*/
use anyhow::Result;
use banyan_shared::{eth::VitalikProvider, proofs, proofs::window, types::*};
use log::info;
use rocket::{
    post,
    serde::{json::Json, Deserialize, Serialize},
    State,
};
use std::io::{Cursor};
use std::sync::Arc;

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
pub struct WebserverState(pub Arc<VitalikProvider>);

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
// TODO this needs to go into banyan_shared
fn deal_over(current_block_num: BlockNum, deal_info: OnChainDealInfo) -> bool {
    current_block_num > deal_info.deal_start_block + deal_info.deal_length_in_blocks
}

/// this validates the deal based on a deal_id, returns a json response of either the success count and num_windows,
/// or an error message to be turned into Json<MyResult> in the caller!
/// TODO fix logging... :|
async fn validate_deal_internal(
    provider: Arc<VitalikProvider>,
    deal_id: DealID,
) -> Result<Json<MyResult>, String> {
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
    let deal_cancelled = false; // TODO need to figure out how to get this

    // this refuses to do the validation computations unless the deal is done with or cancelled
    if !deal_over && !deal_cancelled {
        return Ok(construct_error(deal_id, "Deal is ongoing".to_string()));
    }

    // this computes the actual deal length based on potential cancellation? and gets real number of windows :)
    let agreed_upon_cancellation_block: BlockNum = BlockNum(0u64); // need to figure out how to get this
    let deal_length_in_blocks = if deal_cancelled {
        agreed_upon_cancellation_block - deal_info.deal_start_block
    } else {
        deal_info.deal_length_in_blocks
    };
    let num_windows =
        window::get_num_windows(deal_length_in_blocks, deal_info.proof_frequency_in_blocks)
            .map_err(|e| format!("Could not get number of windows: {e}"))?;

    // iterating over proof blocks (by window)
    let mut success_count = 0;
    for window_num in 0..num_windows {
        let target_window_start = VitalikProvider::compute_target_window_start(
            deal_info.deal_start_block,
            deal_info.proof_frequency_in_blocks,
            window_num,
        );

        let target_block_hash = provider
            .get_block_hash_from_num(target_window_start)
            .await
            .map_err(|e| format!("Could not get block hash: {e}"))?;

        let submitted_proof_in_block_num = match provider
            .get_proof_block_num_from_window(deal_id, window_num as u64)
            .await
            .map_err(|e| {
                format!("Could not get block where proof was submitted for this window: {e}")
            })? {
            Some(block_num) => block_num,
            None => {
                info!("No proof submitted for window {}", window_num);
                continue;
            }
        };

        let proof_bytes: Vec<u8> = match provider
            .get_proof_from_logs(submitted_proof_in_block_num, deal_id)
            .await
            .map_err(|e| {
                format!(
                    "Couldn't get log from block {}: {}",
                    submitted_proof_in_block_num.0, e
                )
            })? {
            Some(proof) => proof,
            None => {
                info!("Proof is too short for window {}", window_num);
                continue;
            }
        };

        let (chunk_offset, chunk_size) =
            proofs::compute_random_block_choice_from_hash(target_block_hash, deal_info.file_size);

        // TODO is there an issue of coercing the Vec<u8> into a &[u8] here?
        match VitalikProvider::check_if_merkle_proof_is_valid(
            Cursor::new(&proof_bytes),
            deal_info.blake3_checksum,
            chunk_offset,
            chunk_size,
        )
        .map_err(|e| {
            format!(
                "Error reading proof {}: {}",
                submitted_proof_in_block_num.0, e
            )
        })? {
            true => success_count += 1,
            false => {
                info!("Proof failed for window {}", window_num);
                continue;
            }
        }
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

#[post("/validate", format = "json", data = "<input_data>")]
pub async fn validate(
    webserver_state: &State<WebserverState>,
    input_data: Json<ChainlinkRequest>,
) -> Json<MyResult> {
    let deal_id = input_data.into_inner().data.deal_id;
    let eth_provider = webserver_state.0.clone();
    validate_deal_internal(eth_provider, deal_id)
        .await
        .map_or_else(|e| construct_error(deal_id, e), |v| v)
}
