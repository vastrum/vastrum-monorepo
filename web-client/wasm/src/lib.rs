#[wasm_bindgen]
pub async fn get_page(page_route: String) -> Result<JsValue, JsValue> {
    let url = get_rpc_endpoint("page".to_string());

    let payload = GetPagePayload {
        page_path: page_route,
    };

    // Serialize to JSON
    let json_body = serde_json::to_string(&payload)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&json_body));

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Accept", "application/json")?;
    request.headers().set("Content-Type", "application/json")?;

    let window = web_sys::window().ok_or("no window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!("HTTP error: {}", resp.status())));
    }

    let json = JsFuture::from(resp.json()?).await?;

    return Ok(json);
}

#[wasm_bindgen]
pub async fn make_call(site_id: String, calldata: String) -> Result<JsValue, JsValue> {
    let url = get_rpc_endpoint("submit".to_string());

    let private_key = ed25519::PrivateKey::from_seed(44);

    let site_id_bytes = hex_to_bytes(&site_id).unwrap();

    let site_id_bytes: [u8; 32] = site_id_bytes.try_into().unwrap();
    let site_id_digest = Sha256Digest::from(site_id_bytes);

    let call_tx_data = TransactionData {
        transaction_type: TransactionType::Call,
        calldata: SiteCall {
            site_id: site_id_digest,
            args: calldata.as_bytes().to_vec(),
        }
        .encode(),
    };

    let nonce = get_random_u64();

    let transaction = Transaction {
        pub_key: private_key.public_key(),
        signature: private_key.sign_hash(call_tx_data.calculate_hash()),
        calldata: call_tx_data.encode(),
        pow_nonce: nonce,
    };

    let payload = SubmitTransactionPayload {
        transaction_bytes: transaction.encode(),
    };

    // Serialize to JSON
    let json_body = serde_json::to_string(&payload)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsValue::from_str(&json_body));

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Accept", "application/json")?;
    request.headers().set("Content-Type", "application/json")?;

    let window = web_sys::window().ok_or("no window")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    let resp: Response = resp_value.dyn_into()?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!("HTTP error: {}", resp.status())));
    }

    return Ok(JsValue::from_str("ok"));
}
pub fn get_rpc_endpoint(route: String) -> String {
    let rpc_url = option_env!("FRONTEND_RPC_URL").unwrap_or("http://127.0.0.1:3000/");
    let url = format!("{}{}/", rpc_url, route);
    return url;
}
pub fn get_random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    let window = web_sys::window().unwrap();
    let crypto = window.crypto().unwrap();
    crypto.get_random_values_with_u8_array(&mut bytes).unwrap();

    // Convert 8 bytes to u64
    u64::from_le_bytes(bytes)
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect()
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub user_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPagePayload {
    pub page_path: String,
}

use serde::{Deserialize, Serialize};
use shared_types::{
    borsh::BorshExt,
    crypto::{ed25519, sha256::Sha256Digest},
    types::{
        application::{
            sitecall::SiteCall,
            transactiondata::{TransactionData, TransactionType},
        },
        execution::transaction::Transaction,
        rpc::types::SubmitTransactionPayload,
    },
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
