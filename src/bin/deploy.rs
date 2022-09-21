#![deny(unused_crate_dependencies)]

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
use rust_chainlink_ea_api::validate;
//use validate::get_deal_info;
use dotenv::dotenv;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::TransactionRequest;
use ethers::{
    abi::Abi,
    contract::{BaseContract, Contract},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, H256, U256},
};
use std::fs;

pub async fn deploy_helper() -> Result<(), anyhow::Error> {
    println!("running deploy helper");
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let provider =
        Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = "0xeb3d5882faC966079dcdB909dE9769160a0a00Ac".parse::<Address>()?; // old addr
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    let contract = Contract::new(address, abi, provider);
    let deal_id = DealID(55378008);
    let deal_start_block = BlockNum(2);
    let deal_length_in_blocks = BlockNum(3);
    let proof_frequency_in_blocks = BlockNum(4);
    let price = TokenAmount(5);
    let collateral = TokenAmount(6);
    let input: [u8; 20] = [
        0xf6, 0x79, 0xd8, 0xd8, 0xa9, 0x0f, 0x66, 0xb4, 0xd8, 0xd9, 0xbf, 0x4f, 0x26, 0x97, 0xd5,
        0x32, 0x79, 0xf4, 0x2b, 0xea,
    ];
    let erc20_token_denomination = Token(ethers::types::H160(input));
    //let erc20_token_denomination = "hello".to_string();

    //let cid_return = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE";
    //let coda = "z".to_owned();
    //let full_cid = format!("{}{}", coda, cid_return);
    //let (_, decoded) = decode(full_cid)?;
    //let reader = Cursor::new(decoded);
    //let ipfs_file_cid = cid::CidGeneric::new_v0(Multihash::read(reader)?)?;
    let ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE".to_string();

    let file_size = 941366;
    let blake3_checksum =
        "c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f".to_string();

    /*let deal: OnChainDealInfo = OnChainDealInfo {
        deal_id: deal_id,
        deal_start_block: deal_start_block,
        deal_length_in_blocks: deal_length_in_blocks,
        proof_frequency_in_blocks: proof_frequency_in_blocks,
        price: price,
        collateral: collateral,
        erc20_token_denomination: erc20_token_denomination,
        ipfs_file_cid,
        file_size,
        blake3_checksum,
    };

    println!("here2");
    let deal2: OnChainDealInfo = OnChainDealInfo {
        deal_id: deal_id,
        deal_start_block,
        deal_length_in_blocks,
        proof_frequency_in_blocks,
        price,
        collateral,
        erc20_token_denomination,
        ipfs_file_cid,
        file_size,
        blake3_checksum,
    };
    let serialized = json::to_string(&deal2)?;
    let deal = json::from_str(&serialized)?;
    //let deal: OnChainDealInfo = json::from_value(deal2)?;
    println!("sdsdgjlg");
    let call = contract.method::<_, H256>("createOffer", deal)?;
    println!("here3");
    let pending_tx = call.send().await?;
    println!("here4");
    let receipt = pending_tx.confirmations(6).await?;
    println!("{:?}", receipt);*/
    Ok(())
}

