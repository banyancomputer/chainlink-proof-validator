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
        types::{Address, Filter, H256, U256, H160}
};
use multibase::decode;
use multihash::Multihash;
use rocket::{serde::{json::Json, Deserialize, Serialize},
            post};
use std::{
    fs,
    io::{Cursor, Read},
    str::FromStr,
};

use banyan_shared::{proofs, types::*};
use dotenv::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    static ref API_TOKEN: String = std::env::var("API_KEY").expect("API_KEY must be set.");
    static ref PROVIDER: Provider<Http> = Provider::<Http>::try_from(API_TOKEN.as_str())
    .expect("could not instantiate HTTP Provider");
    static ref ABI: Abi = serde_json::from_str(
        fs::read_to_string("contract_abi.json")
            .expect("can't read file")
            .as_str()
        ).expect("couldn't load abi");
    static ref ADDRESS: H160 = "0xeb3d5882faC966079dcdB909dE9769160a0a00Ac"
        .parse::<Address>()
        .expect("could not parse contract address");
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    Success,
    Failure
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainlinkRequest {
    pub job_run_id: String,
    pub data: RequestData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestData {
    pub deal_id: DealID
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub deal_id: DealID, 
    pub success_count: u64,
    pub num_windows: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MyResult {
    pub data: ResponseData,
    pub status: Status,
    pub result: String
}

/*
    Gets the deal info from on chain.
*/
async fn get_deal_info(deal_id: DealID) -> Result<OnChainDealInfo, anyhow::Error> {

    let contract = Contract::new(*ADDRESS, (*ABI).clone(), &*PROVIDER);
    let offer_id = deal_id.0;

    let deal_start_block: BlockNum = BlockNum(
        contract
            .method::<_, U256>("getDealStartBlock", offer_id)?
            .call()
            .await?
            .as_u64(),
    );

    let deal_length_in_blocks: BlockNum = BlockNum(
        contract
            .method::<_, U256>("getDealLengthInBlocks", offer_id)?
            .call()
            .await?
            .as_u64(),
    );

    let proof_frequency_in_blocks: BlockNum = BlockNum(
        contract
            .method::<_, U256>("getProofFrequencyInBlocks", offer_id)?
            .call()
            .await?
            .as_u64(),
    );

    let price: TokenAmount = TokenAmount(
        contract
            .method::<_, U256>("getPrice", offer_id)?
            .call()
            .await?
            .as_u64(),
    );

    let collateral: TokenAmount = TokenAmount(
        contract
            .method::<_, U256>("getCollateral", offer_id)?
            .call()
            .await?
            .as_u64(),
    );

    let erc20_token_denomination: Token = Token(
        contract
            .method::<_, Address>("getErc20TokenDenomination", offer_id)?
            .call()
            .await?,
    );

    let cid_return: String = contract
        .method::<_, String>("getIpfsFileCid", offer_id)?
        .call()
        .await?;
    let code = "z".to_owned();
    let full_cid = format!("{}{}", code, cid_return);
    let (_, decoded) = decode(full_cid)?;
    let reader = Cursor::new(decoded);
    let ipfs_file_cid = cid::CidGeneric::new_v0(Multihash::read(reader)?)?;

    let file_size: u64 = contract
        .method::<_, u64>("getFileSize", offer_id)?
        .call()
        .await?;

    let blake3_return: String = contract
        .method::<_, String>("getBlake3Checksum", offer_id)?
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

/* Function to construct an error response to return to Chainlink */
fn construct_error(deal_id: DealID, reason: String) -> Json<MyResult> {
    Json(MyResult {
        data: ResponseData {
            deal_id,
            success_count: 0,
            num_windows: 0,
        },
        status: Status::Failure,
        result: reason
    })
}

/* Function to get the block number that is associated with a certain deal id 
   and window number */
async fn get_block_from_window(deal_id: DealID, window_num: u64) -> Result<u64, anyhow::Error> {
    let contract = Contract::new(*ADDRESS, (*ABI).clone(), &*PROVIDER);
    let block: u64 = contract
        .method::<_, U256>("getProofBlock", (deal_id.0, window_num))?
        .call()
        .await?
        .as_u64();

    return Ok(block);
}

/* Function to check if the deal is over or not */
fn deal_over(current_block_num: BlockNum, deal_info: OnChainDealInfo) -> bool {
    current_block_num > deal_info.deal_start_block + deal_info.deal_length_in_blocks
}

async fn validate_deal_internal(
    deal_id: DealID,
) -> Result<Json<MyResult>, String> {
    dotenv().ok();
    let mut success_count = 0;

    // getting deal info from on chain
    let deal_info = get_deal_info(deal_id)
        .await
        .map_err(|e| format!("Error in get_deal_info: {:?}", e))?;

    // checking that deal is either finished or cancelled
    let current_block_num = BlockNum((*PROVIDER)
        .get_block_number()
        .await
        .map_err(|e| format!("Couldn't get most recent block number: {e}"))?
        .as_u64());

    let deal_over = deal_over(current_block_num, deal_info);
    let deal_cancelled = false; // need to figure out how to get this

    if !deal_over && !deal_cancelled {
        return Ok(construct_error(deal_id, "Deal is ongoing".to_string()));
    }

    let agreed_upon_cancellation_block: BlockNum = BlockNum(0u64); // need to figure out how to get this
    let deal_length_in_blocks = match deal_cancelled {
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
        let block = get_block_from_window(deal_id, window_num as u64)
            .await
            .map_err(|e| format!("Could not get block: {e}"))?;

        let filter: Filter = Filter::new()
            .select(block)
            .topic1(H256::from_low_u64_be(deal_id.0));
        let block_logs = (*PROVIDER)
            .get_logs(&filter)
            .await
            .map_err(|e| format!("Couldn't get logs from block {}: {}", current_block_num.0, e))?;

        let proof_bytes = Cursor::new(&block_logs[0].data);

        // step c. above
        // impl Mul(u: usize) for BlockNum {} in types
        let target_window_start: BlockNum =
            BlockNum(window_num as u64 * window_size + deal_info.deal_start_block.0);

        // step d. above
        // write provider.get_block(block_num: BlockNum)
        let target_block_hash = (*PROVIDER)
            .get_block(target_window_start.0)
            .await
            .map_err(|e| format!("Could not get block number {}: {}", target_window_start.0, e))?
            .ok_or(format!("Could not unpack block number {}", target_window_start.0))?
            .hash
            .ok_or(format!("Could not get hash of block {}", target_window_start.0))?;

        // step e. above
        let (chunk_offset, chunk_size) =
            proofs::compute_random_block_choice_from_hash(target_block_hash, deal_info.file_size);

        // step f. above
        let mut decoded = Vec::new();
        let mut decoder = bao::decode::SliceDecoder::new(
            proof_bytes,
            &(deal_info.blake3_checksum),
            chunk_offset,
            chunk_size,
        );

        match decoder.read_to_end(&mut decoded) {
            Ok(_res) => success_count += 1,
            Err(_e) => println!("Error in decoding: {:?}", window_num),
        };
    }
    if num_windows > 0 {
        Ok(Json(MyResult {
            data: ResponseData {
                deal_id,
                success_count,
                num_windows: num_windows as u64,
            },
            status: Status::Success,
            result: "Ok".to_string()
        }))
    } else {
        Ok(Json(MyResult {
            data: ResponseData {
                deal_id,
                success_count,
                num_windows: num_windows as u64,
            },
            status: Status::Failure,
            result: "No windows found".to_string()
        }))
    }
}

async fn validate_deal(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {
    let deal_id = input_data.into_inner().data.deal_id;
    validate_deal_internal(deal_id)
        .await
        .map_or_else(|e| construct_error(deal_id, e), |v| v)
}

#[post("/validate", format = "json", data = "<input_data>")]
pub async fn validate(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {
    validate_deal(input_data).await
}