pub async fn get_key_value(key: String) -> GetKeyValueResponse {
    let params = GetKeyValueRequest { key, height: None };
    let res = send_request(params, RpcMethod::GetKeyValue).await.unwrap();
    return res;
}

pub async fn get_key_value_at_height(key: String, height: u64) -> GetKeyValueResponse {
    let params = GetKeyValueRequest { key, height: Some(height) };
    let res = send_request(params, RpcMethod::GetKeyValue).await.unwrap();
    return res;
}

pub async fn get_latest_block_height() -> u64 {
    let params = GetLatestBlockHeight {};
    let res: GetLatestBlockHeightResponse =
        send_request(params, RpcMethod::GetLatestBlockHeight).await.unwrap();
    let height = res.height;
    return height;
}

pub async fn get_key_value_by_site_id(site_id: Sha256Digest, key: String) -> GetKeyValueResponse {
    let params = GetKeyValueBySiteIdRequest { site_id, key };
    let res = send_request(params, RpcMethod::GetKeyValueBySiteId).await.unwrap();
    return res;
}

pub async fn make_call(call_data: Vec<u8>) -> MakeCallResponse {
    let params = MakeCallRequest { call_data };
    let res = send_request(params, RpcMethod::MakeCall).await.unwrap();
    return res;
}

pub async fn make_authenticated_call(call_data: Vec<u8>) -> MakeAuthCallResponse {
    let params = MakeAuthCallRequest { call_data };
    let res = send_request(params, RpcMethod::MakeAuthenticatedCall).await.unwrap();
    return res;
}

pub async fn get_private_salt(namespace: String) -> Sha256Digest {
    let params = GetPrivateSalt {};
    let res: GetPrivateSaltResponse =
        send_request(params, RpcMethod::GetPrivateSalt).await.unwrap();
    let bytes = [res.salt.encode(), namespace.as_bytes().to_vec()].concat();
    let salt = sha256_hash(&bytes);
    return salt;
}

pub async fn get_pub_key() -> ed25519::PublicKey {
    let params = GetPubKey {};
    let res: GetPubKeyResponse = send_request(params, RpcMethod::GetSitePubKey).await.unwrap();
    return res.pub_key;
}

pub async fn get_private_key() -> ed25519::PrivateKey {
    let params = GetPrivateKeyRpc {};
    let res: GetPrivateKeyResponse =
        send_request(params, RpcMethod::GetSitePrivateKey).await.unwrap();
    return res.private_key;
}

pub async fn get_tx_hash_inclusion_state(tx_hash: Sha256Digest) -> bool {
    let params = GetTXHashIsConfirmed { tx_hash };
    match send_request::<_, GetTXHashIsConfirmedResponse>(params, RpcMethod::GetTxHashIsIncluded)
        .await
    {
        Ok(r) => r.is_finalized,
        Err(()) => false,
    }
}

pub async fn make_eth_rpc_request(request: EthRPCRequest) -> EthRPCResponse {
    let params = GetEthRPCRequest { request };
    let res: GetEthRPCResponse = send_request(params, RpcMethod::EthRpcRequest).await.unwrap();
    res.eth_rpc_response
}

pub async fn get_current_path() -> String {
    let params = GetCurrentPath {};
    let res: GetCurrentPathResponse =
        send_request(params, RpcMethod::GetCurrentPath).await.unwrap();
    res.path
}

pub async fn update_current_path(path: String, replace: bool) {
    let params = UpdateCurrentPath { path, replace };
    let _res: UpdateCurrentPathResponse =
        send_request(params, RpcMethod::UpdateCurrentPath).await.unwrap();
}

pub fn navigate_to(params: &str) {
    let parsed: PageNavigationEventMessage = serde_json::from_str(params).unwrap();

    let window = window().unwrap();
    let init = CustomEventInit::new();
    init.set_detail(&JsValue::from_str(&parsed.path));

    let event = CustomEvent::new_with_event_init_dict("wasm-navigate", &init).unwrap();
    window.dispatch_event(&event).unwrap();
}

use crate::rpc::send_request;
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::ed25519;
use vastrum_shared_types::crypto::sha256::{Sha256Digest, sha256_hash};
use vastrum_shared_types::iframerpc::types::{
    EthRPCRequest, EthRPCResponse, GetCurrentPath, GetCurrentPathResponse, GetEthRPCRequest,
    GetEthRPCResponse, GetKeyValueBySiteIdRequest, GetKeyValueRequest, GetKeyValueResponse,
    GetLatestBlockHeight, GetLatestBlockHeightResponse, GetPrivateKeyResponse, GetPrivateKeyRpc,
    GetPrivateSalt, GetPrivateSaltResponse, GetPubKey, GetPubKeyResponse, GetTXHashIsConfirmed,
    GetTXHashIsConfirmedResponse, MakeAuthCallRequest, MakeAuthCallResponse, MakeCallRequest,
    MakeCallResponse, PageNavigationEventMessage, RpcMethod, UpdateCurrentPath,
    UpdateCurrentPathResponse,
};
use wasm_bindgen::prelude::*;
use web_sys::{CustomEvent, CustomEventInit, window};
