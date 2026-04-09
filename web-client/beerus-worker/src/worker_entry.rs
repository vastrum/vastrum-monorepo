extern crate console_error_panic_hook;

#[wasm_bindgen]
pub fn init_beerus(_config_json: String) -> Result<(), JsError> {
    console_error_panic_hook::set_once();
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

async fn handle_rpc_request(request_json: &str) -> Result<String, String> {
    let req: StarknetRPCRequest =
        serde_json::from_str(request_json).map_err(|e| format!("bad request: {e}"))?;

    // Route starknet_call through beerus for proof-verified execution
    if req.method == "starknet_call" {
        if let Some(params) = req.params.as_array() {
            if let Some(call_obj) = params.first() {
                match crate::client::execute(call_obj).await {
                    Ok(result) => return serde_json::to_string(&result).map_err(|e| e.to_string()),
                    Err(e) => {
                        web_sys::console::warn_1(
                            &format!("beerus execute failed, falling back to direct RPC: {e}")
                                .into(),
                        );
                    }
                }
            }
        }
    }

    // Default path: direct JSON-RPC
    let result = crate::client::send_direct_rpc(&req.method, &req.params).await;
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

use vastrum_shared_types::iframerpc::types::StarknetRPCRequest;
use wasm_bindgen::prelude::*;