pub async fn deploy_helper_2() -> Result<(), anyhow::Error> {
    println!("running deploy helper 2");
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let provider =
        Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = "0xEc8AcFb22Ff663Df44C14c71c306E0fF31470d35".parse::<Address>()?; // old addr
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("test_contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    let contract = Contract::new(address, abi, provider);

    println!("here2");
    let call = contract.method::<_, H256>("setDealNothing", ())?;
    println!("here3");
    let pending_tx = call.send().await?;
    println!("here4");
    let receipt = pending_tx.confirmations(6).await?;
    println!("{:?}", receipt);
    Ok(())
}

pub async fn deploy_helper_3() -> Result<(), anyhow::Error> {
    println!("running deploy helper 3");
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let private_key: String = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set.");

    let provider =
        Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = "0xb3d3A786F84094a712eba1D2873CF810156a8338".parse::<Address>()?; // old addr
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

    let name = "setDeal";
    let args = "blakeeeeeeeee".to_string();
    let data = contract.encode(name, args).unwrap();

    let sender: Address = "0x8A4E8e012a5B9EC7817a7936e41DcD84489CE5ed".parse::<Address>()?;

    let mut transaction = TransactionRequest::new()
        .to(address)
        .from(sender)
        .data(data)
        .gas(10000000)
        .chain_id(5);

    //let mut typed = TypedTransaction::Legacy(transaction.clone());
    //let transaction: TransactionRequest = Middleware::fill_transaction(&client, &mut transaction, None);
    /*
    let typed = TypedTransaction::Legacy(transaction.clone());
    println!("here0");
    let gas = Middleware::estimate_gas(&client, &typed).await?;
    println!("gas: {}", gas);
    println!("here1");
    let transaction = transaction.gas(gas);
    */

    /*
    let signed_tx = client.sign_transaction(&TypedTransaction::Legacy(transaction), sender).await?;
    The send transaction automatically signs since its called bu the client which is built with the wallet.
    */

    let pending_tx = client.send_transaction(transaction, None).await?;

    println!("here2");
    // let call = contract.method::<_, H256>("setDealNothing", ())?;
    println!("here3");
    //let pending_tx = call.send().await?;
    println!("here4");
    let receipt = pending_tx.confirmations(6).await?;
    println!("{:?}", receipt);

    let deal_id = 0;
    let checksum = contract
        .method::<_, U256>("getDeal", deal_id)?
        .call()
        .await?
        .as_u64();

    println!("checksum: {}", checksum);
    Ok(())
}

// proof helper
pub async fn proof_helper() -> Result<(), anyhow::Error> {
    println!("running proof helper");
    let api_key: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    let provider =
        Provider::<Http>::try_from(api_key).expect("could not instantiate HTTP Provider");
    let address = "0xeb3d5882faC966079dcdB909dE9769160a0a00Ac".parse::<Address>()?; // old addr
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    println!("a");
    let contract = Contract::new(address, abi, provider);
    let deal_id: u64 = 0;
    let window_num: u64 = 0;

    // read local bao_slice_bad.txt
    //let mut file = std::fs::read("hardhat_test/bao_slice_bad.txt")?;
    // make file a random vector of bytes

    let file = vec![0; 1000];
    println!("b");
    let call = contract.method::<_, H256>("save_proof", (file, deal_id, window_num))?;
    println!("c");
    let pending_tx = call.send().await?;
    let receipt = pending_tx.confirmations(6).await?;
    println!("{:?}", receipt);
    Ok(())
}

// public function to modify ethereum state variable in simple contract
/*
pub async fn test_helper() -> Result<(), anyhow::Error> {

    let provider =
        Provider::<Http>::try_from("https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe")
            .expect("could not instantiate HTTP Provider");
    let address = "0xcFEe8e6Ff3a922aC8Ab811E0a550D1939398834d".parse::<Address>()?; // old addr
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("test_contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    let wallet: Wallet<> = "0x8A4E8e012a5B9EC7817a7936e41DcD84489CE5ed".parse()?;
    let client = Client::new(provider, wallet);
    let contract = Contract::new(address, abi, client);


    let deal_id = 0;
    let checksum = "c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f";
    let call = contract.method::<_, H256>("setDeal", checksum.to_string())?;
    let pending_tx = call.send().await?;
    let receipt = pending_tx.confirmations(6).await?;
    println!("{:?}", receipt);
    Ok(())
}
*/
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    //let block = get_block(0, 0).await?;
    //println!("no chance {:}", block);
    //let _dh = deploy_helper().await?;
    // let _ph = proof_helper().await?;
    //let _th = test_helper().await?;
    let _th = deploy_helper_3().await?;
    Ok(())
}
