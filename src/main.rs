#![deny(unused_crate_dependencies)]

//use rust_chainlink_ea_api::validate::*;
pub mod validate;

use anyhow::Result;
use banyan_shared::{eth::EthClient, types::DealID};
use ethers as _;
use rand::Rng;
use rocket::serde::{json::serde_json, json::Json, Deserialize, Serialize};
use rocket::tokio::task::spawn;
use rocket::{post, State};
use std::sync::Arc;
use tokio as _;

pub struct WebserverState {
    pub provider: Arc<EthClient>,
    pub should_be_async: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainlinkEARequest {
    pub id: String,
    pub data: validate::ChainlinkRequestData,
    pub meta: Option<serde_json::Value>,
    pub response_url: Option<String>,
}

fn format_response(
    result: Result<validate::ChainlinkResponse, anyhow::Error>,
) -> Json<serde_json::Value> {
    match result {
        Ok(data) => Json(serde_json::json!(data)),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

// TODO prefix all logs with ID from request
#[post("/compute", format = "json", data = "<input_data>")]
pub async fn compute(
    webserver_state: &State<WebserverState>,
    input_data: Json<ChainlinkEARequest>,
) -> Json<serde_json::Value> {
    if webserver_state.should_be_async {
        let new_provider = webserver_state.provider.clone();
        spawn(async move {
            let result =
                validate::validate_deal_internal(new_provider, input_data.data.clone()).await;
            // send the result to the chainlink node
            reqwest::Client::new()
                .patch(input_data.into_inner().response_url.unwrap())
                .body(format_response(result).to_string())
                .send()
                .await
                .unwrap();
        });
        Json(serde_json::json!({
            "pending": true
        }))
        // end of thread
    } else {
        format_response(
            validate::validate_deal_internal(
                webserver_state.provider.clone(),
                input_data.data.clone(),
            )
            .await,
        )
    }
}

#[rocket::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let should_be_async = std::env::var("SHOULD_BE_ASYNC")
        .map_or_else(|_| false, |n| n.parse::<bool>().unwrap_or(false));

    // create an ethers HTTP provider
    let eth_client = Arc::new(EthClient::default());

    let _ = rocket::build()
        .mount("/", rocket::routes![compute])
        .manage(WebserverState {
            provider: eth_client,
            should_be_async,
        })
        .launch()
        .await?;

    Ok(())
}

/// Helper function for testing inputs to Chainlink EA without having to run a node.
pub async fn rust_chainlink_ea_api_call(
    deal_id: DealID,
    api_url: String,
) -> Result<validate::ChainlinkResponse, anyhow::Error> {
    // Job id when chainlink calls is not random.
    let mut rng = rand::thread_rng();
    let random_job_id: u16 = rng.gen();
    let map = serde_json::json!({
        "id": random_job_id.to_string(),
        "data":
        {
             "deal_id": deal_id.0.to_string()
        }
    });
    let client = reqwest::Client::new();
    let res = client
        .post(api_url)
        .json(&map)
        .send()
        .await?
        .json::<validate::ChainlinkResponse>()
        .await?;
    dbg!("{:?}", &res);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use banyan_shared::{
        deals::DealProposalBuilder,
        eth::EthClient,
        types::{BlockNum, DealID, DealProposal},
    };
    use ethers::types::Bytes;
    use std::fs::File;

    #[tokio::test]
    /// This test will fail if no deal has ever been created on the contract, or if that deal has valid proofs in it.
    async fn api_call_test() -> Result<(), anyhow::Error> {
        let response_data: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(DealID(1), "http://127.0.0.1:8000/compute".to_string())
                .await?;
        assert_eq!(response_data.data.success_count, 1);
        Ok(())
    }

    #[tokio::test]
    /// This tests verifies that a deal with no logged proofs will have a success count of 0
    async fn api_no_proofs_test() -> Result<(), anyhow::Error> {
        let file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let deal_proposal = DealProposal::builder().with_file(file).build().unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal, None, None)
            .await
            .expect("Failed to send deal proposal");

        dbg!("Offer Created for Deal ID: {:}", deal_id);
        let deal = eth_client.get_offer(deal_id).await.unwrap();
        // Assert that the deal we read is the same as the one we sent
        dbg!("deal {:?}", deal.clone());
        assert_eq!(deal.deal_length_in_blocks, BlockNum(10));

        let response_data: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/compute".to_string())
                .await?;
        assert_eq!(response_data.data.success_count, 0);
        Ok(())
    }

    #[tokio::test]
    /// This test verifies that an eth client can create multiple consecutive deals, which all have no logged proofs.
    async fn multiple_deals_same_file_no_proofs() -> Result<(), anyhow::Error> {
        let file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let deal_proposal = DealProposal::builder().with_file(file).build().unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        dbg!("Proof Created for Deal ID: {:}", &deal_id);
        let deal = eth_client.get_offer(deal_id).await.unwrap();
        // Assert that the deal we read is the same as the one we sent
        assert_eq!(deal.deal_length_in_blocks, BlockNum(10));

        let deal_id_2: DealID = eth_client
            .propose_deal(deal_proposal, None, None)
            .await
            .expect("Failed to send deal proposal");
        dbg!("Proof Created for Deal ID: {:}", &deal_id_2);
        let deal = eth_client.get_offer(deal_id_2).await.unwrap();
        assert_eq!(deal.deal_length_in_blocks, BlockNum(10));

        let response_data_1: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/compute".to_string())
                .await?;
        assert_eq!(response_data_1.data.success_count, 0);
        let response_data_2: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id_2, "http://127.0.0.1:8000/compute".to_string())
                .await?;
        assert_eq!(response_data_2.data.success_count, 0);
        Ok(())
    }

