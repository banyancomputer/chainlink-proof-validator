// new addr 0xeb3d5882faC966079dcdB909dE9769160a0a00Ac
#[macro_use]
extern crate rocket;
extern crate rust_chainlink_ea_api;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use banyan_shared::{ipfs, types::*};
use eyre::Result;
use rocket::serde::{json, Deserialize, Serialize};

use ethers::{
    abi::Abi,
    contract::{Contract},
    types::{Address, U256,TransactionRequest, Bytes},
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
    providers::{Middleware, Provider, Http}
};
use dotenv::dotenv;
use std::fs;
use std::{
    fs::{read_dir, File},
    io::{Read, Seek, Write},
};
use std::env;

pub async fn deploy_helper() -> Result<(), anyhow::Error> {
    println!("running deploy helper 3");
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let private_key: String = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set.");

    let provider =
        Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = "0xf8bc718Bd3Af1B19f3139078a5A2A2d8589cb797".parse::<Address>()?; // old addr
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("test_contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;
    let wallet = wallet.with_chain_id(5u64);
    let client = SignerMiddleware::new(provider.clone(), wallet);

    //let contract = BaseContract::from(abi);
    let contract = Contract::new(address, abi, provider);
    
    let name = "createOfferShallow";
    //let deal = BigDeal{blake3_checksum: "banyaannn".to_string(), offer_id: 2};
    let args = (2u64, 1u64, 2u64, 3u64, 4u64, "praying".to_string());
    //let args = BigDeal{blake3_checksum: "boooyakasha".to_string(), offer_id: 2u64};
    let data = contract.encode(name, args).unwrap();

    let sender: Address = "0x8A4E8e012a5B9EC7817a7936e41DcD84489CE5ed".parse::<Address>()?;

    let mut transaction = TransactionRequest::new()
        .to(address)
        .from(sender)
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    
    println!("transaction sent");
    let receipt = pending_tx.confirmations(1).await?;
    println!("{:?}", receipt);
    
    let checksum: String = contract
        .method::<_, String>("getBlake3Checksum", 2u64)?
        .call()
        .await?;

    println!("checksum: {:?}", checksum);
    Ok(())
}

// proof helper
pub async fn proof_helper() -> Result<(), anyhow::Error> {
    println!("running deploy helper 3");
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let private_key: String = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set.");

    let provider =
        Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = "0xf8bc718Bd3Af1B19f3139078a5A2A2d8589cb797".parse::<Address>()?; // old addr
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("test_contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?;
    let wallet = wallet.with_chain_id(5u64);
    let client = SignerMiddleware::new(provider.clone(), wallet);

    //let contract = BaseContract::from(abi);
    let contract = Contract::new(address, abi, provider);
    
    let dir = env::current_dir()?;
    println!("{}", dir.display());

    let name = "save_proof";
    let file_name = "/Users/jonahkaye/Desktop/Banyan/Rust-Chainlink-EA-API/proofs/ethereum_proof_Good.txt";
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    let offer_id: u64 = 613;
    let target_window: u64  = 10; 
    let args = (Bytes::from(file_content), offer_id, target_window);
    let data = contract.encode(name, args).unwrap();


    let sender: Address = "0x8A4E8e012a5B9EC7817a7936e41DcD84489CE5ed".parse::<Address>()?;

    let mut transaction = TransactionRequest::new()
        .to(address)
        .from(sender)
        .data(data)
        .gas(10000000)
        .chain_id(5);
    let pending_tx = client.send_transaction(transaction, None).await?;
    
    println!("transaction sent");
    let receipt = pending_tx.confirmations(1).await?;
    println!("{:?}", receipt);
    
    let checksum: String = contract
        .method::<_, String>("getProofBlock", (613u64, 10u64))?
        .call()
        .await?;

    println!("checksum: {:?}", checksum);
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    //let _th = deploy_helper().await?;
    let _ph = proof_helper().await?;
    Ok(())
}