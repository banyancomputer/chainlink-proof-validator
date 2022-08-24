
use rocket::serde::{/*Serialize, Deserialize,*/ json::{Json}};
//use rocket::{response, Request};
//use rocket::http::Status;
use ethers::{providers::{Middleware, Provider, Http},
             types::{Filter, H256, Address, U256},
             contract::{Contract},
             abi::{Abi}};
//use eyre;
use anyhow::anyhow;
use std::{io::{Read, Cursor},
          str::FromStr,
          fs};
use byteorder::{BigEndian, ByteOrder};
//use math;
use cid;
use multihash::Multihash;
use multibase::{decode};

use crate::{types::{OnChainDealInfo, DealID, BlockNum, TokenAmount, Token},
            ChainlinkRequest, MyResult, ResponseData};

pub(crate) const CHUNK_SIZE: usize = 1024;

fn is_valid(response: usize) -> bool {
    if response == 1024 {
        return true;
    }
    return false;
}

/*
    Gets the deal info from on chain.
*/
pub async fn get_deal_info(offer_id: u64) -> Result<(OnChainDealInfo, Vec<U256>), crate::Error> {
    let deal_id: DealID = DealID(offer_id);
    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" //"https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");
    let address = "0x24A95cffE14A9C3a0CfC2D7BcB0E059757A7f532".parse::<Address>()?; //address of test contract
    let abi: Abi = serde_json::from_str(fs::read_to_string("contract_abi.json").expect("can't read file").as_str())?;
    let contract = Contract::new(address, abi, provider);
    
    let deal_start_block: BlockNum = BlockNum(contract
        .method::<_, U256>("getDealStartBlock", deal_id.0)?
        .call()
        .await?
        .as_u64());

    let deal_length_in_blocks: BlockNum = BlockNum(contract
        .method::<_, U256>("getDealLengthInBlocks", deal_id.0)?
        .call()
        .await?
        .as_u64());

    let proof_frequency_in_blocks: BlockNum = BlockNum(contract
        .method::<_, U256>("getProofFrequencyInBlocks", deal_id.0)?
        .call()
        .await?
        .as_u64());

    let price: TokenAmount = TokenAmount(contract
        .method::<_, U256>("getPrice", deal_id.0)?
        .call()
        .await?
        .as_u64());

    let collateral: TokenAmount = TokenAmount(contract
        .method::<_, U256>("getCollateral", deal_id.0)?
        .call()
        .await?
        .as_u64());

    let erc20_token_denomination: Token = Token(contract
        .method::<_, Address>("getErc20TokenDenomination", deal_id.0)?
        .call()
        .await?);

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

    let proof_blocks: Vec<U256> = contract
        .method::<_, Vec<U256>>("getProofBlocks", deal_id.0)?
        .call()
        .await?;

    let deal_info: OnChainDealInfo = OnChainDealInfo { 
        deal_id: deal_id, 
        deal_start_block: deal_start_block, 
        deal_length_in_blocks: deal_length_in_blocks, 
        proof_frequency_in_blocks: proof_frequency_in_blocks, 
        price: price, 
        collateral: collateral, 
        erc20_token_denomination: erc20_token_denomination, 
        ipfs_file_cid: ipfs_file_cid, 
        file_size: file_size, 
        blake3_checksum: blake3_checksum,
    };

    println!("Deal info: {:?}", deal_info);
    println!("Proof blocks: {:?}", proof_blocks);

    Ok((deal_info, proof_blocks))

}

pub fn construct_error(status: u16, reason: String) -> Json<MyResult> {
    Json(MyResult {
        job_run_id: 0,
        data: ResponseData { offer_id: 0, success_count: 0, num_windows: 0 },
        status: status,
        result: reason
    })
}

