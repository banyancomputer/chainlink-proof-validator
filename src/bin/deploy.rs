use rand::Rng;
use rust_chainlink_ea_api::validate::MyResult;

use dotenv::dotenv;
use serde_json::json;
use banyan_shared::types::DealID;

pub async fn rust_chainlink_ea_api_call(deal_id: DealID, api_url: String) -> Result<u64, anyhow::Error> {
    // Job id when chainlink calls is not random.
    let mut rng = rand::thread_rng();
    let random_job_id: u16 = rng.gen();
    let map = json!({
        "job_run_id": random_job_id.to_string(),
        "data":
        {
             "deal_id": deal_id.0
        }
    });
    let client = reqwest::Client::new();
    let res = client
        .post(api_url)
        .json(&map)
        .send()
        .await?
        .json::<MyResult>()
        .await?;
    dbg!("{:?}", &res);
    Ok(res.data.success_count)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    Ok(())
}

//testing
#[cfg(test)]
mod tests {
    use banyan_shared::{eth::{EthClient}, types::{DealID, BlockNum, DealProposal}, deals::DealProposalBuilder};
    use std::{fs::File, io::{Read,Seek}};
    use super::*;

    #[tokio::test]
    async fn api_call_test() -> Result<(), anyhow::Error>
    {
        let success_count: u64 = rust_chainlink_ea_api_call(DealID(1), "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn api_no_proofs_test() -> Result<(), anyhow::Error> {

        let file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let deal_proposal = DealProposal::builder().build(&file).unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal, None, None)
            .await
            .expect("Failed to send deal proposal");

        dbg!("Proof Created for Deal ID: {:}", &deal_id);
        let deal = eth_client.get_deal(deal_id).await.unwrap();
        // Assert that the deal we read is the same as the one we sent
        assert_eq!(deal.deal_length_in_blocks, BlockNum(10));

        let success_count: u64 =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn multiple_deals_same_file_no_proofs() -> Result<(), anyhow::Error> {

        let file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let deal_proposal = DealProposal::builder().build(&file).unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        dbg!("Proof Created for Deal ID: {:}", &deal_id);
        let deal = eth_client.get_deal(deal_id).await.unwrap();
        // Assert that the deal we read is the same as the one we sent
        assert_eq!(deal.deal_length_in_blocks, BlockNum(10));

        let deal_id_2: DealID = eth_client
            .propose_deal(deal_proposal, None, None)
            .await
            .expect("Failed to send deal proposal");
        dbg!("Proof Created for Deal ID: {:}", &deal_id_2);
        let deal = eth_client.get_deal(deal_id_2).await.unwrap();
        assert_eq!(deal.deal_length_in_blocks, BlockNum(10));

        let success_count: u64 = rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 0);
        let success_count_2: u64 = rust_chainlink_ea_api_call(deal_id_2, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count_2, 0);
        Ok(())
    }

    #[tokio::test]
    async fn post_proof_to_chain() -> Result<(), anyhow::Error> {

        let mut file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let eth_client = EthClient::default();

        let deal_id = DealID(15);
        let deal = eth_client.get_deal(deal_id).await.unwrap();

        let target_window: usize = eth_client
            .compute_target_window(deal.deal_start_block, deal.proof_frequency_in_blocks)
            .await
            .expect("Failed to compute target window");

        let target_block = EthClient::compute_target_block_start(
            deal.deal_start_block,
            deal.proof_frequency_in_blocks,
            target_window,
        );
        // create a proof using the same file we used to create the deal
        let (_hash, proof) = eth_client
            .create_proof_helper(target_block, &mut file, deal.file_size.as_u64(), true)
            .await
            .expect("Failed to create proof");

        let block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, target_block, None, None)
            .await
            .expect("Failed to post proof");
        
        let proof_bytes: Vec<u8> = match eth_client
            .get_proof_from_logs(block_num, deal_id)
            .await?
            {
            Some(proof) => proof,
            None => {
                panic!("Failed to get proof from logs");
            }
        };

