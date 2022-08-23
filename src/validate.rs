

use rocket::serde::{/*Serialize, Deserialize,*/ json::{Json}};
//use rocket::{response, Request};
//use rocket::http::Status;
use ethers::{providers::{Middleware, Provider, Http},
             types::{Filter, H256, Address, U256},
             contract::{Contract},
             abi::{Abi}};
//use eyre;
use anyhow::anyhow;
use std::{io::Read,
          str::FromStr,
          fs};
use byteorder::{BigEndian, ByteOrder};
//use math;
use cid::Cid;
use multihash::Multihash;

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
pub async fn get_deal_info(offer_id: u64) -> Result<OnChainDealInfo, crate::Error> {
    let deal_id: DealID = DealID(offer_id);
    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" //"https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");
    let address = "0x464cBd3d0D8A2872cf04306c133118Beb5711111".parse::<Address>()?; //address of test contract
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

    let cid_return: U256 = contract
        .method::<_, U256>("getIpfsFileCid", deal_id.0)?
        .call()
        .await?; //should be memory pointer in solidity
    let bytes: &[u8; 8] = &cid_return.as_u64().to_be_bytes();
    let test_bytes = [
        0x16, 0x40, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8, 0x9e,
        0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c, 0x3b, 0xfb,
        0xf2, 0x4e,
        0x16, 0x20, 0x64, 0x4b, 0xcc, 0x7e, 0x56, 0x43, 0x73, 0x04, 0x09, 0x99, 0xaa, 0xc8, 0x9e,
        0x76, 0x22, 0xf3, 0xca, 0x71, 0xfb, 0xa1, 0xd9, 0x72, 0xfd, 0x94, 0xa3, 0x1c, 0x3b, 0xfb,
        0xf2, 0x4e, 
        0x11, 0x22
    ];
    let multihash: Multihash = Multihash::from_bytes(&test_bytes)?;
    let ipfs_file_cid: Cid = cid::CidGeneric::new_v1(multihash.code(), multihash);
    
    let file_size: u64 = contract
        .method::<_, u64>("getFileSize", deal_id.0)?
        .call()
        .await?;

    /*let blake3_checksum: String = contract
        .method::<_, String>("getBlake3Checksum", deal_id.0)?
        .call()
        .await?;*/ //should also be a memory pointer
    
    let blake3_checksum = bao::Hash::from_str("c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f").unwrap();
    //let blake3_checksum_actual = bao::Hash::from_str(&blake3_checksum).unwrap();

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
        blake3_checksum: blake3_checksum
    };

    println!("Deal info: {:?}", deal_info);

    Ok(deal_info)

}

pub async fn validate_deal(input_data: Json<ChainlinkRequest>) -> Result<Json<MyResult>, crate::Error> {
    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" //"https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");

    // getting deal info from on chain
    let request: ChainlinkRequest = input_data.into_inner();
    let offer_id = request.data.offer_id.trim().parse::<u64>().unwrap();
    let block_num = request.data.block_num.trim().parse::<u64>().unwrap();

    let deal_info: OnChainDealInfo = get_deal_info(offer_id).await?;

    // checking that deal is either finished or cancelled
    let current_block_num = provider.get_block_number().await?;
    let finished = BlockNum(current_block_num.as_u64()) > deal_info.deal_start_block + deal_info.deal_length_in_blocks;
    let cancelled = false; // need to figure out how to get this

    if !finished && !cancelled {
        return Err(crate::Error(anyhow!("Deal {} is ongoing", offer_id)));
    }

    let agreed_upon_cancellation_block: BlockNum = BlockNum(0u64); // need to figure out how to get this
    let deal_length_in_blocks = match cancelled {
        false => deal_info.deal_length_in_blocks,
        true => agreed_upon_cancellation_block + deal_info.deal_start_block
    };

    let window_size: u64 = 5; // need to figure out how to get this
    
    let _num_windows = math::round::ceil((deal_length_in_blocks.0 / window_size) as f64, 0);


    // SKIP TO END

    let filter = Filter::new().select(block_num).topic1(H256::from_low_u64_be(offer_id))/*.address("0xf679d8d8a90f66b4d8d9bf4f2697d53279f42bea".parse::<Address>().unwrap())*/;
    let block_logs = provider.get_logs(&filter).await?;
    //println!("Block logs: {:?}", block_logs);

    let data = &block_logs[0].data;
    //println!("data: {}", data);
    let data_size = data.get(56..64).ok_or(crate::Error(anyhow!("can't get data from 56 to 64")))?;
    //println!("data_size: {:?}", data_size);
    let actual_size = BigEndian::read_u64(data_size);
    println!("actual size: {}", actual_size);
    //let size: usize = usize::from(data_size);
    // Ok I need the hex value of datasize so I can get rid of the hardcoded length of the 
    // bao file below. data size is an &[u8], and you cant just get the value at 64 data.get(64)
    // since that is returning only one byte, but the size is denominated over several bytes. 

    let end: usize = (64 + actual_size) as usize;
    let data_bytes = data.get(64..end).ok_or(crate::Error(anyhow!("can't get data from {} to {}", 64, end)))?;

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

    Ok(Json(MyResult {job_run_id: 0,
                      data: ResponseData { offer_id: 0, success_count: 7, num_windows: 0 },
                      result: true }))
}
