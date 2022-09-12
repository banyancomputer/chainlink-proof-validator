extern crate rocket;
extern crate rust_chainlink_ea_api;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use eyre::Result;

use ethers::{
    abi::Abi,
    contract::{Contract},
    types::{Address, U256,TransactionRequest, Bytes, H160},
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, Http}
};
use dotenv::dotenv;
use std::fs;
use std::env;
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
    file_size: u64,
    blake3_checksum: String
) -> Result<(), anyhow::Error> {

    println!("running deploy helper");
    let name = "createOfferShallow";
    let args = (offer_id, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, file_size, blake3_checksum);
    let data = contract.encode(name, args).unwrap();
    let transaction = TransactionRequest::new()
        .to(contract.address())
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    println!("offer sent");
    let receipt = pending_tx.confirmations(1).await?;
    println!("{:?}", receipt);
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
    let dir = env::current_dir()?;
    println!("{}", dir.display());
    let name = "save_proof";
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");

    let args = (Bytes::from(file_content), offer_id, target_window);
    let data = contract.encode(name, args).unwrap();
    let transaction = TransactionRequest::new()
        .to(contract.address())
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    println!("proof sent");
    let receipt = pending_tx.confirmations(1).await?;
    println!("{:?}", receipt);
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
    let (provider, client, contract) = setup()?;
    let _th = deploy_helper(client.clone(), contract.clone(), 2u64, 1u64, 2u64, 3u64, 4u64, "praying".to_string()).await?;

    let file_name = "/Users/jonahkaye/Desktop/Banyan/Rust-Chainlink-EA-API/proofs/ethereum_proof_Good.txt";
    let _ph = proof_helper(client, contract, file_name, 2u64, 1u64).await?;
    Ok(())
}

//testing

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn verify_proofs() -> Result<(), anyhow::Error> 
    {
        let (provider, client, contract) = setup().unwrap();
        let offer_id = 1;
        let deal_length_in_blocks: u64 = 10; 
        let proof_frequency_in_blocks: u64 = 5; 

        // Checks for target window validity simulating smart contract 
        let latest_block: u64 = provider.get_block_number().await?.as_u64();
        println!("latest block: {:} ", latest_block);
        let mut diff: u64 = latest_block % proof_frequency_in_blocks;
        if diff == 0
        {
            diff = proof_frequency_in_blocks;
        }
        let target_window: u64 = latest_block - diff;
        println!("target_block: {:} " ,target_window);
        let deal_start_block: u64 = target_window - proof_frequency_in_blocks;
        println!("deal_start_block: {:}" , deal_start_block);
        assert!(target_window >= deal_start_block, "target_window must be greater than deal_start_block");
        let offset: u64 = target_window - deal_start_block;
        assert!(offset < deal_length_in_blocks);
        assert!(offset % proof_frequency_in_blocks == 0); 
        let window_num = (offset/proof_frequency_in_blocks) - 1;
        println!("window_num: {:}", window_num);
        assert!(latest_block > target_window, "latestBlock.number must be greater than target_window");
        assert!(latest_block <= target_window + proof_frequency_in_blocks, "latestBlock.number must be less than target_window + proof_frequency_in_blocks");
        
        // I need my file size and checksum to be the same as the file I'm uploading

        let dh = deploy_helper(client.clone(), contract.clone(), offer_id, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, 4u64, "praying".to_string()).await?;
        let file_name = "/Users/jonahkaye/Desktop/Banyan/Rust-Chainlink-EA-API/proofs/ethereum_proof_Good.txt";

        let _ph = proof_helper(client, contract, file_name, offer_id, window_num).await?;
        assert_eq!(1,1);
        Ok(())
    }
}