        assert_eq!(proof_bytes.len(), 1672);
        Ok(())
        
    }

    #[tokio::test]
    async fn api_one_proof_test() -> Result<(), anyhow::Error> {

        let mut file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let eth_client = EthClient::default();

        let deal_id = DealID(15);
        let deal = eth_client.get_deal(deal_id).await.unwrap();

        // create a proof for an old deal, and put the proof in the first valid window. Not possible in real life, but convenient for testing. 
        let (_hash, proof) = eth_client
            .create_proof_helper(deal.deal_start_block, &mut file, deal.file_size.as_u64(), true)
            .await
            .expect("Failed to create proof");

        let _block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, deal.deal_start_block, None, None)
            .await
            .expect("Failed to post proof");
        
        let success_count: u64 =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/validate".to_string()).await?;

        // The deal is two windows long, and only one proof was submitted. 
        assert_eq!(success_count, 1);
        Ok(())
    } 

    #[tokio::test]
    async fn deal_and_proof_one_window() -> Result<(), anyhow::Error> {

        let mut file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();

        let deal_proposal: DealProposal = DealProposalBuilder::new("0x0000000000000000000000000000000000000000".to_string(), 1, 1, 0.0, 0.0,"0x0000000000000000000000000000000000000000".to_string())
            .build(&file).unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal.clone(), None, None)
            .await
            .expect("Failed to send deal proposal");

        let deal = eth_client.get_deal(deal_id).await.unwrap();

        // create a proof using the same file we used to create the deal
        let (_hash, proof) = eth_client
            .create_proof_helper(deal.deal_start_block, &mut file, deal.file_size.as_u64(), true)
            .await
            .expect("Failed to create proof");
        
        let _block_num: BlockNum = eth_client
            .post_proof(deal_id, proof, deal.deal_start_block, None, None)
            .await
            .expect("Failed to post proof");
        
        let success_count: u64 =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/validate".to_string()).await?;

        assert_eq!(success_count, 1);
        Ok(())
    }
    
    /*
    #[tokio::test]
    async fn one_proof_window_missing () -> Result<(), anyhow::Error>
    {
        let (provider, client, contract) = setup().unwrap();
        let offer_id = 100;
        let deal_length_in_blocks: u64 = 10;
        let proof_frequency_in_blocks: u64 = 5;
        let ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE";
        let latest_block: u64 = provider.get_block_number().await?.as_u64();
        let mut diff: u64 = latest_block % proof_frequency_in_blocks;
        if diff == 0
        {
            diff = proof_frequency_in_blocks;
        }
        let mut target_block: u64 = latest_block - diff;
        let deal_start_block: u64 = target_block - proof_frequency_in_blocks;
        assert!(target_block >= deal_start_block, "target_block must be greater than deal_start_block");
        let offset: u64 = target_block - deal_start_block;
        assert!(offset < deal_length_in_blocks);
        assert!(offset % proof_frequency_in_blocks == 0);
        let window_num = offset/proof_frequency_in_blocks;
        println!("window_num: {:}", window_num);

        let file_name = "../Rust-Chainlink-EA-API/files/ethereum.pdf";
        let target_dir = "../Rust-Chainlink-EA-API/proofs/";

        let target_block = target_block; //(proof_frequency_in_blocks * i);
        let (hash, file_length): (bao::Hash, u64) = make_test::create_good_proof(banyan_shared::types::BlockNum(target_block), file_name, target_dir).await?;

            let _dh = deploy_helper(client.clone(), contract.clone(), offer_id, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, ipfs_file_cid.to_string(), file_length, hash.to_string()).await?;

        let proof_file = format!(
            "../Rust-Chainlink-EA-API/proofs/ethereum_proof_Good_{}.txt",
            target_block.to_string()
        );
        let _ph: () = proof_helper(client.clone(), contract.clone(), &proof_file, offer_id, window_num).await?;

        let success_count  = api_call(offer_id, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 1);
        Ok(())
    }
    */

     /*
    #[tokio::test]
    async fn verify_ten_correct_proofs() -> Result<(), anyhow::Error>
    {
        let (provider, client, contract) = setup().unwrap();
        let offer_id = 10; // Not normally hard coded

        // Conditions set by deal config in contract
        let deal_length_in_blocks: u64 = 10;
        let proof_frequency_in_blocks: u64 = 5;
        let ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE";
        // Checks for target window validity simulating smart contract

        let latest_block: u64 = provider.get_block_number().await?.as_u64();
        //println!("latest block: {:} ", latest_block);
        let mut diff: u64 = latest_block % proof_frequency_in_blocks;
        if diff == 0
        {
            diff = proof_frequency_in_blocks;
        }
        let mut target_block: u64 = latest_block - diff;
        println!("target_block: {:} ", target_block);
        let deal_start_block: u64 = target_block - proof_frequency_in_blocks;
        println!("deal_start_block: {:}" , deal_start_block);
        assert!(target_block >= deal_start_block, "target_block must be greater than deal_start_block");
        let offset: u64 = target_block - deal_start_block;
        assert!(offset < deal_length_in_blocks);
        assert!(offset % proof_frequency_in_blocks == 0);
        let window_num = offset/proof_frequency_in_blocks;
        println!("window_num: {:}", window_num);

        let file_name = "../Rust-Chainlink-EA-API/files/ethereum.pdf";
        let target_dir = "../Rust-Chainlink-EA-API/proofs/";


            println!("here");
            let target_block = target_block + proof_frequency_in_blocks; // (proof_frequency_in_blocks * i);
            println!("here2");
            let (hash, file_length): (bao::Hash, u64) = make_test::create_good_proof(banyan_shared::types::BlockNum(target_block), file_name, target_dir).await?;
            println!("here3");
            let _dh = deploy_helper(client.clone(), contract.clone(), offer_id, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, ipfs_file_cid.to_string(), file_length, hash.to_string()).await?;
            println!("here4");
            let proof_file = format!(
                "../Rust-Chainlink-EA-API/proofs/ethereum_proof_Good_{}.txt",
                target_block.to_string()
            );
            println!("here5");
            // Need to create the proof with the correct target block. Right now this is not correct, since it is using the target window not block.
            let _ph: () = proof_helper(client.clone(), contract.clone(), &proof_file, offer_id, window_num).await?;


        // verify proofs
        let success_count  = api_call(offer_id, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 1);
        Ok(())
    }

    */

}
