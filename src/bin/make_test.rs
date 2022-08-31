use core::slice;
use std::{fs::{File, read_dir},
          io::{Read, Write}};
use ethers::{types::H256,
             providers::{Middleware, Provider, Http}};
use anyhow::{Error, anyhow};
use tempfile;

use banyan_shared::{types::*, proofs};

#[derive(PartialEq, Debug)]
pub enum Quality {
    Good,
    Bad
}

/* Computes the target block hash from the target block number */
pub async fn compute_target_block_hash(target_window_start: BlockNum) -> Result<H256, Error> {
    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe")
        .expect("could not instantiate HTTP Provider");

    let target_block_hash = match provider.get_block(target_window_start.0).await? {
        Some(h) => h.hash.unwrap(),
        None => return Err(anyhow::anyhow!("Could not get block hash from number"))
    };
    Ok(target_block_hash)
}

/* Reads a local text file from a directory called "files/" and finds the 
   length of the file */
pub fn file_len(file_name: &str) -> usize {
    let mut file_content = Vec::new();
    let dir = "files/";
    let full_path = format!("{}{}", dir, file_name);
    let mut file = File::open(&full_path).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    let length = file_content.len();
    return length;
}

pub async fn create_proof_helper(target_window_start: BlockNum, file: &str, quality: Quality) -> Result<(bao::Hash, u64), Error> {
    std::fs::create_dir_all("proofs/")?;
    let target_block_hash = compute_target_block_hash(target_window_start).await?;
    let split = file.split(".").collect::<Vec<&str>>();
    let input_file_name = split[0];
    let file_length = file_len(input_file_name) as u64;
    let (chunk_offset, chunk_size) = 
        proofs::compute_random_block_choice_from_hash(target_block_hash, file_length);
    
    let f = File::open(file)?;
    let (hash, bao_file) = proofs::gen_obao(f).await?;
    
    let mut extractor = 
        bao::encode::SliceExtractor::new(bao_file, 
                                        chunk_offset, 
                                        chunk_size);
    let mut slice: Vec<u8> = Vec::new();
    extractor.read_to_end(&mut slice)?;

    if quality == Quality::Bad {
        let last_index = slice.len() - 1;
        slice[last_index] ^= 1;
    }

    let mut proof_file = 
        File::create(format!("proofs/{}_proof_{:?}.txt", 
                     input_file_name, 
                     quality))?;
    proof_file.write_all(&slice)?;

    Ok((hash, file_length))
}

pub async fn create_good_proof(target_window_start: BlockNum, file: &str) -> Result<(bao::Hash, u64), Error> {
    create_proof_helper(target_window_start, file, Quality::Good).await
}

pub async fn create_bad_proof(target_window_start: BlockNum, file: &str) -> Result<(bao::Hash, u64), Error> {
    create_proof_helper(target_window_start, file, Quality::Bad).await
}

pub async fn create_proofs(target_window_starts: &[BlockNum], dir: &str) -> Result<Vec<(bao::Hash, u64)>, Error> {
    let mut result: Vec<(bao::Hash, u64)> = Vec::new();
    let paths = read_dir(dir)?;
    for (target_window_start, file) in target_window_starts.iter().zip(paths) {
        let file = file?.path();
        let file: &str = match file.to_str() {
            Some(f) => f,
            None => return Err(anyhow!("Could not convert file name {:?} to string.", file))
        };
        result.push(create_good_proof(*target_window_start, file).await?);
        result.push(create_bad_proof(*target_window_start, file).await?);
    }
    Ok(result)
}

fn main() {
    let file_name = "files/filecoin.pdf";
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    let length = file_content.len();
    println!("file length: {}", length);
}

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
