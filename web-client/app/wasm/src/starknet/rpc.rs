pub async fn send_starknet_rpc(req: StarknetRPCRequest) -> Value {
    let envelope = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": req.method,
        "params": req.params,
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&req.rpc_url)
        .timeout(std::time::Duration::from_secs(30))
        .json(&envelope)
        .send()
        .await;

    let response = match response {
        Ok(r) => r,
        Err(e) => return rpc_error(&format!("fetch failed: {e}")),
    };

    let parsed: Value = match response.json().await {
        Ok(v) => v,
        Err(e) => return rpc_error(&format!("json parse failed: {e}")),
    };

    if let Some(result) = parsed.get("result") {
        return result.clone();
    }
    if let Some(error) = parsed.get("error") {
        return serde_json::json!({"error": error});
    }
    parsed
}

fn rpc_error(msg: &str) -> Value {
    serde_json::json!({"error": {"code": -32603, "message": msg}})
}

use serde_json::Value;
use vastrum_shared_types::iframerpc::types::StarknetRPCRequest;
