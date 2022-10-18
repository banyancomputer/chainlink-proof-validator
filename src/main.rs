#![deny(unused_crate_dependencies)]

//use rust_chainlink_ea_api::validate::*;
pub mod validate;

use anyhow::Result;
use banyan_shared::eth::EthClient;
use rocket::serde::{json::serde_json, json::Json, Deserialize, Serialize};
use rocket::tokio::task::spawn;
use rocket::{post, State};
use std::sync::Arc;

pub struct WebserverState {
    pub provider: Arc<EthClient>,
    pub should_be_async: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainlinkEARequest {
    pub id: String,
    pub data: validate::ChainlinkRequestData,
    pub meta: Option<serde_json::Value>,
    pub response_url: Option<String>,
}

fn format_response(
    result: Result<validate::ChainlinkResponse, anyhow::Error>,
) -> Json<serde_json::Value> {
    match result {
        Ok(data) => Json(serde_json::json!(data)),
        Err(e) => Json(serde_json::json!({"error": e.to_string()})),
    }
}

// TODO prefix all logs with ID from request
#[post("/compute", format = "json", data = "<input_data>")]
pub async fn compute(
    webserver_state: &State<WebserverState>,
    input_data: Json<ChainlinkEARequest>,
) -> Json<serde_json::Value> {
    if webserver_state.should_be_async {
        let new_provider = webserver_state.provider.clone();
        spawn(async move {
            let result = validate::validate_deal_internal(new_provider, input_data.data.clone()).await;
            // send the result to the chainlink node
            reqwest::Client::new()
                .patch(input_data.into_inner().response_url.unwrap())
                .body(format_response(result).to_string())
                .send()
                .await
                .unwrap();
        });
        Json(serde_json::json!({
            "pending": true
        }))
        // end of thread
    } else {
        format_response(
            validate::validate_deal_internal(webserver_state.provider.clone(), input_data.data.clone())
                .await,
        )
    }
}

#[rocket::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let should_be_async = std::env::var("SHOULD_BE_ASYNC")
        .map_or_else(|_| false, |n| n.parse::<bool>().unwrap_or(false));

    // create an ethers HTTP provider
    let eth_client = Arc::new(EthClient::default());

    let _ = rocket::build()
        .mount("/", rocket::routes![compute])
        .manage(WebserverState {
            provider: eth_client,
            should_be_async,
        })
        .launch()
        .await?;

    Ok(())
}