use std::cell::RefCell;

use beerus::client::{Client, State};
use beerus::config::Config;
use serde_json::Value;

const STARKNET_RPC_URL: &str = "https://rpc.starknet.lava.build";

thread_local! {
    static BEERUS: RefCell<Option<BeerusState>> = RefCell::new(None);
}

struct BeerusState {
    client: Client<Http>,
    state: Option<State>,
}

/// Custom HTTP backend that supports both async (reqwest) and blocking (sync XHR) in WASM.
#[derive(Clone)]
pub struct Http(pub reqwest::Client);

impl Http {
    pub fn new() -> Self {
        Self(reqwest::Client::new())
    }
}

// Async: use reqwest (works fine in WASM)
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl beerus::r#gen::client::HttpClient for Http {
    async fn post(
        &self,
        url: &str,
        request: &iamgroot::jsonrpc::Request,
    ) -> std::result::Result<iamgroot::jsonrpc::Response, iamgroot::jsonrpc::Error> {
        let response = self
            .0
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| iamgroot::jsonrpc::Error::new(32101, format!("request failed: {e:?}")))?
            .json()
            .await
            .map_err(|e| {
                iamgroot::jsonrpc::Error::new(32102, format!("invalid response: {e:?}"))
            })?;
        Ok(response)
    }
}

// Blocking: use synchronous XMLHttpRequest on WASM (needed by exe::call during Cairo VM execution)
impl beerus::r#gen::client::blocking::HttpClient for Http {
    fn post(
        &self,
        url: &str,
        request: &iamgroot::jsonrpc::Request,
    ) -> std::result::Result<iamgroot::jsonrpc::Response, iamgroot::jsonrpc::Error> {
        let json = serde_json::to_string(&request)
            .map_err(|e| iamgroot::jsonrpc::Error::new(32101, format!("serialize failed: {e:?}")))?;

        let result = sync_post(url, &json)
            .map_err(|e| iamgroot::jsonrpc::Error::new(32101, format!("sync post failed: {e:?}")))?;

        let response = serde_json::from_str(&result)
            .map_err(|e| iamgroot::jsonrpc::Error::new(32102, format!("parse failed: {e:?}")))?;
        Ok(response)
    }
}

#[cfg(target_arch = "wasm32")]
fn sync_post(url: &str, body: &str) -> Result<String, String> {
    let xhr = web_sys::XmlHttpRequest::new().map_err(|e| format!("{e:?}"))?;
    xhr.open_with_async("POST", url, false)
        .map_err(|e| format!("{e:?}"))?;
    xhr.set_request_header("Content-Type", "application/json")
        .map_err(|e| format!("{e:?}"))?;
    xhr.send_with_opt_str(Some(body))
        .map_err(|e| format!("{e:?}"))?;

    let status = xhr.status().map_err(|e| format!("{e:?}"))?;
    if status != 200 {
        return Err(format!("HTTP {status}"));
    }

    xhr.response_text()
        .map_err(|e| format!("{e:?}"))?
        .ok_or_else(|| "empty response".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_post(_url: &str, _body: &str) -> Result<String, String> {
    unimplemented!("sync_post is only supported on wasm32")
}

pub async fn init() {
    let config = Config {
        starknet_rpc: STARKNET_RPC_URL.to_string(),
        #[cfg(not(target_arch = "wasm32"))]
        data_dir: Default::default(),
    };
    let http = Http::new();
    match Client::new(&config, http).await {
        Ok(client) => {
            let state = client.get_state().await.ok();
            BEERUS.with(|b| {
                *b.borrow_mut() = Some(BeerusState { client, state });
            });
            web_sys::console::log_1(&"beerus: initialized".into());
        }
        Err(e) => {
            web_sys::console::error_1(&format!("beerus: init failed: {e}").into());
        }
    }
}

pub async fn execute(request_json: &Value) -> Result<Value, String> {
    let initialized = BEERUS.with(|b| b.borrow().is_some());
    if !initialized {
        init().await;
    }

    BEERUS.with(|b| {
        let borrow = b.borrow();
        let Some(beerus) = borrow.as_ref() else {
            return Err("beerus not initialized".to_string());
        };
        let Some(state) = &beerus.state else {
            return Err("beerus state not available".to_string());
        };

        let call: beerus::r#gen::FunctionCall = serde_json::from_value(request_json.clone())
            .map_err(|e| format!("bad request: {e}"))?;

        let result = beerus
            .client
            .execute(call, state.clone())
            .map_err(|e| format!("execute failed: {e}"))?;

        let felts: Vec<String> = result.iter().map(|f| f.as_ref().to_string()).collect();
        Ok(serde_json::to_value(felts).unwrap())
    })
}

pub async fn send_direct_rpc(method: &str, params: &Value) -> Value {
    let envelope = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });

    let client = reqwest::Client::new();
    let response = client
        .post(STARKNET_RPC_URL)
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

pub fn rpc_error(msg: &str) -> Value {
    serde_json::json!({"error": {"code": -32603, "message": msg}})
}
