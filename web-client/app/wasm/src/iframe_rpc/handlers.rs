pub async fn handle_get_key_value(params: GetKeyValueRequest) -> Result<GetKeyValueResponse> {
    let site_id = get_current_site_id()?;
    let rpc_response = get_key_value_with_height(site_id, params.key, params.height).await;
    let value = match rpc_response {
        Ok(r) => r.value,
        Err(_) => vec![],
    };
    Ok(GetKeyValueResponse { value })
}

pub async fn handle_get_latest_block_height() -> Result<GetLatestBlockHeightResponse> {
    let height = get_latest_block_height().await?;
    Ok(GetLatestBlockHeightResponse { height })
}

pub async fn make_call(params: MakeCallRequest) -> Result<MakeCallResponse> {
    let site_id = get_current_site_id()?;
    let tx_hash = submit_call(site_id, params.call_data).await?;
    return Ok(MakeCallResponse { tx_hash });
}

pub async fn make_authenticated_call(params: MakeAuthCallRequest) -> Result<MakeAuthCallResponse> {
    let site_id = get_current_site_id()?;
    let tx_hash =
        submit_authenticated_call(site_id, params.call_data, keystore::get_site_private_key()?)
            .await?;
    return Ok(MakeAuthCallResponse { tx_hash });
}

pub async fn get_private_salt_for_site_id(
    _params: GetPrivateSalt,
) -> Result<GetPrivateSaltResponse> {
    let site_id = get_current_site_id()?;
    let private_key = keystore::get_account_private_key()?;
    let bytes = [site_id.encode(), b"VASTRUM_PRIVATE_SALT_NAMESPACE".to_vec()].concat();
    let signature = private_key.sign(&bytes);
    let salt = sha256_hash(&signature.encode());
    return Ok(GetPrivateSaltResponse { salt });
}

pub async fn get_site_pub_key(_params: GetPubKey) -> Result<GetPubKeyResponse> {
    let pub_key = keystore::get_site_private_key()?.public_key();
    return Ok(GetPubKeyResponse { pub_key });
}

pub async fn get_site_private_key(_params: GetPrivateKeyRpc) -> Result<GetPrivateKeyResponse> {
    let private_key = keystore::get_site_private_key()?;
    return Ok(GetPrivateKeyResponse { private_key });
}

pub async fn get_tx_hash_is_included(
    params: GetTXHashIsConfirmed,
) -> Result<GetTXHashIsConfirmedResponse> {
    let is_finalized = get_tx_hash_inclusion_state(params.tx_hash).await?;
    return Ok(GetTXHashIsConfirmedResponse { is_finalized });
}

pub async fn handle_eth_rpc_request(params: GetEthRPCRequest) -> GetEthRPCResponse {
    let res = send_eth_rpc_to_worker(params.request).await;
    let eth_rpc_response = EthRPCResponse { value_json: res };
    return GetEthRPCResponse { eth_rpc_response };
}

pub async fn handle_starknet_rpc_request(
    params: GetStarknetRPCRequest,
) -> GetStarknetRPCResponse {
    let res = crate::starknet::rpc::send_starknet_rpc(params.request).await;
    let starknet_rpc_response = StarknetRPCResponse { value_json: res };
    return GetStarknetRPCResponse { starknet_rpc_response };
}

pub fn get_current_path(_params: GetCurrentPath) -> GetCurrentPathResponse {
    let path = web_sys::window().unwrap().location().pathname().unwrap_or_default();
    GetCurrentPathResponse { path }
}

pub async fn update_current_path(params: UpdateCurrentPath) -> Result<UpdateCurrentPathResponse> {
    let history = web_sys::window().unwrap().history()?;
    if params.replace {
        history.replace_state_with_url(&JsValue::NULL, "", Some(&params.path))?;
    } else {
        history.push_state_with_url(&JsValue::NULL, "", Some(&params.path))?;
    }
    return Ok(UpdateCurrentPathResponse {});
}

use crate::crypto::keystore;
use crate::helios::worker::send_eth_rpc_to_worker;
use crate::networking::rpc::get_key_value_with_height;
use crate::networking::rpc::get_latest_block_height;
use crate::networking::rpc::get_tx_hash_inclusion_state;
use crate::networking::rpc::submit_authenticated_call;
use crate::networking::rpc::submit_call;
use crate::utils::error::Result;
use crate::utils::site_id::get_current_site_id;
use vastrum_shared_types::borsh::BorshExt;
use vastrum_shared_types::crypto::sha256::sha256_hash;
use vastrum_shared_types::iframerpc::types::*;
use wasm_bindgen::JsValue;
