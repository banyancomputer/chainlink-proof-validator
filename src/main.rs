
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

//cast send 0xabc123 "saveProof(uint256 offerId, bytes memory _proof)" 3 --rpc-url https://eth-mainnet.alchemyapi.io (--private-key=abc123)

#[macro_use] 
extern crate rocket;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use rocket::serde::{Serialize, Deserialize, json::{Json}};
use rocket::{response, Request};
//use rocket::http::Status;
use ethers::{providers::{Middleware, Provider, Http},
             types::{Filter, H256, Address, U256},
             contract::{Contract},
             abi::{Abi}};
use eyre;
use anyhow::anyhow;
use std::{io::Read,
          str::FromStr,
          fs};
use byteorder::{BigEndian, ByteOrder};
//use math;
use cid::Cid;
use multihash::MultihashGeneric;

mod types;

use types::{OnChainDealInfo, DealID, BlockNum, TokenAmount, Token};

pub(crate) const CHUNK_SIZE: usize = 1024;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct Valid {
    number: u64,
    result: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct Invalid {
    number: String,
    result: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
enum OutputDataTest {
    Valid(Valid),
    Invalid(Invalid)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct MyResultTest {
    data: OutputDataTest
}

/* checks if given number is even or odd, only accepts valid input */
#[get("/check/<num>")]
fn check(num: u64) -> Json<MyResultTest> {
    let mut d = Valid {
        number: num,
        result: String::new()
    };
    if num % 2 == 0 {
        d.result = "even".to_string();
    }
    else {
        d.result = "odd".to_string();
    }
    Json(MyResultTest { data: OutputDataTest:: Valid(d)})
}

/* route for forwarding invalid input */
#[get("/check/<num>", rank = 2)]
fn check2(num: &str) -> Json<MyResultTest> {
    Json(MyResultTest { 
            data: OutputDataTest::Invalid(Invalid {
                                    number: num.to_string(),
                                    result: "invalid".to_string()
            }) 
        })
}

/* Implementing Responder for anyhow::Error.
   This is based on rocket_anyhow, but importing it wouldn't work. */
#[derive(Debug)]
struct Error(
    anyhow::Error
);

impl<E> From<E> for crate::Error
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Error(error.into())
    }
}

impl<'r> response::Responder<'r, 'static> for Error {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static, > {
        response::Debug(self.0).respond_to(request)
    }
}

fn is_valid(response: usize) -> bool {
    if response == 1024 {
        return true;
    }
    return false;
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct DataTest {
    block_num: String, //don't need, can get list of relevant blocks from just offerId
    offer_id: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq)] //make sure serialize works with u64
