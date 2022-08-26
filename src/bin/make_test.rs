use std::fs::File;
use std::io::Read;
use ethers::{types::H256,
             providers::{Middleware, Provider, Http}};
use anyhow;

extern crate rust_chainlink_ea_api;
use rust_chainlink_ea_api::types::BlockNum;

pub async fn compute_target_block_hash(target_window_start: BlockNum) -> Result<H256, anyhow::Error> {
    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe")
        .expect("could not instantiate HTTP Provider");

    let target_block_hash = match provider.get_block(target_window_start.0).await? {
        Some(h) => h.hash.unwrap(),
        None => return Err(anyhow::anyhow!("Could not get block hash from number"))
    };
    Ok(target_block_hash)
}

pub fn file_len(file_name: &str) -> usize {
    let mut file_content = Vec::new();
    let dir = "files/";
    let full_path = format!("{}{}", dir, file_name);
    let mut file = File::open(&full_path).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    let length = file_content.len();
    return length;
}



fn main() {
    let file_name = "ethereum.pdf";
    let mut file_content = Vec::new();
    let mut file = File::open(&file_name).expect("Unable to open file");
    file.read_to_end(&mut file_content).expect("Unable to read");
    let length = file_content.len();
    println!("file length: {}", length);
}
