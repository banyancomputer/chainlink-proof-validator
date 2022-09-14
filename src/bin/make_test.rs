use anyhow::{anyhow, Error};
use banyan_shared::{eth::VitalikProvider, proofs, types::*};
use dotenv::dotenv;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::H256,
};
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

/* Computes the target block hash from the target block number */
pub async fn compute_target_block_hash(target_window_start: BlockNum) -> Result<H256, Error> {
    let api_token = std::env::var("API_KEY").expect("API_KEY must be set.");
    let provider =
        Provider::<Http>::try_from(api_token).expect("could not instantiate HTTP Provider");

    let target_block_hash = match provider.get_block(target_window_start.0).await? {
        Some(h) => h.hash.unwrap(),
        None => return Err(anyhow::anyhow!("Could not get block hash from number")),
    };
    Ok(target_block_hash)
}

/* Reads a local text file and finds the length of the file */
pub fn file_len(file_name: &str) -> usize {
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    let length = file_content.len();
    length
}

pub async fn create_proof_helper(
    target_window_start: BlockNum,
    file: &str,
    quality: Quality,
    target_dir: &str,
) -> Result<(bao::Hash, u64), Error> {
    std::fs::create_dir_all("proofs/")?;

    let target_block_hash = compute_target_block_hash(target_window_start).await?;

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
    let mut proof_file = File::create(format!(
        "{}{}_proof_{:?}_{}.txt",
        target_dir, input_file_name, quality, target_window_start.0.to_string()
    ))?;
    proof_file.write_all(&slice)?;
    Ok((hash, file_length))
}

pub async fn create_good_proof(
    target_window_start: BlockNum,
    file: &str,
    target_dir: &str,
) -> Result<(bao::Hash, u64), Error> {
    create_proof_helper(target_window_start, file, Quality::Good, target_dir).await
}

pub async fn create_bad_proof(
    target_window_start: BlockNum,
    file: &str,
    target_dir: &str,
) -> Result<(bao::Hash, u64), Error> {
    create_proof_helper(target_window_start, file, Quality::Bad, target_dir).await
}

pub async fn create_good_proofs(
    target_window_starts: &[BlockNum],
    input_dir: &str,
    target_dir: &str,
) -> Result<Vec<(bao::Hash, u64)>, Error> {
    let mut result: Vec<(bao::Hash, u64)> = Vec::new();
    let paths = read_dir(input_dir)?;
    for (target_window_start, file) in target_window_starts.iter().zip(paths) {
        let file = file?.path();
        let file: &str = match file.to_str() {
            Some(f) => f,
            None => return Err(anyhow!("Could not convert file name {:?} to string.", file)),
        };
        result.push(create_good_proof(*target_window_start, file, target_dir).await?);
        result.push(create_good_proof(*target_window_start, file, target_dir).await?);
    }
    Ok(result)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let api_url = std::env::var("URL").expect("URL must be set.");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set.");
    let url = format!("{}{}", api_url, api_key);
    let contract_address =
        std::env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set.");
    let provider = VitalikProvider::new(url, contract_address)?;
    let info = provider.get_onchain(DealID(55378008)).await?;
    println!("{:?}", info);

    // Implement integration_testing_logic here, just without a time delay. Intend to separate the two proofs by
    // the size of the window. In the integration testing, do the same calculation for the first target_window,
    // create the deal, log the first proof, add the size of the window, and wait until the current_window is fast
    // forwarded by size of the window, and log the second proof.
    //
    // Concurrently, calculate the two target windows, and then the folders will be created with the files by rust.
    // Ethereum blocks change every 12 seconds so this should world fine. Maybe put the two commands in a bash script.

    /*let target_window_starts = [BlockNum(1), BlockNum(2)];
    let input_dir = "../Rust-Chainlink-EA-API/files/";
    let target_dir = "../Rust-Chainlink-EA-API/proofs/";
    create_proofs(&target_window_starts, input_dir, target_dir).await?;*/
    Ok(())
}
// add tests to check that good proof is good and bad proof is bad
#[cfg(test)]
mod tests {
    use super::*;
    /*
    #[test]

    fn test_file_len() {
        let eth_len = file_len("ethereum.pdf");
        let filecoin_len = file_len("filecoin.pdf");
        assert_eq!(eth_len, 941366);
        assert_eq!(filecoin_len, 629050);
        //assert_eq!(File::open("files/ethereum.pdf")?.metadata().unwrap().len(), 941366);
    }
    */
}
