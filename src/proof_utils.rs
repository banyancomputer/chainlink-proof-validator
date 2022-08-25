use crate::types::*;
use anyhow::Result;
use bao::encode::{Encoder, SliceExtractor};
use ethers::abi::ethereum_types::BigEndianHash;
use ethers::prelude::H256;
use std::fs::File;
use std::io::{Read, Seek, Write};

/// 1024 bytes per bao chunk
const CHUNK_SIZE: u64 = 1024;

fn get_num_chunks(size: u64) -> u64 {
    (size as f32 / CHUNK_SIZE as f32).ceil() as u64
}

/// returns tuple (chunk_offset, chunk_size) for the Nth bao hash that you need to grab :)
pub fn compute_random_block_choice_from_hash(block_hash: H256, file_length: u64) -> (u64, u64) {
    let chunk_number = (block_hash.into_uint() % get_num_chunks(file_length)).as_u64();
    let chunk_offset = chunk_number * CHUNK_SIZE;
    let chunk_size = if chunk_number == get_num_chunks(file_length) - 1 {
        file_length - chunk_offset
    } else {
        CHUNK_SIZE
    };
    (chunk_offset, chunk_size)
}
/* 
/// returns the root hash of the obao as well as the file where it's temporarily stored
pub async fn gen_obao<R: Read>(mut reader: R) -> Result<(bao::Hash, File)> {
    // make a temp file to hold the obao
    let obao_file = tempfile::tempfile()?;
    let mut encoder = Encoder::new_outboard(obao_file);
    loop {
        let mut buf = [0u8; CHUNK_SIZE as usize];
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        encoder.write_all(&buf[..n])?;
    }
    let hash = encoder.finalize()?;
    Ok((hash, encoder.into_inner()))
}
*/
/*pub async fn gen_proof<R: Read + Seek>(
    block_number: BlockNum,
    block_hash: H256,
    file_handle: R,
    obao_handle: R,
    file_length: u64,
) -> Result<Proof> {
    let (chunk_offset, chunk_size) = compute_random_block_choice_from_hash(block_hash, file_length);

    let mut bao_proof_data = vec![];
    let _ = SliceExtractor::new_outboard(file_handle, obao_handle, chunk_offset, chunk_size)
        .read_to_end(&mut bao_proof_data)?;

    Ok(Proof {
        block_number,
        bao_proof_data,
    })
}*/