#[serde(crate = "rocket::serde")]
struct InputDataTest {
    id: String,
    data: DataTest
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "rocket::serde")]
struct ChainlinkRequest {
    id: u64,
    data: RequestData
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(crate = "rocket::serde")]
struct RequestData {
    offer_id: u64
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct ResponseData {
    offer_id: u64,
    success_count: u8,
    num_windows: u8
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct MyResult {
    job_run_id: u64,
    data: ResponseData,
    //status: rocket::http::Status,
    result: bool
}

/*
    Gets the deal info from on chain.
*/
async fn get_deal_info(request: ChainlinkRequest) -> Result<OnChainDealInfo, Error> {
    let deal_id: DealID = DealID(request.data.offer_id);
    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" //"https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");
    let address = "0x464cBd3d0D8A2872cf04306c133118Beb5711111".parse::<Address>()?; //address of test contract
    let abi: Abi = serde_json::from_str(fs::read_to_string("src/contract_abi.json").expect("can't read file").as_str())?;
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
        .await?;
    let multihash = MultihashGeneric::from_bytes(&cid_return.as_u64().to_be_bytes())?;
    let ipfs_file_cid: Cid = cid::CidGeneric::new_v1(cid_return.as_u64(), multihash);

    let file_size: u64 = contract
        .method::<_, u64>("getFileSize", deal_id.0)?
        .call()
        .await?;

    /*let blake3_checksum: bao::Hash = contract
        .method::<_, _>("getBlake3Checksum", deal_id.0)?
        .call()
        .await?;*/
    
    let blake3_checksum = bao::Hash::from_str("c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f").unwrap();

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

// check about timeouts with chainlink 
 
#[post("/validate", format = "json", data = "<input_data>")]
async fn validate(input_data: Json<ChainlinkRequest>) -> Result<Json<MyResult>, Error> {

    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" //"https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");

    // getting deal info from on chain
    let request: ChainlinkRequest = input_data.into_inner();
    let deal_info: OnChainDealInfo = get_deal_info(request).await?;

    // checking that deal is either finished or cancelled
    let current_block_num = provider.get_block_number().await?;
    let finished = BlockNum(current_block_num.as_u64()) > deal_info.deal_start_block + deal_info.deal_length_in_blocks;
    let cancelled = false; // need to figure out how to get this

    if !finished && !cancelled {
        return Err(Error(anyhow!("Deal {} is ongoing", request.data.offer_id)));
    }

    let agreed_upon_cancellation_block: BlockNum = BlockNum(0u64); // need to figure out how to get this
    let deal_length_in_blocks = match cancelled {
        false => deal_info.deal_length_in_blocks,
        true => agreed_upon_cancellation_block + deal_info.deal_start_block
    };

    let window_size: u64 = 5; // need to figure out how to get this
    
    let num_windows = math::round::ceil((deal_length_in_blocks.0 / window_size) as f64, 0);

    Ok(Json(MyResult {job_run_id: 0,
                      data: ResponseData { offer_id: 0, success_count: 0, num_windows: 0 },
                      result: true }))
}

#[post("/validatefake", format = "json", data = "<input_data>")]
async fn validatefake(input_data: Json<InputDataTest>) -> Result<Json<MyResultTest>, Error> {
    println!("Running validate");
    let provider = Provider::<Http>::try_from(
        "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" //"https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");

    //let num2 = 11208056u64; // for testing purposes hardcoded
    let block_num = input_data.data.block_num.parse::<u64>().unwrap();
    let offer_id = input_data.data.offer_id.parse::<u64>().unwrap();
    let filter = Filter::new().select(block_num).topic1(H256::from_low_u64_be(offer_id))/*.address("0xf679d8d8a90f66b4d8d9bf4f2697d53279f42bea".parse::<Address>().unwrap())*/;
    let block_logs = provider.get_logs(&filter).await?;
    println!("Block logs: {:?}", block_logs);

    let data = &block_logs[0].data;
    println!("data: {}", data);
    let data_size = data.get(56..64).ok_or(Error(anyhow!("can't get data from 56 to 64")))?;
    println!("data_size: {:?}", data_size);
    let actual_size = BigEndian::read_u64(data_size);
    println!("actual size: {}", actual_size);
    //let size: usize = usize::from(data_size);
    // Ok I need the hex value of datasize so I can get rid of the hardcoded length of the 
    // bao file below. data size is an &[u8], and you cant just get the value at 64 data.get(64)
    // since that is returning only one byte, but the size is denominated over several bytes. 

    let end: usize = (64 + actual_size) as usize;
    let data_bytes = data.get(64..end).ok_or(Error(anyhow!("can't get data from {} to {}", 64, end)))?;

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
    println!("Response: {}", response); 
    if is_valid(response) {
        Ok(Json(MyResultTest { data: OutputDataTest::Valid(Valid {number: block_num, result: "yay!".to_string()})}))
    }
    else {
        Ok(Json(MyResultTest { data: OutputDataTest::Valid(Valid {number: block_num, result: "oh no!".to_string()})}))
    }
}


#[rocket::main]
async fn main() -> eyre::Result<()> {
    //let request: ChainlinkRequest = ChainlinkRequest { id: 0, data: RequestData { offer_id: 5378008} };

    //env::set_var("RUST_BACKTRACE", "1");
    let _rocket = rocket::build()
        .mount("/", routes![check, check2, validatefake])
        .launch()
        .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::http::Status;
    use rocket::http::ContentType;

    #[test]
    fn test_even() {
        let client = Client::tracked(rocket::build().mount("/", routes![check, check2])).expect("valid rocket instance");
        let response = client.get(uri!(check(2))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(response.into_json(), 
                   Some(MyResultTest{data: OutputDataTest::Valid(Valid {number: 2, result: "even".to_string()})}));
    }

    #[test]
    fn test_odd() {
        let client = Client::tracked(rocket::build().mount("/", routes![check])).expect("valid rocket instance");
        let response = client.get(uri!(check(3))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(response.into_json(), 
                   Some(MyResultTest{data: OutputDataTest::Valid(Valid {number: 3, result: "odd".to_string()})}));
    }

    #[test]
    fn test_invalid() {
        let client = Client::tracked(rocket::build().mount("/", routes![check2])).expect("valid rocket instance");
        let response = client.get(uri!(check2("asdf".to_string()))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(response.into_json(),
                   Some(MyResultTest{data: OutputDataTest::Invalid(Invalid {number: "asdf".to_string(), result: "invalid".to_string()})}));
    }

    //11177037u64
    #[test]
    fn test_validate() {
        let test_stuff: DataTest = DataTest {
            block_num: 11177037u64,
            offer_id: 613
        };
        let test_input: InputDataTest = InputDataTest {
            id: 0,
            data: test_stuff
        };
        let test_data = serde_json::to_string(&test_input).unwrap();
        let client: Client = Client::tracked(rocket::build().mount("/", routes![validatefake])).expect("valid rocket instance");
        let response = client.post(uri!(validatefake())).header(ContentType::JSON).body(test_data).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(response.into_json(),
                   Some(MyResultTest{data: OutputDataTest::Valid(Valid {number: 11208056u64, result: "yay!".to_string()})}));
    }

}
