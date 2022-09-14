pub mod make_test;
extern crate rocket;
extern crate rust_chainlink_ea_api;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use eyre::Result;

use ethers::{
    abi::Abi,
    contract::{Contract},
    types::{Address,TransactionRequest, Bytes},
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, Http}
};
use dotenv::dotenv;
use std::fs;
use std::{
    fs::{File},
    io::{Read},
};

pub async fn deploy_helper(
    client: SignerMiddleware<Provider<ethers::providers::Http>,LocalWallet>, 
    contract: Contract<ethers::providers::Provider<ethers::providers::Http>>,
    offer_id: u64, 
    deal_start_block: u64,
    deal_length_in_blocks: u64, 
    proof_frequency_in_blocks: u64,
    ipfs_file_cid: String,
    file_size: u64,
    blake3_checksum: String
) -> Result<(), anyhow::Error> {

    println!("running deploy helper");
    let name = "createOfferShallow";
    let args = (offer_id, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, ipfs_file_cid, file_size, blake3_checksum);
    let data = contract.encode(name, args).unwrap();
    let transaction = TransactionRequest::new()
        .to(contract.address())
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    //println!("offer sent");
    let _receipt = pending_tx.confirmations(1).await?;
    //println!("{:?}", receipt);
    Ok(())
}

// proof helper
pub async fn proof_helper(
    client: SignerMiddleware<Provider<ethers::providers::Http>,LocalWallet>, 
    contract: Contract<ethers::providers::Provider<ethers::providers::Http>>,
    file_name: &str,
    offer_id: u64,
    target_window: u64

) -> Result<(), anyhow::Error> {
    println!("running proof helper");
    println!("file_name: {}", file_name);
    let name = "save_proof";

    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");

    println!("proof length: {:?}", file_content.len());

    let args = (Bytes::from(file_content), offer_id, target_window);
    let data = contract.encode(name, args).unwrap();
    let transaction = TransactionRequest::new()
        .to(contract.address())
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    println!("proof sent");
    let _receipt = pending_tx.confirmations(1).await?;
    //println!("{:?}", receipt);
    Ok(())
}

pub fn setup () -> Result<(
    Provider<ethers::providers::Http>,
    SignerMiddleware<Provider<ethers::providers::Http>,LocalWallet>, 
    Contract<ethers::providers::Provider<ethers::providers::Http>>), anyhow::Error>
    {
    dotenv().ok();
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let private_key: String = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set.");
    let provider = Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = std::env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set.").parse::<Address>()?; // old addr
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


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    /*
    let _th = deploy_helper().await?;
    let _ph = proof_helper().await?;
    */
    //let (provider, client, contract) = setup()?;
    //let _th = deploy_helper(client.clone(), contract.clone(), 2u64, 1u64, 2u64, 3u64, 4u64, "praying".to_string()).await?;
    //let file_name = "/Users/jonahkaye/Desktop/Banyan/Rust-Chainlink-EA-API/proofs/ethereum_proof_Good.txt";
    //let _ph = proof_helper(client, contract, file_name, 2u64, 1u64).await?;
    Ok(())
}

//testing

mod tests {
    use super::*;
    use std::{thread, time};
    use rocket::serde::json::json;
    use bao;

    #[tokio::test]
    async fn verify_proofs() -> Result<(), anyhow::Error> 
    {
        let (provider, client, contract) = setup().unwrap();
        let offer_id = 1; // Not normally hard coded
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
        let target_window: u64 = latest_block - diff;
        println!("target_block: {:} ", target_window);
        let deal_start_block: u64 = target_window - proof_frequency_in_blocks;
        println!("deal_start_block: {:}" , deal_start_block);
        assert!(target_window >= deal_start_block, "target_window must be greater than deal_start_block");
        let offset: u64 = target_window - deal_start_block;
        assert!(offset < deal_length_in_blocks);
        assert!(offset % proof_frequency_in_blocks == 0); 
        let window_num = offset/proof_frequency_in_blocks;
        println!("window_num: {:}", window_num);

        //assert!(latest_block > target_window, "latestBlock.number must be greater than target_window");
        //assert!(latest_block <= target_window + proof_frequency_in_blocks, "latestBlock.number must be less than target_window + proof_frequency_in_blocks");
        

        let file_name = "../Rust-Chainlink-EA-API/files/ethereum.pdf";
        let target_dir = "../Rust-Chainlink-EA-API/proofs/";
        let (hash, file_length): (bao::Hash, u64) = make_test::create_good_proof(banyan_shared::types::BlockNum(target_window), file_name, target_dir).await?;


     
        // I need my file size and checksum to be the same as the file I'm uploading
        let _dh = deploy_helper(client.clone(), contract.clone(), offer_id, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, ipfs_file_cid.to_string(), file_length, hash.to_string()).await?;

        let good_file_1 = format!(
            "../Rust-Chainlink-EA-API/proofs/ethereum_proof_Good_{}.txt",
            target_window.to_string()
        );


        // need to create the proof with the correct target block. Right now this is not correct, since it is using the target window not block.  
        let _ph: () = proof_helper(client.clone(), contract.clone(), &good_file_1, offer_id, window_num).await?; 

        // Wait five blocks so the proof is logged in the next window 
        //let five_blocks = time::Duration::from_millis(80000);
        //thread::sleep(five_blocks);
        println!("bug here 1");
        let target_window_2 = target_window + proof_frequency_in_blocks;
        println!("bug here 2");
        let (_hash, _file_length): (bao::Hash, u64) = make_test::create_good_proof(banyan_shared::types::BlockNum(target_window_2), file_name, target_dir).await?;
        println!("bug here 3");
        let good_file_2 = format!(
            "../Rust-Chainlink-EA-API/proofs/ethereum_proof_Good_{}.txt",
            target_window_2.to_string()
        );
        println!("bug here 4");
        let _ph2: () = proof_helper(client.clone(), contract.clone(), &good_file_2, offer_id, window_num + 1).await?;
        println!("bug here 5");
        // verify proofs
        let map = json!({
            "job_run_id": "613",
            "data":
            {
                 "deal_id": offer_id
            }
        });
        println!("bug here 6");
        let client = reqwest::Client::new();
        let res = client.post("http://localhost:8000/validate")
            .json(&map)
            .send()
            .await?;
        println!("Result: {:?}", res.text().await?);
        
        Ok(())
    }
}