pub async fn validate_deal(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {

    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" //"https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");

    // getting deal info from on chain
    let request: ChainlinkRequest = input_data.into_inner();
    let zev_do_not_change_this_unless_you_have_something_that_works = request.data.block_num.trim().parse::<u64>().unwrap();
    let offer_id = request.data.offer_id.trim().parse::<u64>().unwrap();
    let (deal_info, proof_blocks): (OnChainDealInfo, Vec<U256>) = match get_deal_info(offer_id).await {
        Ok((d, pb)) => (d, pb),
        Err(e) => return construct_error(500, format!("Error in get_deal_info: {:?}", e))
    };

    // checking that deal is either finished or cancelled
    let current_block_num = match provider.get_block_number().await {
        Ok(num) => num,
        Err(e) => return construct_error(500, format!("Couldn't get most recent block number: {:?}", e))
    };
    let finished = BlockNum(current_block_num.as_u64()) > deal_info.deal_start_block + deal_info.deal_length_in_blocks;
    let cancelled = false; // need to figure out how to get this

    if !finished && !cancelled {
        return Json(MyResult {job_run_id: 0,
                              data: ResponseData { offer_id: 0, success_count: 0, num_windows: 0 },
                              status: 500,
                              result: format!("Deal {} is not finished or cancelled.", offer_id)});
    }

    let agreed_upon_cancellation_block: BlockNum = BlockNum(0u64); // need to figure out how to get this
    let deal_length_in_blocks = match cancelled {
        false => deal_info.deal_length_in_blocks,
        true => agreed_upon_cancellation_block + deal_info.deal_start_block
    };

    let window_size: u64 = 5; // need to figure out how to get this
    
    let _num_windows = math::round::ceil((deal_length_in_blocks.0 / window_size) as f64, 0);

    // FIGURE OUT WINDOW STUFF

    for block in proof_blocks {
        let filter = Filter::new().select(block.low_u64()).topic1(H256::from_low_u64_be(offer_id));
    }
    
    let filter = Filter::new().select(zev_do_not_change_this_unless_you_have_something_that_works).topic1(H256::from_low_u64_be(offer_id))/*.address("0xf679d8d8a90f66b4d8d9bf4f2697d53279f42bea".parse::<Address>().unwrap())*/;
    let block_logs = match provider.get_logs(&filter).await {
        Ok(l) => l,
        Err(e) => return construct_error(500, format!("Couldn't get logs from block {}: {:?}", current_block_num, e))
    };
    //println!("Block logs: {:?}", block_logs);

    let data = &block_logs[0].data;
    //println!("data: {}", data);
    let data_size = match data.get(56..64) {// .ok_or(crate::Error(anyhow!("can't get data from 56 to 64")))?; 
        Some(size) => size,
        None => return construct_error(500, "Couldn't get size of proof data from bytes 56-64 in log".to_string())
    };
    //println!("data_size: {:?}", data_size);
    let actual_size = BigEndian::read_u64(data_size);
    println!("actual size: {}", actual_size);
    //let size: usize = usize::from(data_size);
    // Ok I need the hex value of datasize so I can get rid of the hardcoded length of the 
    // bao file below. data size is an &[u8], and you cant just get the value at 64 data.get(64)
    // since that is returning only one byte, but the size is denominated over several bytes. 

    let end: usize = (64 + actual_size) as usize;
    let data_bytes = match data.get(64..end) {//.ok_or(crate::Error(anyhow!("can't get data from {} to {}", 64, end)))?;
        Some(bytes) => bytes,
        None => return construct_error(500, format!("Couldn't get proof data from log. Problem reading bytes 64-{}", end))
    };
    let hash: bao::Hash = bao::Hash::from_str("c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f").unwrap();
    let start_index = 532480;

    let mut decoded = Vec::new();
    let mut decoder = bao::decode::SliceDecoder::new(
        data_bytes,
        &hash,
        start_index.try_into().unwrap(),
        CHUNK_SIZE.try_into().unwrap(),
    );
    let response = decoder.read_to_end(&mut decoded).unwrap();
    println!("response: {:?}", response);
    Json(MyResult {job_run_id: 0,
                      data: ResponseData { offer_id: 0, success_count: 100, num_windows: 0 },
                      status: 200,
                      result: "Ok".to_string() })
}
