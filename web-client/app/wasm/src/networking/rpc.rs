pub async fn get_key_value(site_id: Sha256Digest, key: String) -> Result<GetKeyValueResponse> {
    get_key_value_with_height(site_id, key, None).await
}

pub async fn get_key_value_with_height(
    site_id: Sha256Digest,
    key: String,
    height: Option<u64>,
) -> Result<GetKeyValueResponse> {
    let payload = GetKeyValuePayload { site_id, key: key.clone(), height_lock: height };
    let resp = send_request("getkeyvalue", &payload.encode()).await?;
    let result: GetKeyValueResult = borsh::from_slice(&resp)?;
    let response = match result {
        GetKeyValueResult::Ok(r) => r,
        GetKeyValueResult::Err(e) => return Err(WasmErr::RpcError(format!("{e:?}"))),
    };

    let data = read_frontend_data();
    proof_verification::verify_keyvalue_proof(
        &response,
        site_id,
        &key,
        &data.genesis_validators,
        data.total_validator_stake,
        (js_sys::Date::now() / 1000.0) as u64,
    )?;

    Ok(response)
}

pub async fn submit_call(site_id: Sha256Digest, call_data: Vec<u8>) -> Result<Sha256Digest> {
    let private_key = generate_private_key();
    let recent_block_height = get_latest_block_height().await?;

    let transaction = build_call_transaction(
        site_id,
        call_data,
        get_random_u64(),
        private_key,
        recent_block_height,
    );
    let payload = SubmitTransactionPayload { transaction_bytes: transaction.encode() };
    send_fire_and_forget("submit", &payload.encode()).await?;
    let tx_hash = transaction.calculate_txhash();
    return Ok(tx_hash);
}

pub async fn submit_authenticated_call(
    site_id: Sha256Digest,
    call_data: Vec<u8>,
    account_private_key: ed25519::PrivateKey,
) -> Result<Sha256Digest> {
    let recent_block_height = get_latest_block_height().await?;

    let transaction = build_call_transaction(
        site_id,
        call_data,
        get_random_u64(),
        account_private_key,
        recent_block_height,
    );
    let payload = SubmitTransactionPayload { transaction_bytes: transaction.encode() };
    send_fire_and_forget("submit", &payload.encode()).await?;
    let tx_hash = transaction.calculate_txhash();
    return Ok(tx_hash);
}

pub async fn get_latest_block_height() -> Result<u64> {
    let resp = send_request("getlatestblockheight", &[]).await?;
    let response: GetLatestBlockHeightResponse = borsh::from_slice(&resp)?;
    return Ok(response.height);
}

pub async fn get_tx_hash_inclusion_state(tx_hash: Sha256Digest) -> Result<bool> {
    let payload = GetTxHashIsIncluded { tx_hash };
    let resp = send_request("gettxhashinclusionstate", &payload.encode()).await?;
    let res: GetTxHashIsIncludedResponse = borsh::from_slice(&resp)?;
    return Ok(res.included);
}

pub async fn eth_proxy(url: String, method: String, body: Vec<u8>) -> Result<EthProxyResponse> {
    let payload = EthProxyRequest { url, method, body };
    let resp = send_request("ethproxy", &payload.encode()).await?;
    let response: EthProxyResponse = borsh::from_slice(&resp)?;
    Ok(response)
}

pub async fn connect_to_rpc() {
    let data = read_frontend_data();
    let selected_node_id = (get_random_u64() as usize) % data.rpc_nodes.len();
    let node = &data.rpc_nodes[selected_node_id];
    start_webrtc_connection(node.addr, node.fingerprint).await;
}

pub fn get_rpc_endpoint(route: &str) -> String {
    let hostname = web_sys::window().unwrap().location().hostname().unwrap();
    format!("http://{hostname}:{HTTP_RPC_PORT}/{route}/")
}

pub async fn get_page(page_path: String, site_identifier: String) -> Result<JSPageResponse> {
    let payload =
        GetPagePayload { site_identifier: site_identifier.clone(), page_path: page_path.clone() };
    let resp = send_request("page", &payload.encode()).await?;
    let result: GetPageResult = borsh::from_slice(&resp)?;
    let response = match result {
        GetPageResult::Ok(r) => r,
        GetPageResult::Err(e) => return Err(WasmErr::RpcError(format!("{e:?}"))),
    };

    let data = read_frontend_data();
    proof_verification::verify_page_proof(
        &response,
        &data.genesis_validators,
        data.total_validator_stake,
        (js_sys::Date::now() / 1000.0) as u64,
    )?;

    let is_exact_page_path = response.page_path == page_path;
    let is_index_page_fallback = response.page_path.is_empty();
    let not_valid_page = !is_exact_page_path && !is_index_page_fallback;
    if not_valid_page {
        return Err(WasmErr::RpcError(format!(
            "page path mismatch: requested '{}', got '{}'",
            page_path, response.page_path
        )));
    }

    set_current_site_id(response.site_id);
    let site_id = response.site_id.to_string();
    let content = brotli_decompress_html(&response.brotli_html_content)?;

    return Ok(JSPageResponse { content, site_id });
}

#[derive(Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi)]
pub struct JSPageResponse {
    pub content: String,
    pub site_id: String,
}

use crate::{
    crypto::keystore::generate_private_key,
    networking::connection::{send_fire_and_forget, send_request},
    read_frontend_data,
    utils::{
        error::{Result, WasmErr},
        get_random_u64,
        site_id::set_current_site_id,
    },
};
use serde::{Deserialize, Serialize};
use vastrum_shared_types::proof_verification;
use vastrum_shared_types::{
    borsh::BorshExt,
    compression::brotli::brotli_decompress_html,
    crypto::{ed25519, sha256::Sha256Digest},
    ports::HTTP_RPC_PORT,
    transactioning::transaction_generator::build_call_transaction,
    types::rpc::types::{
        EthProxyRequest, EthProxyResponse, GetKeyValuePayload, GetKeyValueResponse,
        GetKeyValueResult, GetLatestBlockHeightResponse, GetPagePayload, GetPageResult,
        GetTxHashIsIncluded, GetTxHashIsIncludedResponse, SubmitTransactionPayload,
    },
};
use tsify::Tsify;

use super::connection::start_webrtc_connection;
