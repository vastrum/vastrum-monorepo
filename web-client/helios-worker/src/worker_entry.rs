extern crate console_error_panic_hook;

#[wasm_bindgen]
pub fn init_helios(config_json: String) -> Result<(), JsError> {
    console_error_panic_hook::set_once();
    let config: HeliosConfig = serde_json::from_str(&config_json)
        .map_err(|e| JsError::new(&format!("invalid helios config: {e}")))?;
    set_helios_config(config);
    Ok(())
}

#[wasm_bindgen]
pub async fn worker_rpc(request_json: String) -> String {
    console_error_panic_hook::set_once();
    match handle_rpc_request(&request_json).await {
        Ok(json) => json,
        Err(e) => serde_json::json!({
            "error": {"code": -32603, "message": e.to_string()}
        })
        .to_string(),
    }
}
async fn handle_rpc_request(request_json: &str) -> eyre::Result<String> {
    let req: EthRPCRequest = serde_json::from_str(request_json)?;
    let result = resolve_eth_rpc_request(req).await?;
    Ok(serde_json::to_string(&result)?)
}

use crate::client::provider::{HeliosConfig, set_helios_config};
use crate::client::rpc::resolve_eth_rpc_request;
use vastrum_shared_types::iframerpc::types::EthRPCRequest;
use wasm_bindgen::prelude::*;