    #[tokio::test]
    /// This test verifies that we can create a deal with one window, submit a proof, and verify it.
    async fn deal_and_proof_one_window() -> Result<(), anyhow::Error> {
        let mut file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();

        let deal_proposal: DealProposal = DealProposalBuilder::new(
            "0x0000000000000000000000000000000000000000".to_string(),
            3,
            3,
            0.0,
            0.0,
            "0x0000000000000000000000000000000000000000".to_string(),
        )
        .with_file(file.try_clone()?)
        .build()
        .unwrap();

        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        let deal = eth_client.get_offer(deal_id).await.unwrap();

        // create a proof using the same file we used to create the deal
        let (_hash, proof) = eth_client
            .create_proof_helper(
                deal.deal_start_block,
                &mut file,
                deal.file_size.as_u64(),
                true,
            )
            .await
            .expect("Failed to create proof");

        // Wait one block until current block is no longer the deal start block
        let mut current_block_num = deal.deal_start_block;
        while current_block_num == deal.deal_start_block {
            current_block_num = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        let target_block = deal.deal_start_block;
        let _block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, target_block, None, None)
            .await
            .expect("Failed to post proof");

        let mut current_block_num_2 = BlockNum(0);
        while !EthClient::deal_over(current_block_num_2, deal.clone()) {
            current_block_num_2 = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        let response_data: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/compute".to_string())
                .await?;

        assert_eq!(response_data.data.success_count, 1);
        Ok(())
    }

    #[tokio::test]
    /// This test verifies that we can create a deal with two window, post one proof and simply not post a second, and recieve
    /// A success count of 1.
    async fn one_proof_window_missing() -> Result<(), anyhow::Error> {
        let mut file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();

        let deal_proposal: DealProposal = DealProposalBuilder::new(
            "0x0000000000000000000000000000000000000000".to_string(),
            6,
            3,
            0.0,
            0.0,
            "0x0000000000000000000000000000000000000000".to_string(),
        )
        .with_file(file.try_clone()?)
        .build()
        .unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        let deal = eth_client.get_offer(deal_id).await.unwrap();

        let target_window: usize = eth_client
            .compute_target_window(deal.deal_start_block, deal.proof_frequency_in_blocks)
            .await
            .expect("Failed to compute target window");

        let target_block = EthClient::compute_target_block_start(
            deal.deal_start_block,
            deal.proof_frequency_in_blocks,
            target_window,
        );

        // Wait one block until current block is no longer the deal start block
        let mut current_block_num = deal.deal_start_block;
        while current_block_num == deal.deal_start_block {
            current_block_num = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        // create a proof using the same file we used to create the deal
        let (_hash, proof) = eth_client
            .create_proof_helper(target_block, &mut file, deal.file_size.as_u64(), true)
            .await
            .expect("Failed to create proof");

        let block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, target_block, None, None)
            .await
            .expect("Failed to post proof");

        // Wait for the second window to start
        // checking that deal is either finished or cancelled
        let mut current_block_num_2 = block_num;
        while !EthClient::deal_over(current_block_num_2, deal.clone()) {
            current_block_num_2 = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        let response_data: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/compute".to_string())
                .await?;

        assert_eq!(response_data.data.success_count, 1);
        assert_eq!(response_data.data.num_windows, 2);
        Ok(())
    }

    #[tokio::test]
    /// This test verifies that we can submit two correct proofs and get two successes.
    async fn two_correct_proofs() -> Result<(), anyhow::Error> {
        let mut file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();

        let deal_proposal: DealProposal = DealProposalBuilder::new(
            "0x0000000000000000000000000000000000000000".to_string(),
            4,
            2,
            0.0,
            0.0,
            "0x0000000000000000000000000000000000000000".to_string(),
        )
        .with_file(file.try_clone()?)
        .build()
        .unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        let deal = eth_client.get_offer(deal_id).await.unwrap();

        let target_window: usize = eth_client
            .compute_target_window(deal.deal_start_block, deal.proof_frequency_in_blocks)
            .await
            .expect("Failed to compute target window");

        let target_block = EthClient::compute_target_block_start(
            deal.deal_start_block,
            deal.proof_frequency_in_blocks,
            target_window,
        );

        dbg!("deal start block: {}", deal.deal_start_block);
        dbg!("target block start: {}", target_block);

        let (_hash, proof) = eth_client
            .create_proof_helper(target_block, &mut file, deal.file_size.as_u64(), true)
            .await
            .expect("Failed to create proof");

        let block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, target_block, None, None)
            .await
            .expect("Failed to post proof");

        let target_block_2 = target_block + deal.proof_frequency_in_blocks;

        let mut current_block_num = block_num;
        while current_block_num != target_block_2 {
            current_block_num = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        let (_hash, bad_proof) = eth_client
            .create_proof_helper(target_block_2, &mut file, deal.file_size.as_u64(), true)
            .await
            .expect("Failed to create proof");

        let block_num_2: BlockNum = eth_client
            .post_proof(deal_id, bad_proof, target_block_2, None, None)
            .await
            .expect("Failed to post proof");

        current_block_num = block_num_2;
        while !EthClient::deal_over(current_block_num, deal.clone()) {
            current_block_num = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        let response_data: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/compute".to_string())
                .await?;

        assert_eq!(response_data.data.success_count, 2);
        assert_eq!(response_data.data.num_windows, 2);
        Ok(())
    }

    #[tokio::test]
    /// This test verifies that we can get a success count of 1 when submitting one correct proof and one incorrect proof
    async fn one_proof_correct_one_incorrect() -> Result<(), anyhow::Error> {
        let mut file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();

        let deal_proposal: DealProposal = DealProposalBuilder::new(
            "0x0000000000000000000000000000000000000000".to_string(),
            4,
            2,
            0.0,
            0.0,
            "0x0000000000000000000000000000000000000000".to_string(),
        )
        .with_file(file.try_clone()?)
        .build()
        .unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        let deal = eth_client.get_offer(deal_id).await.unwrap();

        let target_window: usize = eth_client
            .compute_target_window(deal.deal_start_block, deal.proof_frequency_in_blocks)
            .await
            .expect("Failed to compute target window");

        let target_block = EthClient::compute_target_block_start(
            deal.deal_start_block,
            deal.proof_frequency_in_blocks,
            target_window,
        );

        dbg!("deal start block: {}", deal.deal_start_block);
        dbg!("target block start: {}", target_block);

        let (_hash, proof) = eth_client
            .create_proof_helper(target_block, &mut file, deal.file_size.as_u64(), true)
            .await
            .expect("Failed to create proof");

        let block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, target_block, None, None)
            .await
            .expect("Failed to post proof");

        let target_block_2 = target_block + deal.proof_frequency_in_blocks;

        let mut current_block_num = block_num;
        while current_block_num != target_block_2 {
            current_block_num = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }
        let (_hash, bad_proof) = eth_client
            .create_proof_helper(target_block_2, &mut file, deal.file_size.as_u64(), false)
            .await
            .expect("Failed to create proof");

        let block_num_2: BlockNum = eth_client
            .post_proof(deal_id, bad_proof, target_block_2, None, None)
            .await
            .expect("Failed to post proof");

        current_block_num = block_num_2;
        while !EthClient::deal_over(current_block_num, deal.clone()) {
            current_block_num = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        let response_data: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/compute".to_string())
                .await?;

        assert_eq!(response_data.data.success_count, 1);
        assert_eq!(response_data.data.num_windows, 2);
        Ok(())
    }

    #[tokio::test]
    /// This test verifies that an empty proof will not be counted as a success.
    async fn empty_proof_unsuccessful() -> Result<(), anyhow::Error> {
        let file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let deal_proposal: DealProposal = DealProposalBuilder::new(
            "0x0000000000000000000000000000000000000000".to_string(),
            1,
            1,
            0.0,
            0.0,
            "0x0000000000000000000000000000000000000000".to_string(),
        )
        .with_file(file)
        .build()
        .unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        let deal = eth_client.get_offer(deal_id).await.unwrap();

        // create a proof using the same file we used to create the deal
        let proof = Bytes::from(Vec::new());

        let block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, deal.deal_start_block, None, None)
            .await
            .expect("Failed to post proof");

        let mut current_block_num = block_num;
        while !EthClient::deal_over(current_block_num, deal.clone()) {
            current_block_num = eth_client
                .get_latest_block_num()
                .await
                .expect("Failed to get current block");
        }

        let response_data: validate::ChainlinkResponse =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/compute".to_string())
                .await?;

        assert_eq!(response_data.data.success_count, 0);
        Ok(())
    }
}
