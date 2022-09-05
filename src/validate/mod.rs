/*
1. when it gets a request, find the deal_id’s info on chain
2. check that the deal is either FINISHED (current_block_num > deal_start_block
   + deal_length_in_blocks) or CANCELLED (and do the computations below with
   deal_length_in_blocks := (agreed_upon_cancellation_block - deal_start).
3. start iterating over proof_blocks  from window_num \in (0, num_windows),
   num_windows = ceiling(deal_length_in_blocks / window_size)
        a. if there isn’t a proof recorded in proof_blocks under that window, continue
        b. find the proof in that block’s logs, stick it in proof_bytes
        c. if there is, set target_window_start to window_num * window_size + deal_start_block
        d. get the target_block_hash as block_hash(target_window_start)
        e. get the chunk_offset and chunk_size according to the function
           compute_random_block_choice_from_hash(target_block_hash, deal_info.file_length)
           defined in my code here: https://github.com/banyancomputer/ipfs-proof-buddy/blob/9f0ae728f7a103da615c5eedf37491267f470e48/src/proof_utils.rs#L17
           (by the way let’s not copy-paste or reimplement this- let’s make a banyan-shared
            crate when you get to this)
        f. validate the proof, and if you pass, increment success_count
4. then once you get done with iterating over all the proofs, return
   (success_count, num_windows)  and whatever id/deal_id you need in order
   to identify the computation performed back to chain
*/

/* lazy static macro - needs to be thread safe, maybe use to instantiate a provider
or maybe instance methods
open provider once

cargo fmt, then cargo check, then cargo clippy
cargo build - default is debug mode, does overflow checking 
cargo build --release for benchmarking

codecov, cicd
*/
use anyhow;
use cid;
use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Middleware, Provider},
    types::{Address, Filter, H256, U256},
};
use multibase::decode;
use multihash::Multihash;
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::{
    fs,
    io::{Cursor, Read},
    str::FromStr,
};

