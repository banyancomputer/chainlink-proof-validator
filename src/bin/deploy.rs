use rand::Rng;
use rust_chainlink_ea_api::validate::MyResult;

use dotenv::dotenv;
use ethers::{
    abi::Abi,
    contract::Contract,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, Bytes, TransactionRequest},
};
use serde_json::json;
use std::fs;
use std::{fs::File, io::Read};

use banyan_shared::types::DealID;


pub async fn deploy_helper(
    client: SignerMiddleware<Provider<ethers::providers::Http>, LocalWallet>,
    contract: Contract<ethers::providers::Provider<ethers::providers::Http>>,
    offer_id: u64,
    deal_start_block: u64,
    deal_length_in_blocks: u64,
    proof_frequency_in_blocks: u64,
    ipfs_file_cid: String,
    file_size: u64,
    blake3_checksum: String,
) -> Result<(), anyhow::Error> {
    println!("running deploy helper");
    let name = "createOfferShallow";
    let args = (
        offer_id,
        deal_start_block,
        deal_length_in_blocks,
        proof_frequency_in_blocks,
        ipfs_file_cid,
        file_size,
        blake3_checksum,
    );
    let data = contract.encode(name, args).unwrap();
    let transaction = TransactionRequest::new()
        .to(contract.address())
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    println!("offer sent");
    let _receipt = pending_tx.confirmations(1).await?;
    //println!("{:?}", receipt);
    println!("offer made successfully");
    Ok(())
}

// proof helper
pub async fn proof_helper(
    client: SignerMiddleware<Provider<ethers::providers::Http>, LocalWallet>,
    contract: Contract<Provider<Http>>,
    file_name: &str,
    offer_id: u64,
    target_block: u64,
) -> Result<(), anyhow::Error> {
    let name = "save_proof";
    println!("running proof helper");
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");

    // println!("proof length: {:?}", file_content.len());

    let args = (Bytes::from(file_content), offer_id, target_block);
    let data = contract.encode(name, args).unwrap();
    let transaction = TransactionRequest::new()
        .to(contract.address())
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    println!("Proof for {:?} sent successfully", file_name);
    let _receipt = pending_tx.confirmations(1).await?;
    //println!("{:?}", receipt);
    Ok(())
}

pub fn setup() -> Result<
    (
        Provider<ethers::providers::Http>,
        SignerMiddleware<Provider<ethers::providers::Http>, LocalWallet>,
        Contract<ethers::providers::Provider<ethers::providers::Http>>,
    ),
    anyhow::Error,
> {
    dotenv().ok();
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let private_key: String = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set.");
    let provider =
        Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = std::env::var("CONTRACT_ADDRESS")
        .expect("CONTRACT_ADDRESS must be set.")
        .parse::<Address>()?; // old addr
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("test_contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;
    let wallet = wallet.with_chain_id(5u64);
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let contract = Contract::new(address, abi, provider.clone());
    Ok((provider, client, contract))
}

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
    println!("{:?}", res);
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
    use banyan_shared::{eth::{EthClient}, types::{DealID, BlockNum, DealProposal}};
    use ethers::prelude::k256::sha2::digest::block_buffer::Block;
    use super::*;

    #[tokio::test]
    async fn api_call_test() -> Result<(), anyhow::Error>
    {
        let success_count: u64 = rust_chainlink_ea_api_call(DealID(1), "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 1);
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

        let deal = eth_client.get_deal(deal_id).await.unwrap();
        // Assert that the deal we read is the same as the one we sent
        assert_eq!(deal.deal_length_in_blocks, BlockNum(10));

        let success_count: u64 =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 0);
        Ok(())
    }

    #[tokio::test]
    async fn api_one_proofs_test() -> Result<(), anyhow::Error> {

        let file = File::open("../Rust-Chainlink-EA-API/test_files/ethereum.pdf").unwrap();
        let deal_proposal = DealProposal::builder().build(&file).unwrap();
        let eth_client = EthClient::default();

        let deal_id: DealID = eth_client
            .propose_deal(deal_proposal, None, None)
            .await
            .expect("Failed to send deal proposal");

        // TO DO make proof
        let block_num: BlockNum = eth_client
            .post_proof(deal_id, proof)
            .await
            .expect("Failed to post proof");
    
        let success_count: u64 =
            rust_chainlink_ea_api_call(deal_id, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 0);
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
