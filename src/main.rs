#[macro_use] 
extern crate rocket;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use rocket::serde::{Serialize, Deserialize, json::Json};
use ethers::providers::{Middleware, Provider, Http};
use eyre::Result;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Valid {
    number: u64,
    result: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct Invalid {
    number: String,
    result: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
enum Data {
    Valid(Valid),
    Invalid(Invalid)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct MyResult {
    data: Data
}

#[get("/check/<num>")]
fn check(num: u64) -> Json<MyResult> {
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
    Json(MyResult { data: Data:: Valid(d)})
}

#[get("/check/<num>", rank = 2)]
fn check2(num: &str) -> Json<MyResult> {
    Json(MyResult { 
            data: Data::Invalid(Invalid {
                                    number: num.to_string(),
                                    result: "invalid".to_string()
            }) 
        })
}

#[rocket::main]
async fn main() -> Result<()> {

    let provider = Provider::<Http>::try_from(
        "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");
    let current_block_number = provider.get_block_number().await?;
    let block_info = provider.get_block(current_block_number).await?;
    let block_num = serde_json::to_string(&current_block_number)?;
    let block_hash: String;
    match block_info {
        None    => block_hash = String::from(""),
        Some(h) => block_hash = serde_json::to_string(&h.hash)?
    }
    println!("Got block: {}", block_num);
    println!("Block hash: {}", block_hash);

    let _rocket = rocket::build()
        .mount("/", routes![check, check2])
        .launch()
        .await?;

    Ok(())
}