use banyan_shared::{proofs, types::*};
use dotenv::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    // OPEN PROVIDER (TRY)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainlinkRequest {
    pub job_run_id: String,
    pub data: RequestData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestData {
    pub offer_id: String, // should also be DealId type, use serialize_with
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    // maybe change so status and result are in the MyResult struct instead
    pub offer_id: u64, // number with meaning, wrap in a type (DealId)
    pub success_count: u64,
    pub num_windows: u64,
    pub status: u16,    // should be an enum
    pub result: String, // look into better option
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResult {
    pub data: ResponseData,
}

/*
    Gets the deal info from on chain.
*/
pub async fn get_deal_info(offer_id: u64) -> Result<OnChainDealInfo, anyhow::Error> {
    let deal_id: DealID = DealID(offer_id);
    let api_token = std::env::var("API_KEY").expect("API_KEY must be set.");
    let provider = Provider::<Http>::try_from(api_token.as_str())
        .expect("could not instantiate HTTP Provider");
    let address = "0xeb3d5882faC966079dcdB909dE9769160a0a00Ac".parse::<Address>()?;
    let abi: Abi = serde_json::from_str(
        fs::read_to_string("contract_abi.json")
            .expect("can't read file")
            .as_str(),
    )?;
    let contract = Contract::new(address, abi, provider);

    let deal_start_block: BlockNum = BlockNum(
        contract
            .method::<_, U256>("getDealStartBlock", deal_id.0)?
            .call()
            .await?
            .as_u64(),
    );

    let deal_length_in_blocks: BlockNum = BlockNum(
        contract
            .method::<_, U256>("getDealLengthInBlocks", deal_id.0)?
            .call()
            .await?
            .as_u64(),
    );

    let proof_frequency_in_blocks: BlockNum = BlockNum(
        contract
            .method::<_, U256>("getProofFrequencyInBlocks", deal_id.0)?
            .call()
            .await?
            .as_u64(),
    );

    let price: TokenAmount = TokenAmount(
        contract
            .method::<_, U256>("getPrice", deal_id.0)?
            .call()
            .await?
            .as_u64(),
    );

    let collateral: TokenAmount = TokenAmount(
        contract
            .method::<_, U256>("getCollateral", deal_id.0)?
            .call()
            .await?
            .as_u64(),
    );

    let erc20_token_denomination: Token = Token(
        contract
            .method::<_, Address>("getErc20TokenDenomination", deal_id.0)?
            .call()
            .await?,
    );

    let cid_return: String = contract
        .method::<_, String>("getIpfsFileCid", deal_id.0)?
        .call()
        .await?;
    let code = "z".to_owned();
    let full_cid = format!("{}{}", code, cid_return);
    let (_, decoded) = decode(full_cid)?;
    let reader = Cursor::new(decoded);
    let ipfs_file_cid = cid::CidGeneric::new_v0(Multihash::read(reader)?)?;

    let file_size: u64 = contract
        .method::<_, u64>("getFileSize", deal_id.0)?
        .call()
        .await?;

    let blake3_return: String = contract
        .method::<_, String>("getBlake3Checksum", deal_id.0)?
        .call()
        .await?;
    let blake3_checksum = bao::Hash::from_str(&blake3_return)?;

    let deal_info: OnChainDealInfo = OnChainDealInfo {
        deal_id,
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

    Ok(deal_info)
}

pub fn construct_error(reason: String) -> Json<MyResult> {
    Json(MyResult {
        data: ResponseData {
            offer_id: 0,
            success_count: 0,
            num_windows: 0,
            status: 500,
            result: reason,
        },
    })
}

pub async fn get_block(offer_id: u64, window_num: u64) -> Result<u64, anyhow::Error> {
    let api_token = std::env::var("API_KEY").expect("API_KEY must be set.");
    let provider = Provider::<Http>::try_from(api_token)
            .expect("could not instantiate HTTP Provider");
    let address = 
        "0xeb3d5882faC966079dcdB909dE9769160a0a00Ac".parse::<Address>()?; 
    let abi: Abi = serde_json::from_str(fs::read_to_string("contract_abi.json")
                                            .expect("can't read file")
                                            .as_str())?;

    let contract = Contract::new(address, abi, provider);
    let block: u64 = contract
        .method::<_, U256>("getProofBlock", (offer_id, window_num))?
        .call()
        .await?
        .as_u64();

    return Ok(block);
}

pub async fn validate_deal_internal(input_data: Json<ChainlinkRequest>) -> Result<Json<MyResult>, String> {
    dotenv().ok();
    let api_token = std::env::var("API_KEY").expect("API_KEY must be set.");
    let mut success_count = 0;

    let provider =
        Provider::<Http>::try_from(api_token).expect("could not instantiate HTTP Provider");
    let _address = "0xeb3d5882faC966079dcdB909dE9769160a0a00Ac".parse::<Address>().map_err(|e| format!("Could not parse address: {e}"))?;
    let _abi: Abi = serde_json::from_str(
        fs::read_to_string("contract_abi.json")
            .expect("can't read file")
            .as_str()).map_err(|e| format!("Could not get contract abi: {e}"))?;
    
    // getting deal info from on chain
    let request: ChainlinkRequest = input_data.into_inner();
    // this parsing shoulg be done at the border by serde
    let offer_id = request.data.offer_id.trim().parse::<u64>().unwrap();
    let deal_info = get_deal_info(offer_id).await.map_err(|e| format!("Error in get_deal_info: {:?}", e))?;

    // checking that deal is either finished or cancelled
    let current_block_num = provider.get_block_number().await.map_err(|e| format!("Couldn't get most recent block number: {e}"))?;

    // wrapper over this that just returns the blocknum and does all your casts for you
    // maybe also a wrapper that checks deal_over
    let finished = BlockNum(current_block_num.as_u64())
        > deal_info.deal_start_block + deal_info.deal_length_in_blocks;
    let cancelled = false; // need to figure out how to get this

    if !finished && !cancelled {
        return Ok(Json(MyResult {
            data: ResponseData {
                offer_id: 0,
                success_count: 0,
                num_windows: 0,
                status: 500,
                result: format!("Deal {} is ongoing.", offer_id),
            },
        }));
    }

    let agreed_upon_cancellation_block: BlockNum = BlockNum(0u64); // need to figure out how to get this
    let deal_length_in_blocks = match cancelled {
        false => deal_info.deal_length_in_blocks,
        true => agreed_upon_cancellation_block - deal_info.deal_start_block,
    };
    let window_size = deal_info.proof_frequency_in_blocks.0;
    // this should be in window_uril.rs and tested meticulously (it might eb in the proof_gen.rs thing)
    let num_windows: usize =
        math::round::ceil((deal_length_in_blocks.0 / window_size) as f64, 0) as usize;

    // iterating over proof blocks (by window)
    for window_num in 0..num_windows {
        // step b. above
        let block = get_block(offer_id, window_num as u64).await.map_err(|e| format!("Could not get block: {e}"))?;

        let filter: Filter = Filter::new()
            .select(block)
            .topic1(H256::from_low_u64_be(offer_id));
        let block_logs = provider.get_logs(&filter).await.map_err(|e| format!("Couldn't get logs from block {current_block_num}: {e}"))?;

        let proof_bytes = Cursor::new(&block_logs[0].data);

        // step c. above
        // impl Mul(u: usize) for BlockNum {} in types
        let target_window_start: BlockNum =
            BlockNum(window_num as u64 * window_size + deal_info.deal_start_block.0);

        // step d. above
        // write provider.get_block(block_num: BlockNum)
        let target_block_hash = provider.get_block(target_window_start.0).await.map_err(|e| format!("Could not get block number {block}: {e}"))?.ok_or(format!("Could not unpack block number {block}"))?.hash.ok_or(format!("Could not get hash of block {block}"))?;

        // step e. above
        println!("target_block_hash: {:?}", target_block_hash);
        println!("file_size {:?}", deal_info.file_size);
        
        let (chunk_offset, chunk_size) = 
            proofs::compute_random_block_choice_from_hash(target_block_hash, deal_info.file_size);

        // step f. above
        println!("checksum: {:?}", deal_info.blake3_checksum);
        println!("chunk_offset: {:?}", chunk_offset);
        println!("chunk_size: {:?}", chunk_size);

        let mut decoded = Vec::new();
        let mut decoder = 
            bao::decode::SliceDecoder::new(proof_bytes, 
                                        &(deal_info.blake3_checksum), 
                                        chunk_offset, 
                                        chunk_size);

        match decoder.read_to_end(&mut decoded) {
            Ok(_res) => success_count += 1,
            Err(_e) => println!("Error in decoding: {:?}", window_num)
        };
    }
    if num_windows > 0
    {
        Ok(Json(MyResult {data: ResponseData { offer_id: offer_id, 
                                            success_count: success_count, 
                                            num_windows: num_windows as u64, 
                                            status: 200,
                                            result: "Ok".to_string()}}))
    }
    else {
        Ok(Json(MyResult {data: ResponseData { offer_id: offer_id, 
                                            success_count: success_count, 
                                            num_windows: num_windows as u64, 
                                            status: 500,
                                            result: "No windows found".to_string()}}))
    }
}

pub async fn validate_deal(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {
    validate_deal_internal(input_data).await.map_or_else(|e| construct_error(e), |v| v)
}