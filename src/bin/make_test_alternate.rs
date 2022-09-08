use anyhow::{anyhow, Error};
use ethers::{
    providers::{Http, Middleware, Provider},
    types::H256,
};
use std::{
    fs::{read_dir, File},
    io::{Read, Seek, Write},
};

use banyan_shared::{proofs, types::*};
use eyre::Result;

#[derive(PartialEq, Debug)]
pub enum Quality {
    Good,
    Bad,
}

use std::io::Cursor;

// Implement integration_testing_logic here, just without a time delay. Intend to separate the two proofs by
// the size of the window. In the integration testing, do the same calculation for the first target_window,
// create the deal, log the first proof, add the size of the window, and wait until the current_window is fast
// forwarded by size of the window, and log the second proof.
//
// Concurrently, calculate the two target windows, and then the folders will be created with the files by rust.
// Ethereum blocks change every 12 seconds so this should world fine. Maybe put the two commands in a bash script.

/* Computes the target block hash from the target block number */
pub async fn compute_target_block_hash(target_window_start: BlockNum) -> Result<H256, Error> {
    let provider =
        Provider::<Http>::try_from("https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe")
            .expect("could not instantiate HTTP Provider");

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
    return length;
}

pub fn jonah_obao<R: Read>(mut reader: R) -> Result<(Vec<u8>, bao::Hash)> {
    let mut file_content = Vec::new();
    reader
        .read_to_end(&mut file_content)
        .expect("Unable to read");

    let (obao, hash) = bao::encode::outboard(&file_content);
    Ok((obao, hash)) // return the outboard encoding
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
    let split = file.split(".").collect::<Vec<&str>>();
    let input_file_path = split[0];
    let input_file_name = input_file_path.split('/').next_back().unwrap();
    let file_length = file_len(file) as u64;
    println!("file length: {file_length}");
    let (chunk_offset, chunk_size) =
        proofs::compute_random_block_choice_from_hash(target_block_hash, file_length);
    println!("{chunk_offset}, {chunk_size}");

    let mut f = File::open(file)?;
    let (obao_file, hash) = jonah_obao(&f).unwrap();
    f.rewind()?;
    let mut extractor = bao::encode::SliceExtractor::new_outboard(
        f,
        Cursor::new(&obao_file[..]),
        chunk_offset,
        chunk_size,
    );
    let mut slice = Vec::new();
    extractor.read_to_end(&mut slice)?;

    if quality == Quality::Bad {
        let last_index = slice.len() - 1;
        slice[last_index] ^= 1;
    } else {
        println!("proof length: {:}, chunksize {:}", slice.len(), chunk_size);
        let mut decoded = Vec::new();
        let mut decoder = bao::decode::SliceDecoder::new(
            &*slice,
            &hash,
            chunk_offset.try_into().unwrap(),
            chunk_size.try_into().unwrap(),
        );
        decoder.read_to_end(&mut decoded)?;
        println!("good proof")
    }

    println!("{input_file_name}");
    let new = format!("{}/{}_proof_{:?}.txt", target_dir, input_file_name, quality);
    println!("{new}");
    let mut proof_file = File::create(new)?;
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

pub async fn create_proofs(
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
        println!(
            "Creating proof for file {} with target_window_start {}",
            file, target_window_start.0
        );
        result.push(create_good_proof(*target_window_start, file, target_dir).await?);
        result.push(create_bad_proof(*target_window_start, file, target_dir).await?);
    }
    Ok(result)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Implement integration_testing_logic here, just without a time delay. Intend to separate the two proofs by
    // the size of the window. In the integration testing, do the same calculation for the first target_window,
    // create the deal, log the first proof, add the size of the window, and wait until the current_window is fast
    // forwarded by size of the window, and log the second proof.
    //
    // Concurrently, calculate the two target windows, and then the folders will be created with the files by rust.
    // Ethereum blocks change every 12 seconds so this should world fine. Maybe put the two commands in a bash script.

    let target_window_starts = [BlockNum(1), BlockNum(2)];
    let input_dir = "../Rust-Chainlink-EA-API/files/";
    let target_dir = "../Rust-Chainlink-EA-API/proofs/";
    create_proofs(&target_window_starts, input_dir, target_dir).await?;
    Ok(())
}
// add tests to check that good proof is good and bad proof is bad
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
