
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

mod types;
mod validate;


/* Implementing Responder for anyhow::Error.
   This is based on rocket_anyhow, but importing it wouldn't work. */
#[derive(Debug)]
pub struct Error(
    pub anyhow::Error
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ChainlinkRequest {
    pub id: String,
    pub data: RequestData
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "rocket::serde")]
pub struct RequestData {
    pub offer_id: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
pub struct ResponseData {
    pub offer_id: u64,
    pub success_count: u8,
    pub num_windows: u8
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
pub struct MyResult {
    pub job_run_id: u64,
    pub data: ResponseData,
    pub status: u16,
    pub result: String
}


// check about timeouts with chainlink 

#[post("/val", format = "json", data = "<input_data>")]
async fn val(input_data: Json<ChainlinkRequest>) -> Json<MyResult> { // NEEDS TO RETURN THE CHAINLINK JSON ALWAYS
    // NEEDS TO RETURN THE CHAINLINK JSON ALWAYS
    // NEEDS TO RETURN THE CHAINLINK JSON ALWAYS
    // NEEDS TO RETURN THE CHAINLINK JSON ALWAYS
    // need to finish the get_deal_info function, full logic
    // need to correct the getter functions in get_deal_info

    /* Call your own function that returns a Result<Json<MyResult>, Error> */
    validate::validate_deal(input_data).await

}


#[rocket::main]
async fn main() -> eyre::Result<()> { // NEEDS TO RETURN THE CHAINLINK JSON ALWAYS

    let _rocket = rocket::build()
        .mount("/", routes![val])
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

    //11177037u64
    /*#[test]
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
    }*/

}
