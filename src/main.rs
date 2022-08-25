
#[macro_use] 
extern crate rocket;
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod types;
mod validate;
mod proof_utils;

use rocket::serde::json::Json;
use eyre::Result;
use validate::{ChainlinkRequest, MyResult};

/* Implementing Responder for anyhow::Error.
   This is based on rocket_anyhow, but importing it wouldn't work. */
/*
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
*/
// check about timeouts with chainlink 

#[post("/val", format = "json", data = "<input_data>")]
async fn val(input_data: Json<ChainlinkRequest>) -> Json<MyResult> {

    /* Call your own function that returns a Json<MyResult>
       (MyResult is consistent with the Chainlink EA specs) */
    validate::validate_deal(input_data).await

}

#[rocket::main]
async fn main() -> Result<()> {

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
