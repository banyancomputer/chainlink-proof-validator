use anyhow::{anyhow, Result};
use banyan_shared::{eth::EthClient, proofs, proofs::window, types::*};
use log::info;
use rocket::serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainlinkRequestData {
    pub deal_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseData {
    pub deal_id: DealID,
    pub success_count: u64,
    pub num_windows: u64,
    pub status: u16,
    pub result: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainlinkResponse {
    pub data: ResponseData,
}
pub struct WebserverState(pub Arc<EthClient>);

/* Function to construct an error response to return to Chainlink */
fn construct_error(deal_id: DealID, reason: String) -> ChainlinkResponse {
    ChainlinkResponse {
        data: ResponseData {
            deal_id,
            success_count: 0,
            num_windows: 0,
            status: 0,
            result: reason,
        },
    }
}

/// this validates the deal based on a deal_id, returns a json response of either the success count and num_windows,
/// or an error message to be turned into Json<ChainlinkResponse> in the caller!
/// TODO fix logging... :|
pub(crate) async fn validate_deal_internal(
    provider: Arc<EthClient>,
    input_data: ChainlinkRequestData,
) -> Result<ChainlinkResponse> {
    let deal_id = from_str(&input_data.deal_id)?;
    let deal_info = provider
        .get_offer(deal_id)
        .await
        .map_err(|e| anyhow!("Error in get_deal: {:?}", e))?;

    // checking that deal is either finished or cancelled
    let current_block_num = provider
        .get_latest_block_num()
        .await
        .map_err(|e| anyhow!("Couldn't get most recent block number: {e}"))?;

    // TODO: Why have any of these checks in the API. Shouldn't they all be in the Smart Contract Logic.

    let deal_over = EthClient::deal_over(current_block_num, deal_info.clone());
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
            .map_err(|e| anyhow!("Could not get number of windows: {e}"))?;

    // iterating over proof blocks (by window)
    let mut success_count = 0;
    for window_num in 0..num_windows {
        let target_window_start = EthClient::compute_target_block_start(
            deal_info.deal_start_block,
            deal_info.proof_frequency_in_blocks,
            window_num,
        );

        let target_block_hash = provider
            .get_block_hash_from_num(target_window_start)
            .await
            .map_err(|e| anyhow!("Could not get block hash: {e}"))?;

        let submitted_proof_in_block_num = match provider
            .get_proof_block_num_from_window(deal_id, window_num as u64)
            .await
            .map_err(|e| {
                anyhow!("Could not get block where proof was submitted for this window: {e}")
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
                anyhow!(
                    "Couldn't get log from block {}: {}",
                    submitted_proof_in_block_num.0,
                    e
                )
            })? {
            Some(proof) => proof,
            None => {
                info!("Proof is too short for window {}", window_num);
                continue;
            }
        };
        let (chunk_offset, chunk_size) = proofs::compute_random_block_choice_from_hash(
            target_block_hash,
            deal_info.file_size.as_u64(),
        );

        // TODO is there an issue of coercing the Vec<u8> into a &[u8] here?
        match EthClient::check_if_merkle_proof_is_valid(
            Cursor::new(&proof_bytes),
            deal_info.blake3_checksum.hash(),
            chunk_offset,
            chunk_size,
        )
        .map_err(|e| {
            anyhow!(
                "Error reading proof {}: {}",
                submitted_proof_in_block_num.0,
                e
            )
        })? {
            true => {
                info!("Proof succeeded for window {}", window_num);
                success_count += 1;
            }
            false => {
                info!("Proof failed for window {}", window_num);
                continue;
            }
        }
    }
    if num_windows > 0 {
        Ok(ChainlinkResponse {
            data: ResponseData {
                deal_id,
                success_count,
                num_windows: num_windows as u64,
                status: 1,
                result: "Ok".to_string(),
            },
        })
    } else {
        Ok(ChainlinkResponse {
            data: ResponseData {
                deal_id,
                success_count,
                num_windows: num_windows as u64,
                status: 0,
                result: "No windows found".to_string(),
            },
        })
    }
}
