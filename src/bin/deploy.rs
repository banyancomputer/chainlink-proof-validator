use eyre::Result;
use rand::Rng;
use rocket::serde::json::json;
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
use std::fs;
use std::{fs::File, io::Read};

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
    contract: Contract<ethers::providers::Provider<ethers::providers::Http>>,
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

pub async fn api_call(offer_id: u64, api_url: String) -> Result<u64, anyhow::Error> {
    // Job id when chainlink calls is not random.
    let mut rng = rand::thread_rng();
    let random_job_id: u16 = rng.gen();
    let map = json!({
        "job_run_id": random_job_id.to_string(),
        "data":
        {
             "deal_id": offer_id
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
    return Ok(res.data.success_count);
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
#[cfg(test)]
mod tests {

    use anyhow::{anyhow, Error};
    use banyan_shared::{eth::VitalikProvider, proofs, types::*};
    use eyre::Result;
    use std::{
        fs::{read_dir, File},
        io::{Cursor, Read, Seek, Write},
    };

    #[derive(Eq, PartialEq, Debug)]
    pub enum Quality {
        Good,
        Bad,
    }

    // Implement integration_testing_logic here, just without a time delay. Intend to separate the two proofs by
    // the size of the window. In the integration testing, do the same calculation for the first target_window,
    // create the deal, log the first proof, add the size of the window, and wait until the current_window is fast
    // forwarded by size of the window, and log the second proof.
    //
    // Concurrently, calculate the two target windows, and then the folders will be created with the files by rust.
    // Ethereum blocks change every 12 seconds so this should world fine. Maybe put the two commands in a bash script.

    /* Reads a local text file and finds the length of the file */
    pub fn file_len(file_name: &str) -> usize {
        let mut file_content = Vec::new();
        let mut file = File::open(&file_name).expect("Unable to open file");
        file.read_to_end(&mut file_content).expect("Unable to read");
        let length = file_content.len();
        length
    }

    pub async fn create_proof_helper(
        eth_client: &VitalikProvider,
        target_window_start: BlockNum,
        file: &str,
        quality: Quality,
        target_dir: &str,
    ) -> Result<(bao::Hash, u64), Error> {
        std::fs::create_dir_all("proofs/")?;

        let target_block_hash = eth_client
            .get_block_hash_from_num(target_window_start)
            .await?;

        // file stuff
        let split = file.split('.').collect::<Vec<&str>>();
        let input_file_path = split[2];
        let input_file_name = input_file_path.split('/').next_back().unwrap();

        let file_length = file_len(file) as u64;
        let (chunk_offset, chunk_size) =
            proofs::compute_random_block_choice_from_hash(target_block_hash, file_length);
        let mut f = File::open(file)?;
        let (obao_file, hash) = proofs::gen_obao(&f)?;
        f.rewind()?;
        let cursor = Cursor::new(obao_file);
        let mut extractor =
            bao::encode::SliceExtractor::new_outboard(f, cursor, chunk_offset, chunk_size);
        let mut slice = Vec::new();
        extractor.read_to_end(&mut slice)?;

        if quality == Quality::Bad {
            let last_index = slice.len() - 1;
            slice[last_index] ^= 1;
        }

        // TODO pass this through memory.....
        let mut proof_file = File::create(format!(
            "{}{}_proof_{:?}_{}.txt",
            target_dir,
            input_file_name,
            quality,
            target_window_start.0.to_string()
        ))?;
        proof_file.write_all(&slice)?;
        Ok((hash, file_length))
    }

    pub async fn create_good_proofs(
        target_window_starts: &[BlockNum],
        input_dir: &str,
        target_dir: &str,
    ) -> Result<Vec<(bao::Hash, u64)>, Error> {
        let mut result: Vec<(bao::Hash, u64)> = Vec::new();
        let paths = read_dir(input_dir)?;

        // make a vitalikprovider
        let eth_client = VitalikProvider::new();
        for (target_window_start, file) in target_window_starts.iter().zip(paths) {
            let file = file?.path();
            let file: &str = match file.to_str() {
                Some(f) => f,
                None => return Err(anyhow!("Could not convert file name {:?} to string.", file)),
            };
            result.push(
                create_proof_helper(
                    eth_client,
                    *target_window_start,
                    file,
                    Quality::Good,
                    target_dir,
                )
                .await?,
            );
            result.push(
                create_proof_helper(
                    eth_client,
                    *target_window_start,
                    file,
                    Quality::Good,
                    target_dir,
                )
                .await?,
            );
        }
        Ok(result)
    }

    // TODO add tests to check that good proof is good and bad proof is bad
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_file_len() {
            let eth_len = file_len("ethereum.pdf");
            let filecoin_len = file_len("filecoin.pdf");
            assert_eq!(eth_len, 941366);
            assert_eq!(filecoin_len, 629050);
            //assert_eq!(File::open("files/ethereum.pdf")?.metadata().unwrap().len(), 941366);
        }
    }

    //use super::*;
    //use std::{thread, time};
    //use bao;
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
    async fn api_call_test() -> Result<(), anyhow::Error>
    {
        let success_count: u64 = api_call(1, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 1);
        Ok(())
    }
    */
    #[tokio::test]
    async fn api_no_proofs_test() -> Result<(), anyhow::Error> {
        let (provider, client, contract) = setup().unwrap();
        let offer_id = 67;
        let deal_length_in_blocks: u64 = 10;
        let proof_frequency_in_blocks: u64 = 5;
        let ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE";
        let latest_block: u64 = provider.get_block_number().await?.as_u64();
        let mut diff: u64 = latest_block % proof_frequency_in_blocks;
        if diff == 0 {
            diff = proof_frequency_in_blocks;
        }
        let mut target_block: u64 = latest_block - diff;
        let deal_start_block: u64 = target_block - proof_frequency_in_blocks;
        assert!(
            target_block >= deal_start_block,
            "target_block must be greater than deal_start_block"
        );
        let offset: u64 = target_block - deal_start_block;
        assert!(offset < deal_length_in_blocks);
        assert!(offset % proof_frequency_in_blocks == 0);
        let window_num = offset / proof_frequency_in_blocks;
        println!("window_num: {:}", window_num);
        let file_name = "../Rust-Chainlink-EA-API/files/ethereum.pdf";
        let target_dir = "../Rust-Chainlink-EA-API/proofs/";
        let target_block = target_block; //(proof_frequency_in_blocks * i);
        let (hash, file_length): (bao::Hash, u64) = make_test::create_good_proof(
            banyan_shared::types::BlockNum(target_block),
            file_name,
            target_dir,
        )
        .await?;
        let _dh = deploy_helper(
            client.clone(),
            contract.clone(),
            offer_id,
            deal_start_block,
            deal_length_in_blocks,
            proof_frequency_in_blocks,
            ipfs_file_cid.to_string(),
            file_length,
            hash.to_string(),
        )
        .await?;

        let success_count: u64 =
            api_call(offer_id, "http://127.0.0.1:8000/validate".to_string()).await?;
        assert_eq!(success_count, 0);
        Ok(())
    }
}
