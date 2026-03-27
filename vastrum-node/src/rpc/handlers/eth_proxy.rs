//please dont dos 🙏
const EXECUTION_URLS: &[&str] = &[
    "https://wild-floral-model.quiknode.pro/fa5c46935beb34252e1246c4fdf2799152b14df4",
    "https://eth.drpc.org",
    "https://ethereum-rpc.publicnode.com",
];

pub const CONSENSUS_URLS: &[&str] = &[
    "https://wild-floral-model.quiknode.pro/fa5c46935beb34252e1246c4fdf2799152b14df4",
    "https://ethereum-beacon-api.publicnode.com",
    "https://lodestar-mainnet.chainsafe.io",
];
//https://drpc.org/chainlist
//https://ethereum.publicnode.com/

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub async fn eth_execution_proxy(path: &str, method: &str, body: Vec<u8>) -> EthProxyResponse {
    raw_proxy_with_fallback(EXECUTION_URLS, path, method, body).await
}

pub async fn eth_consensus_proxy(path: &str, method: &str, body: Vec<u8>) -> EthProxyResponse {
    raw_proxy_with_fallback(CONSENSUS_URLS, path, method, body).await
}

async fn raw_proxy_with_fallback(
    urls: &[&str],
    path: &str,
    method: &str,
    body: Vec<u8>,
) -> EthProxyResponse {
    let method = if method == "POST" { Method::POST } else { Method::GET };
    let mut last_error = String::from("no providers configured");

    for &provider_url in urls {
        match try_single_request(provider_url, path, &method, &body).await {
            AttemptOutcome::Final(response) => return response,
            AttemptOutcome::Retriable(err) => {
                last_error = format!("{provider_url}: {err}");
            }
        }
    }

    EthProxyResponse {
        status: 502,
        body: serde_json::to_vec(&serde_json::json!({
            "error": format!("all providers failed, last: {last_error}")
        }))
        .unwrap(),
        content_type: "application/json".into(),
    }
}

enum AttemptOutcome {
    Final(EthProxyResponse),
    Retriable(String),
}

async fn try_single_request(
    provider_url: &str,
    path: &str,
    method: &Method,
    body: &[u8],
) -> AttemptOutcome {
    let url = format!("{provider_url}{path}");
    let client = Client::builder().timeout(REQUEST_TIMEOUT).build().unwrap();

    let mut req = client.request(method.clone(), &url);
    if !body.is_empty() {
        req = req.header("content-type", "application/json").body(body.to_vec());
    }

    let resp = match req.send().await {
        Ok(resp) => resp,
        Err(e) => return AttemptOutcome::Retriable(e.to_string()),
    };

    let status = resp.status().as_u16();

    if !resp.status().is_success() {
        return AttemptOutcome::Retriable(format!("upstream returned {status}"));
    }

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json")
        .to_string();

    let body_bytes = match resp.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => return AttemptOutcome::Retriable(format!("read error: {e}")),
    };

    if body_bytes.len() > MAX_PROXY_BODY_SIZE {
        return AttemptOutcome::Final(EthProxyResponse {
            status: 502,
            body: br#"{"error":"response too large"}"#.to_vec(),
            content_type: "application/json".into(),
        });
    }

    AttemptOutcome::Final(EthProxyResponse { status, body: body_bytes, content_type })
}

use crate::utils::limits::MAX_PROXY_BODY_SIZE;
use reqwest::{Client, Method};
use serde_json;
use vastrum_shared_types::types::rpc::types::EthProxyResponse;
