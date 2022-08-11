#[macro_use] 
extern crate rocket;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket::{response, Request};
use ethers::providers::{Middleware, Provider, Http};
use ethers::types::{Filter, Address};
use eyre;
use anyhow;

/* All these structs are for creating a JSON that has a "data" field (so it is 
compliant with Chainlink EA) and contains the input and the result. Valid is 
for valid data input, and Invalid is for invalid data input. They do not have 
to do with whether the block has been validated or not. The result field tells 
us the result of the block validation. */
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
enum Data {
    Valid(Valid),
    Invalid(Invalid)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(crate = "rocket::serde")]
struct MyResult {
    data: Data
}

/* checks if given number is even or odd, only accepts valid input */
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

/* route for forwarding invalid input */
#[get("/check/<num>", rank = 2)]
fn check2(num: &str) -> Json<MyResult> {
    Json(MyResult { 
            data: Data::Invalid(Invalid {
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

fn is_valid(data: &str) -> bool {
    let length = data.len();
    if length % 2 == 0 {
        return true;
    }
    return false;
}

/* can change these inputs but as of now 
   - num: block number
   - start: starting byte for data in log
   - end: ending byte for data in log
*/
#[get("/validate/<num>/<start>/<end>")]
async fn validate(num: u64, start: usize, end: usize) -> Result<Json<MyResult>, Error> {
    
    let provider = Provider::<Http>::try_from(
        "https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe" // "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
    ).expect("could not instantiate HTTP Provider");

    let filter = Filter::new().select(num).address("0x097384fa333a457599fb65aa0d931f3a756c3f12".parse::<Address>().unwrap());
    let block_log = provider.get_logs(&filter).await?;
    let data = &block_log[0].data;
    let data_bytes = data.get(start..end).ok_or(Error(anyhow::anyhow!("can't get data from {} to {}", start, end)))?;
    let data = std::str::from_utf8(data_bytes)?;
    
    if is_valid(data) {
        Ok(Json(MyResult { data: Data::Valid(Valid {number: num, result: "valid block".to_string()})}))
    }
    else {
        Ok(Json(MyResult { data: Data::Valid(Valid {number: num, result: "invalid block".to_string()})}))
    }
    
}

#[rocket::main]
async fn main() -> eyre::Result<()> {

    let _rocket = rocket::build()
        .mount("/", routes![check, check2, validate])
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
                   Some(MyResult{data: Data::Valid(Valid {number: 2, result: "even".to_string()})}));
    }

    #[test]
    fn test_odd() {
        let client = Client::tracked(rocket::build().mount("/", routes![check])).expect("valid rocket instance");
        let response = client.get(uri!(check(3))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(response.into_json(), 
                   Some(MyResult{data: Data::Valid(Valid {number: 3, result: "odd".to_string()})}));
    }

    #[test]
    fn test_invalid() {
        let client = Client::tracked(rocket::build().mount("/", routes![check2])).expect("valid rocket instance");
        let response = client.get(uri!(check2("asdf".to_string()))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(response.into_json(),
                   Some(MyResult{data: Data::Invalid(Invalid {number: "asdf".to_string(), result: "invalid".to_string()})}));
    }

    //11177037u64
    #[test]
    fn test_validate() {
        let client = Client::tracked(rocket::build().mount("/", routes![validate])).expect("valid rocket instance");
        let response = client.get(uri!(validate(11177037u64, 64, 91))).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(response.into_json(),
                   Some(MyResult{data: Data::Valid(Valid {number: 11177037u64, result: "Jonah and zev to the rescue".to_string()})}));
    }

}