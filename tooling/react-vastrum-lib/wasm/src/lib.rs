//used to ensure page navigation works correctly
#[wasm_bindgen]
pub async fn get_current_path() -> String {
    let path = vastrum_frontend_lib::get_current_path().await;
    return path;
}

#[wasm_bindgen]
pub async fn await_tx_inclusion(tx_hash_hex: String) {
    let digest = Sha256Digest::from_string(&tx_hash_hex).unwrap();
    if vastrum_frontend_lib::get_tx_hash_inclusion_state(digest).await {
        return;
    }
    TimeoutFuture::new(5).await;
    for _ in 0..240 {
        if vastrum_frontend_lib::get_tx_hash_inclusion_state(digest).await {
            return;
        }
        TimeoutFuture::new(500).await;
    }
}
#[wasm_bindgen]
pub async fn update_current_path(path: String, replace: bool) {
    vastrum_frontend_lib::update_current_path(path, replace).await;
}

#[wasm_bindgen]
pub async fn send_eth_rpc_request(request: JsValue) -> Result<JsValue, JsError> {
    let req: EthRPCRequest = serde_wasm_bindgen::from_value(request)?;
    let res = make_eth_rpc_request(req).await;

    if let Some(err_obj) = res.value_json.get("error") {
        let msg = err_obj.get("message").and_then(|m| m.as_str()).unwrap_or("unknown RPC error");
        return Err(JsError::new(msg));
    }
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    let js_value = res.value_json.serialize(&serializer)?;
    return Ok(js_value);
}

use gloo_timers::future::TimeoutFuture;
use serde::Serialize;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use vastrum_shared_types::iframerpc::types::EthRPCRequest;
use vastrum_frontend_lib::make_eth_rpc_request;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::prelude::*;
