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
use ethers::{types::{Filter, H256}, utils::get_contract_address};
use log::info;
use rocket::{
    post,
    serde::{json::Json, Deserialize, Serialize},
    State,
};
use std::io::{Cursor, Read};
use std::sync::Arc;

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
            window_num); 

        let target_block_hash = provider
            .get_block_hash_from_num(target_window_start)
            .await
            .map_err(|e| format!("Could not get block hash: {e}"))?;

        // step b. above
        // TODO: perhaps this just means they didn't submit? shouldn't just error here.
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

        // TODO these should be rolled up into one function in eth_shared pls
        let filter = Filter::new()
            .select(submitted_proof_in_block_num.0)
            // TODO figure this guy out later :)
            .address(provider.get_contract_address())
            .topic1(H256::from_low_u64_be(deal_id.0));

        let block_logs = provider.get_logs_from_filter(&filter).await.map_err(|e| {
            format!(
                "Couldn't get log from block {}: {}",
                submitted_proof_in_block_num.0, e
            )
        })?;

        // The first two 32 byte words of log data are a pointer and the size of the data.
        // TODO put this in banyan_shared!
        let data = &block_logs[0].data;
        if data.len() < WORD * 2 {
            return Err(format!("Data is too short: {:?}", data));
        }

        // TODO yeah there's definitely a bug here. this slice is only 32 bytes long, not 64.
        // TODO This works probably as a solution but might be a bug. I could submit a really long block, 
        // but since only takes last 8 bytes, it could slice the length value such that it read the
        // length as the actual size. That would cause it to only read the first x bytes of the proof,
        // and then it could read an incorrect proof as correct. BUTT the proof would have to have to 
        // be incorrect in such a way that it had the correct proof in the first x bytes, and then other 
        // incorrect stuff after, which means someone would have had to construct an correct proof and then
        // decide to append other stuff for some reason. I don't know how that would help an attacker. 
        // Is this a bug? I am not sure. 

        let mut a = [0u8; 8];
            a.clone_from_slice(&data[(WORD * 2) - 8..WORD * 2]);
            println!("a: {:?}", a);
        let data_size = u64::from_be_bytes({
            //let mut a = [0u8; 8];
            //a.clone_from_slice(&data[WORD..WORD + 8]);
            a
        });
        println!("data size: {}", data_size);
        if data.len() < WORD * 2 + data_size as usize {
            return Err(format!("Proof is too short: {:?}", data));
        }

        let data_bytes = &data[WORD * 2..WORD * 2 + data_size as usize];
        let proof_bytes = Cursor::new(data_bytes);

        // step e. above
        let (chunk_offset, chunk_size) =
            proofs::compute_random_block_choice_from_hash(target_block_hash, deal_info.file_size);

        println!("target window: {}", target_window_start.0);
        println!("chunk offset val: {}", chunk_offset);
        // step f. above
        // TODO put this in banyan shared. 
        if bao::decode::SliceDecoder::new(
            proof_bytes,
            &(deal_info.blake3_checksum),
            chunk_offset,
            chunk_size,
        )
        .read_to_end(&mut vec![])
        .is_ok()
        {
            success_count += 1
        } else {
            info!("Proof failed for window {}", window_num);
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